import { ApiError, Errors } from "../api/errors";
import type { NodeDataContextManager } from "./NodeDataContextManager";
import type { NodeManager } from "./NodeManager";
import type { NodeRoleManager } from "./NodeRoleManager";
import type {
  NodeDataContext,
  NodeInstance,
  NodeRoleApplication,
  NodeRoleApplicationPlan,
  NodeRoleProfile,
  NodeSettings,
  PluginId,
  RoleSyncStrategy,
  StorageEngine,
} from "../types";

export interface NodeRoleApplicationOptions {
  storageEngine?: StorageEngine;
}

export interface NodeRoleApplicationServiceDeps {
  roleManager: NodeRoleManager;
  dataContextManager: NodeDataContextManager;
  nodeManager: Pick<NodeManager,
    | "getNode"
    | "ensureStorageEngine"
    | "installPlugin"
    | "updatePluginConfig"
    | "setPluginEnabled"
    | "updateNode"
  >;
}

export interface NodeRoleApplicationResult {
  application: NodeRoleApplication;
  node: NodeInstance;
}

interface PreparedDataContext {
  context: NodeDataContext;
  created: boolean;
}

const VALID_STORAGE_ENGINES = new Set<StorageEngine>(["leveldb", "rocksdb"]);

function nodeNotStopped(node: NodeInstance): ApiError {
  return new ApiError(
    "NODE_NOT_STOPPED",
    `Node must be stopped before applying roles; current status is ${node.process.status}`,
    "Stop the node, then retry the role application.",
    409,
  );
}

function roleNotFound(roleId: string): ApiError {
  return new ApiError(
    "NODE_ROLE_NOT_FOUND",
    `Node role ${roleId} not found`,
    "Choose an existing role from GET /api/node-roles.",
    404,
  );
}

function roleIncompatible(role: NodeRoleProfile, node: NodeInstance): ApiError {
  return new ApiError(
    "NODE_ROLE_INCOMPATIBLE",
    `Role ${role.name} is not compatible with ${node.type}`,
    "Choose a role that supports this node type.",
  );
}

function rolePluginsUnsupported(role: NodeRoleProfile, node: NodeInstance): ApiError {
  return new ApiError(
    "NODE_ROLE_PLUGIN_UNSUPPORTED",
    `Role ${role.name} contains plugin changes that are not supported for ${node.type}`,
    "Use plugin-bearing roles only with neo-cli nodes, or remove plugin desired state from the custom role.",
  );
}

function rolePrerequisiteMissing(message: string): ApiError {
  return new ApiError(
    "NODE_ROLE_PREREQUISITE_MISSING",
    message,
    "Complete the role prerequisites before applying this role.",
  );
}

function invalidStorageEngine(storageEngine: string): ApiError {
  return new ApiError(
    "INVALID_STORAGE_ENGINE",
    `Invalid storage engine: ${storageEngine || "(empty)"}`,
    'Use "leveldb" or "rocksdb".',
  );
}

function cloneJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function renderLabel(template: string, node: NodeInstance, storageEngine: StorageEngine, role: NodeRoleProfile): string {
  return template
    .replaceAll("{network}", node.network)
    .replaceAll("{storageEngine}", storageEngine)
    .replaceAll("{role}", role.id || role.name);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function settingsWithoutStorageEngine(settings: Partial<NodeSettings> | undefined): Partial<NodeSettings> {
  if (!settings) return {};
  const { storageEngine: _storageEngine, ...rest } = settings;
  return rest;
}

function roleSettingsForNode(role: NodeRoleProfile, node: NodeInstance): Partial<NodeSettings> {
  const settings = settingsWithoutStorageEngine(role.profile.settings);
  if (
    settings.keyProtection?.mode === "secure-signer"
    && !settings.keyProtection.signerProfileId
    && node.settings.keyProtection?.mode === "secure-signer"
  ) {
    return {
      ...settings,
      keyProtection: {
        ...node.settings.keyProtection,
        ...settings.keyProtection,
      },
    };
  }
  return settings;
}

export class NodeRoleApplicationService {
  constructor(private readonly deps: NodeRoleApplicationServiceDeps) {}

  plan(roleId: string, nodeId: string, options: NodeRoleApplicationOptions = {}): NodeRoleApplicationPlan {
    const node = this.requireNode(nodeId);
    const role = this.requireCompatibleRole(roleId, node);
    const storageEngine = this.normalizeStorageEngine(options.storageEngine);
    const activeDataContext = this.deps.dataContextManager.getActiveContext(nodeId) ?? undefined;

    return this.deps.roleManager.planRoleApplication({
      roleId: role.id,
      node,
      storageEngineOverride: storageEngine,
      activeDataContext,
    });
  }

  async apply(
    roleId: string,
    nodeId: string,
    options: NodeRoleApplicationOptions = {},
    appliedBy?: string,
  ): Promise<NodeRoleApplicationResult> {
    let node = this.requireNode(nodeId);
    const role = this.requireCompatibleRole(roleId, node);
    if (node.process.status !== "stopped") {
      throw nodeNotStopped(node);
    }
    this.preflightRole(role, node);

    const storageEngineOverride = this.normalizeStorageEngine(options.storageEngine);
    const activeDataContext = this.deps.dataContextManager.getActiveContext(nodeId) ?? undefined;
    const plan = this.deps.roleManager.planRoleApplication({
      roleId: role.id,
      node,
      storageEngineOverride,
      activeDataContext,
    });
    const previousState = {
      settings: cloneJson(node.settings ?? {}),
      plugins: cloneJson(node.plugins ?? []),
      activeDataContext: activeDataContext ? cloneJson(activeDataContext) : null,
    };
    let preparedDataContext: PreparedDataContext | null = null;

    try {
      const desiredStorageEngine = storageEngineOverride
        ?? role.profile.storageEngine
        ?? activeDataContext?.storageEngine
        ?? node.settings.storageEngine
        ?? "leveldb";

      preparedDataContext = this.prepareDataContext(role, node, desiredStorageEngine, activeDataContext);

      if ((node.settings.storageEngine ?? "leveldb") !== desiredStorageEngine) {
        node = await this.deps.nodeManager.ensureStorageEngine(nodeId, desiredStorageEngine);
      }

      if (node.type === "neo-cli") {
        node = await this.applyPlugins(role, node);
      }

      const activatedContext = preparedDataContext
        ? this.deps.dataContextManager.activateContext(nodeId, preparedDataContext.context.id)
        : null;
      const roleSettings = roleSettingsForNode(role, node);
      node = await this.deps.nodeManager.updateNode(nodeId, {
        settings: {
          ...roleSettings,
          ...(activatedContext ? {
            activeDataContextId: activatedContext.id,
            syncStrategy: activatedContext.syncStrategy,
          } : {}),
          role: {
            id: role.id,
            name: role.name,
            appliedAt: Date.now(),
          },
        },
      });

      const application = this.deps.roleManager.recordApplication({
        nodeId,
        roleId: role.id,
        roleName: role.name,
        applicationPlan: plan,
        previousState,
        appliedBy,
        status: "applied",
      });

      return { application, node };
    } catch (error) {
      this.rollbackPreparedDataContext(nodeId, activeDataContext, preparedDataContext);
      this.deps.roleManager.recordApplication({
        nodeId,
        roleId: role.id,
        roleName: role.name,
        applicationPlan: plan,
        previousState: {
          ...previousState,
          resultingState: this.captureState(nodeId),
        },
        appliedBy,
        status: "failed",
        errorMessage: errorMessage(error),
      });
      throw error;
    }
  }

  private requireNode(nodeId: string): NodeInstance {
    const node = this.deps.nodeManager.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    return node;
  }

  private requireCompatibleRole(roleId: string, node: NodeInstance): NodeRoleProfile {
    const role = this.deps.roleManager.getRole(roleId);
    if (!role) {
      throw roleNotFound(roleId);
    }
    if (!role.nodeTypes.includes(node.type)) {
      throw roleIncompatible(role, node);
    }
    if (node.type !== "neo-cli" && (role.profile.plugins?.length ?? 0) > 0) {
      throw rolePluginsUnsupported(role, node);
    }
    return role;
  }

  private preflightRole(role: NodeRoleProfile, node: NodeInstance): void {
    const keyProtection = role.profile.settings?.keyProtection;
    const existingSignerProfileId = node.settings.keyProtection?.mode === "secure-signer"
      ? node.settings.keyProtection.signerProfileId
      : undefined;
    if (keyProtection?.mode === "secure-signer" && !keyProtection.signerProfileId && !existingSignerProfileId) {
      throw rolePrerequisiteMissing(`Role ${role.name} requires a secure signer profile before it can be applied`);
    }
  }

  private captureState(nodeId: string): Record<string, unknown> {
    const currentNode = this.deps.nodeManager.getNode(nodeId);
    const activeDataContext = this.deps.dataContextManager.getActiveContext(nodeId);
    return {
      settings: currentNode ? cloneJson(currentNode.settings ?? {}) : null,
      plugins: currentNode ? cloneJson(currentNode.plugins ?? []) : null,
      activeDataContext: activeDataContext ? cloneJson(activeDataContext) : null,
    };
  }

  private normalizeStorageEngine(storageEngine: StorageEngine | undefined): StorageEngine | undefined {
    if (storageEngine === undefined) return undefined;
    if (!VALID_STORAGE_ENGINES.has(storageEngine)) {
      throw invalidStorageEngine(storageEngine);
    }
    return storageEngine;
  }

  private prepareDataContext(
    role: NodeRoleProfile,
    node: NodeInstance,
    storageEngine: StorageEngine,
    activeDataContext: NodeDataContext | undefined,
  ): PreparedDataContext | null {
    const desiredContext = role.profile.dataContext;
    if (!desiredContext) {
      return null;
    }

    const syncStrategy: RoleSyncStrategy = role.profile.sync?.strategy
      ?? activeDataContext?.syncStrategy
      ?? node.settings.syncStrategy
      ?? node.syncMode;
    const label = renderLabel(desiredContext.labelTemplate, node, storageEngine, role);
    const existing = desiredContext.mode === "reuse-or-create"
      ? this.deps.dataContextManager.listContexts(node.id).find((context) =>
        context.label === label
        && context.storageEngine === storageEngine
        && context.syncStrategy === syncStrategy)
      : undefined;
    const created = !existing;
    const context = existing ?? this.deps.dataContextManager.createContext(node.id, {
      label,
      storageEngine,
      syncStrategy,
    }, { active: false });

    return { context, created };
  }

  private rollbackPreparedDataContext(
    nodeId: string,
    previousActiveContext: NodeDataContext | undefined,
    preparedDataContext: PreparedDataContext | null,
  ): void {
    if (!preparedDataContext) {
      return;
    }

    try {
      const currentActive = this.deps.dataContextManager.getActiveContext(nodeId);
      if (currentActive?.id === preparedDataContext.context.id && previousActiveContext) {
        this.deps.dataContextManager.activateContext(nodeId, previousActiveContext.id);
      }
      if (preparedDataContext.created) {
        this.deps.dataContextManager.deleteContext(nodeId, preparedDataContext.context.id);
      }
    } catch {
      // The failed application record captures the resulting state for manual recovery.
    }
  }

  private async applyPlugins(role: NodeRoleProfile, node: NodeInstance): Promise<NodeInstance> {
    for (const desiredPlugin of role.profile.plugins ?? []) {
      let latestNode = this.requireNode(node.id);
      const currentPlugin = (latestNode.plugins ?? []).find((plugin) => plugin.id === desiredPlugin.id);
      if (desiredPlugin.enabled) {
        if (!currentPlugin) {
          await this.deps.nodeManager.installPlugin(node.id, desiredPlugin.id as PluginId, desiredPlugin.config);
        } else if (desiredPlugin.config) {
          this.deps.nodeManager.updatePluginConfig(node.id, desiredPlugin.id as PluginId, desiredPlugin.config);
        }
        await this.deps.nodeManager.setPluginEnabled(node.id, desiredPlugin.id as PluginId, true);
      } else if (currentPlugin) {
        await this.deps.nodeManager.setPluginEnabled(node.id, desiredPlugin.id as PluginId, false);
      }
      latestNode = this.requireNode(node.id);
      node = latestNode;
    }

    return this.requireNode(node.id);
  }
}
