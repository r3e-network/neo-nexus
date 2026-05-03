import crypto from 'node:crypto';
import type Database from 'better-sqlite3';
import type {
  InstalledPlugin,
  NodeInstance,
  NodeDataContext,
  NodeRoleApplication,
  NodeRoleApplicationPlan,
  NodeRolePluginDesiredState,
  NodeRoleProfile,
  NodeRoleProfileBody,
  NodeSettings,
  NodeType,
  RoleSyncStrategy,
  StorageEngine,
} from '../types/index';
import type { NodeRoleApplicationRow, NodeRoleProfileRow } from '../types/database';

export interface RolePlanInput {
  roleId: string;
  node: NodeInstance;
  storageEngineOverride?: StorageEngine;
  activeDataContext?: Pick<NodeDataContext, 'id' | 'label' | 'storageEngine' | 'syncStrategy' | 'active'>;
}

export interface CreateCustomRoleInput {
  name: string;
  description?: string;
  nodeTypes: NodeType[];
  profile: NodeRoleProfileBody;
  createdBy?: string;
}

const now = () => Date.now();

const cloneRole = (role: NodeRoleProfile): NodeRoleProfile => JSON.parse(JSON.stringify(role)) as NodeRoleProfile;

const parseJson = <T>(value: string, context: string): T => {
  try {
    return JSON.parse(value) as T;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to parse ${context}: ${message}`);
  }
};

const deepFreeze = <T>(value: T): T => {
  if (value && typeof value === 'object' && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) {
      deepFreeze(child);
    }
    Object.freeze(value);
  }

  return value;
};

const BUILTIN_ROLE_TEMPLATES: readonly NodeRoleProfile[] = deepFreeze([
  {
    id: 'builtin-rpc-api',
    name: 'RPC / API Node',
    description: 'Expose JSON-RPC and optional REST APIs for wallets, dApps, and monitoring.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'leveldb',
      settings: { relay: true, maxConnections: 80, minPeers: 10, maxPeers: 60 },
      plugins: [{ id: 'RpcServer', enabled: true, config: {} }],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'rpc-{network}-{storageEngine}' },
      sync: { strategy: 'fast-sync', allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: 'builtin-state',
    name: 'State Node',
    description: 'Track state roots and serve state proof workflows.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'rocksdb',
      settings: { relay: true, maxConnections: 80, minPeers: 10, maxPeers: 60 },
      plugins: [
        { id: 'StateService', enabled: true, config: { AutoStart: true, FullState: false } },
        { id: 'RpcServer', enabled: true, config: {} },
      ],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'state-{network}-{storageEngine}' },
      sync: { strategy: 'fast-sync', allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: 'builtin-oracle',
    name: 'Oracle Node',
    description: 'Run OracleService for off-chain data requests.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'leveldb',
      settings: { relay: true, minPeers: 10, maxPeers: 60 },
      plugins: [
        { id: 'OracleService', enabled: true, config: { AutoStart: true } },
        { id: 'RpcServer', enabled: true, config: {} },
      ],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'oracle-{network}-{storageEngine}' },
      sync: { strategy: 'fast-sync', allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: 'builtin-consensus',
    name: 'Consensus Node',
    description: 'Configure dBFT consensus participation for validator nodes.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'leveldb',
      settings: { relay: true, minPeers: 10, maxPeers: 80 },
      plugins: [{ id: 'DBFTPlugin', enabled: true, config: { AutoStart: false } }],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'consensus-{network}-{storageEngine}' },
      sync: { strategy: 'full', allowCheckpoint: false },
      warnings: ['Consensus nodes require validator key material or a secure signing workflow.'],
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: 'builtin-indexer',
    name: 'Indexer Node',
    description: 'Index application logs and token transfers.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'rocksdb',
      settings: { relay: true, maxConnections: 100, minPeers: 10, maxPeers: 80 },
      plugins: [
        { id: 'ApplicationLogs', enabled: true, config: {} },
        { id: 'TokensTracker', enabled: true, config: {} },
        { id: 'RpcServer', enabled: true, config: {} },
      ],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'indexer-{network}-{storageEngine}' },
      sync: { strategy: 'fast-sync', allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: 'builtin-secure-signer-client',
    name: 'Secure Signer Client',
    description: 'Use SignClient for external signing.',
    kind: 'builtin',
    nodeTypes: ['neo-cli'],
    profile: {
      storageEngine: 'leveldb',
      settings: { keyProtection: { mode: 'secure-signer' } },
      plugins: [{ id: 'SignClient', enabled: true, config: {} }],
      dataContext: { mode: 'reuse-or-create', labelTemplate: 'signer-client-{network}-{storageEngine}' },
      sync: { strategy: 'full', allowCheckpoint: false },
      prerequisites: ['Select a secure signer profile before applying this role.'],
    },
    createdAt: 0,
    updatedAt: 0,
  },
]);

export const BUILTIN_ROLES: readonly NodeRoleProfile[] = deepFreeze(BUILTIN_ROLE_TEMPLATES.map(cloneRole));

const isPlainObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const deepEqual = (left: unknown, right: unknown): boolean => {
  if (Object.is(left, right)) return true;
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right) || left.length !== right.length) return false;
    return left.every((item, index) => deepEqual(item, right[index]));
  }
  if (isPlainObject(left) || isPlainObject(right)) {
    if (!isPlainObject(left) || !isPlainObject(right)) return false;
    const leftKeys = Object.keys(left);
    const rightKeys = Object.keys(right);
    if (leftKeys.length !== rightKeys.length) return false;
    return leftKeys.every((key) => Object.hasOwn(right, key) && deepEqual(left[key], right[key]));
  }

  return false;
};

const matchesDesiredValue = (current: unknown, desired: unknown): boolean => {
  if (isPlainObject(desired) && isPlainObject(current)) {
    return Object.entries(desired).every(([key, value]) =>
      Object.hasOwn(current, key) && matchesDesiredValue(current[key], value));
  }

  return deepEqual(current, desired);
};

const containsConfig = (current: Record<string, unknown>, desired: Record<string, unknown>): boolean =>
  Object.entries(desired).every(([key, value]) => Object.hasOwn(current, key) && matchesDesiredValue(current[key], value));

export class NodeRoleManager {
  constructor(private readonly db: Database.Database) {}

  listRoles(): NodeRoleProfile[] {
    const customRows = this.db.prepare('SELECT * FROM node_role_profiles ORDER BY name').all() as NodeRoleProfileRow[];
    return [...BUILTIN_ROLE_TEMPLATES.map(cloneRole), ...customRows.map((row) => this.mapRoleRow(row))];
  }

  getRole(roleId: string): NodeRoleProfile | null {
    const builtin = BUILTIN_ROLE_TEMPLATES.find((role) => role.id === roleId);
    if (builtin) return cloneRole(builtin);

    const row = this.db.prepare('SELECT * FROM node_role_profiles WHERE id = ?').get(roleId) as
      | NodeRoleProfileRow
      | undefined;

    return row ? this.mapRoleRow(row) : null;
  }

  createCustomRole(input: CreateCustomRoleInput): NodeRoleProfile {
    const timestamp = now();
    const nodeTypes = [...input.nodeTypes];
    const profile = JSON.parse(JSON.stringify(input.profile)) as NodeRoleProfileBody;
    const role: NodeRoleProfile = {
      id: `role-${crypto.randomUUID()}`,
      name: input.name,
      description: input.description,
      kind: 'custom',
      nodeTypes,
      profile,
      createdBy: input.createdBy,
      createdAt: timestamp,
      updatedAt: timestamp,
    };

    this.db.prepare(`
      INSERT INTO node_role_profiles (id, name, description, kind, node_types, profile, created_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      role.id,
      role.name,
      role.description ?? null,
      role.kind,
      JSON.stringify(role.nodeTypes),
      JSON.stringify(role.profile),
      role.createdBy ?? null,
      role.createdAt,
      role.updatedAt,
    );

    return cloneRole(role);
  }

  planRoleApplication(input: RolePlanInput): NodeRoleApplicationPlan {
    const role = this.getRole(input.roleId);
    if (!role) throw new Error(`Role ${input.roleId} not found`);
    if (!role.nodeTypes.includes(input.node.type)) {
      throw new Error(`Role ${role.name} is not compatible with ${input.node.type}`);
    }

    const currentStorageEngine = input.activeDataContext?.storageEngine ?? input.node.settings.storageEngine ?? 'leveldb';
    const storageEngine = input.storageEngineOverride
      ?? role.profile.storageEngine
      ?? currentStorageEngine
      ?? 'leveldb';
    const plugins = role.profile.plugins ?? [];
    const warnings = [...(role.profile.warnings ?? []), ...(role.profile.prerequisites ?? [])];
    const changes: NodeRoleApplicationPlan['changes'] = [];

    if (currentStorageEngine !== storageEngine) {
      changes.push({ type: 'storage', summary: `Switch storage engine to ${storageEngine}` });
    }
    const desiredSyncStrategy = role.profile.sync?.strategy;
    if (role.profile.dataContext) {
      const label = this.renderLabel(role.profile.dataContext.labelTemplate, input.node, storageEngine, role);
      if (this.dataContextNeedsChange(role.profile.dataContext.mode, label, storageEngine, desiredSyncStrategy, input)) {
        changes.push({
          type: 'data-context',
          summary: `${role.profile.dataContext.mode === 'always-create' ? 'Create' : 'Ensure'} data context ${label}`,
        });
      }
    }
    for (const plugin of plugins) {
      if (this.pluginNeedsChange(plugin, input.node.plugins ?? [])) {
        changes.push({ type: 'plugin', summary: `${plugin.enabled ? 'Enable' : 'Disable'} ${plugin.id}` });
      }
    }
    if (this.settingsNeedChange(role.profile.settings, input.node.settings)) {
      changes.push({
        type: 'settings',
        summary: 'Apply node settings from role',
      });
    }
    if (desiredSyncStrategy && this.syncStrategyNeedsChange(desiredSyncStrategy, input)) {
      changes.push({
        type: 'fast-sync',
        summary: `Switch sync strategy to ${desiredSyncStrategy}`,
      });
    }

    return {
      nodeId: input.node.id,
      roleId: role.id,
      roleName: role.name,
      requiresRestart: changes.length > 0,
      changes,
      warnings,
    };
  }

  recordApplication(application: Omit<NodeRoleApplication, 'id' | 'appliedAt'>): NodeRoleApplication {
    const record: NodeRoleApplication = {
      ...application,
      id: `role-app-${crypto.randomUUID()}`,
      appliedAt: now(),
    };

    this.db.prepare(`
      INSERT INTO node_role_applications (id, node_id, role_id, role_name, application_plan, previous_state, applied_at, applied_by, status, error_message)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      record.id,
      record.nodeId,
      record.roleId,
      record.roleName,
      JSON.stringify(record.applicationPlan),
      record.previousState ? JSON.stringify(record.previousState) : null,
      record.appliedAt,
      record.appliedBy ?? null,
      record.status,
      record.errorMessage ?? null,
    );

    return record;
  }

  listApplications(nodeId: string): NodeRoleApplication[] {
    const rows = this.db.prepare('SELECT * FROM node_role_applications WHERE node_id = ? ORDER BY applied_at DESC')
      .all(nodeId) as NodeRoleApplicationRow[];

    return rows.map((row) => ({
      id: row.id,
      nodeId: row.node_id,
      roleId: row.role_id,
      roleName: row.role_name,
      applicationPlan: parseJson<NodeRoleApplicationPlan>(
        row.application_plan,
        `node_role_applications.application_plan for ${row.id}`,
      ),
      previousState: row.previous_state
        ? parseJson<Record<string, unknown>>(
          row.previous_state,
          `node_role_applications.previous_state for ${row.id}`,
        )
        : undefined,
      appliedAt: row.applied_at,
      appliedBy: row.applied_by ?? undefined,
      status: row.status as NodeRoleApplication['status'],
      errorMessage: row.error_message ?? undefined,
    }));
  }

  private mapRoleRow(row: NodeRoleProfileRow): NodeRoleProfile {
    return {
      id: row.id,
      name: row.name,
      description: row.description ?? undefined,
      kind: row.kind as NodeRoleProfile['kind'],
      nodeTypes: parseJson<NodeType[]>(row.node_types, `node_role_profiles.node_types for ${row.id}`),
      profile: parseJson<NodeRoleProfileBody>(row.profile, `node_role_profiles.profile for ${row.id}`),
      createdBy: row.created_by ?? undefined,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }

  private pluginNeedsChange(desired: NodeRolePluginDesiredState, currentPlugins: InstalledPlugin[]): boolean {
    const current = currentPlugins.find((plugin) => plugin.id === desired.id);
    if (!current) return desired.enabled;
    if (current.enabled !== desired.enabled) return true;
    if (!desired.enabled) return false;

    return desired.config ? !containsConfig(current.config, desired.config) : false;
  }

  private settingsNeedChange(desired: Partial<NodeSettings> | undefined, current: NodeSettings): boolean {
    if (!desired) return false;

    return Object.entries(desired).some(([key, value]) => !matchesDesiredValue(current[key as keyof NodeSettings], value));
  }

  private dataContextNeedsChange(
    mode: NonNullable<NodeRoleProfileBody['dataContext']>['mode'],
    label: string,
    storageEngine: StorageEngine,
    desiredSyncStrategy: RoleSyncStrategy | undefined,
    input: RolePlanInput,
  ): boolean {
    if (mode === 'always-create') return true;

    const activeContext = input.activeDataContext;
    if (!activeContext?.active) return true;

    return activeContext.label !== label
      || activeContext.storageEngine !== storageEngine
      || (desiredSyncStrategy ? activeContext.syncStrategy !== desiredSyncStrategy : false);
  }

  private syncStrategyNeedsChange(desired: RoleSyncStrategy, input: RolePlanInput): boolean {
    if (input.activeDataContext?.active) {
      return input.activeDataContext.syncStrategy !== desired;
    }

    return (input.node.syncMode ?? 'full') !== desired;
  }

  private renderLabel(
    template: string,
    node: NodeInstance,
    storageEngine: StorageEngine,
    role: Pick<NodeRoleProfile, 'id' | 'name'>,
  ): string {
    return template
      .replaceAll('{network}', node.network)
      .replaceAll('{storageEngine}', storageEngine)
      .replaceAll('{role}', role.id || role.name);
  }
}
