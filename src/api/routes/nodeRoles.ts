import { Router, type Request, type Response } from "express";
import type { NodeRoleApplicationService, NodeRoleApplicationOptions } from "../../core/NodeRoleApplicationService";
import type { CreateCustomRoleInput, NodeRoleManager } from "../../core/NodeRoleManager";
import type { NodeManager } from "../../core/NodeManager";
import type { PluginId, RoleSyncStrategy, StorageEngine } from "../../types";
import type { AuthenticatedRequest } from "../middleware/auth";
import { ApiError, Errors } from "../errors";
import { respondWithApiError } from "../respond";

interface NodeRolesRouterDeps {
  roleManager: Pick<NodeRoleManager, "listRoles" | "getRole" | "createCustomRole">;
  applicationService: Pick<NodeRoleApplicationService, "plan" | "apply">;
}

interface NodeRoleApplicationsRouterDeps {
  nodeManager: Pick<NodeManager, "getNode">;
  roleManager: Pick<NodeRoleManager, "listApplications">;
}

type MaybeAuthenticatedRequest = Request & {
  user?: AuthenticatedRequest["user"];
};

const VALID_STORAGE_ENGINES = new Set<StorageEngine>(["leveldb", "rocksdb"]);
const VALID_NODE_TYPES = new Set(["neo-cli", "neo-go", "neox-go"]);
const VALID_SYNC_STRATEGIES = new Set<RoleSyncStrategy>(["full", "light", "fast-sync"]);
const VALID_PLUGIN_IDS = new Set<PluginId>([
  "ApplicationLogs",
  "DBFTPlugin",
  "LevelDBStore",
  "OracleService",
  "RestServer",
  "RocksDBStore",
  "RpcServer",
  "SignClient",
  "SQLiteWallet",
  "StateService",
  "StorageDumper",
  "TokensTracker",
]);

function nodeRoleNotFound(roleId: string): ApiError {
  return new ApiError(
    "NODE_ROLE_NOT_FOUND",
    `Node role ${roleId} not found`,
    "Choose an existing role from GET /api/node-roles.",
    404,
  );
}

function invalidRequest(message: string, suggestion = "Fix the request body and try again."): ApiError {
  return new ApiError("NODE_ROLE_REQUEST_INVALID", message, suggestion);
}

function parseStorageEngine(value: unknown): StorageEngine | undefined {
  if (value === undefined || value === null || value === "") return undefined;
  if (typeof value !== "string" || !VALID_STORAGE_ENGINES.has(value as StorageEngine)) {
    throw new ApiError(
      "INVALID_STORAGE_ENGINE",
      `Invalid storage engine: ${String(value || "(empty)")}`,
      'Use "leveldb" or "rocksdb".',
    );
  }
  return value as StorageEngine;
}

function requireNodeId(value: unknown): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw Errors.missingField("nodeId");
  }
  return value;
}

function parseApplicationOptions(body: unknown): { nodeId: string; options: NodeRoleApplicationOptions } {
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    throw Errors.missingField("nodeId");
  }
  const record = body as Record<string, unknown>;
  return {
    nodeId: requireNodeId(record.nodeId),
    options: {
      storageEngine: parseStorageEngine(record.storageEngine),
    },
  };
}

function parseCreateCustomRoleInput(body: unknown, createdBy: string | undefined): CreateCustomRoleInput {
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    throw Errors.missingFields("name", "nodeTypes", "profile");
  }
  const record = body as Record<string, unknown>;
  const missing = [
    typeof record.name === "string" && record.name.trim() !== "" ? null : "name",
    Array.isArray(record.nodeTypes) && record.nodeTypes.length > 0 ? null : "nodeTypes",
    record.profile && typeof record.profile === "object" && !Array.isArray(record.profile) ? null : "profile",
  ].filter((field): field is string => Boolean(field));
  if (missing.length > 0) {
    throw Errors.missingFields(...missing);
  }
  const nodeTypes = record.nodeTypes as unknown[];
  if (!nodeTypes.every((type) => typeof type === "string" && VALID_NODE_TYPES.has(type))) {
    throw invalidRequest("nodeTypes must contain supported node types");
  }
  if (record.description !== undefined && typeof record.description !== "string") {
    throw invalidRequest("description must be a string");
  }

  const profile = parseRoleProfile(record.profile, nodeTypes as string[]);

  return {
    name: String(record.name).trim(),
    description: typeof record.description === "string" ? record.description : undefined,
    nodeTypes: nodeTypes as CreateCustomRoleInput["nodeTypes"],
    profile,
    createdBy,
  };
}

function parseRoleProfile(value: unknown, nodeTypes: string[]): CreateCustomRoleInput["profile"] {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw invalidRequest("profile must be an object");
  }
  const profile = value as Record<string, unknown>;

  if (profile.storageEngine !== undefined) {
    parseRequiredStorageEngine(profile.storageEngine, "profile.storageEngine");
  }
  if (profile.settings !== undefined && (!profile.settings || typeof profile.settings !== "object" || Array.isArray(profile.settings))) {
    throw invalidRequest("profile.settings must be an object");
  }
  if (profile.plugins !== undefined) {
    parseRolePlugins(profile.plugins, nodeTypes);
  }
  if (profile.dataContext !== undefined) {
    parseRoleDataContext(profile.dataContext);
  }
  if (profile.sync !== undefined) {
    parseRoleSync(profile.sync);
  }
  for (const field of ["warnings", "prerequisites"]) {
    const list = profile[field];
    if (list !== undefined && (!Array.isArray(list) || !list.every((item) => typeof item === "string"))) {
      throw invalidRequest(`profile.${field} must be an array of strings`);
    }
  }

  return profile as CreateCustomRoleInput["profile"];
}

function parseRequiredStorageEngine(value: unknown, field: string): StorageEngine {
  if (typeof value !== "string" || !VALID_STORAGE_ENGINES.has(value as StorageEngine)) {
    throw invalidRequest(`${field} must be "leveldb" or "rocksdb"`);
  }
  return value as StorageEngine;
}

function parseRolePlugins(value: unknown, nodeTypes: string[]): void {
  if (!Array.isArray(value)) {
    throw invalidRequest("profile.plugins must be an array");
  }
  if (value.length > 0 && !nodeTypes.every((nodeType) => nodeType === "neo-cli")) {
    throw invalidRequest("profile.plugins are supported only for neo-cli custom roles");
  }
  for (const [index, plugin] of value.entries()) {
    if (!plugin || typeof plugin !== "object" || Array.isArray(plugin)) {
      throw invalidRequest(`profile.plugins[${index}] must be an object`);
    }
    const desired = plugin as Record<string, unknown>;
    if (typeof desired.id !== "string" || !VALID_PLUGIN_IDS.has(desired.id as PluginId)) {
      throw invalidRequest(`profile.plugins[${index}].id must be a supported plugin id`);
    }
    if (typeof desired.enabled !== "boolean") {
      throw invalidRequest(`profile.plugins[${index}].enabled must be a boolean`);
    }
    if (desired.config !== undefined && (!desired.config || typeof desired.config !== "object" || Array.isArray(desired.config))) {
      throw invalidRequest(`profile.plugins[${index}].config must be an object`);
    }
  }
}

function parseRoleDataContext(value: unknown): void {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw invalidRequest("profile.dataContext must be an object");
  }
  const dataContext = value as Record<string, unknown>;
  if (dataContext.mode !== "reuse-or-create" && dataContext.mode !== "always-create") {
    throw invalidRequest('profile.dataContext.mode must be "reuse-or-create" or "always-create"');
  }
  if (typeof dataContext.labelTemplate !== "string" || dataContext.labelTemplate.trim() === "") {
    throw invalidRequest("profile.dataContext.labelTemplate must be a non-empty string");
  }
}

function parseRoleSync(value: unknown): void {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw invalidRequest("profile.sync must be an object");
  }
  const sync = value as Record<string, unknown>;
  if (typeof sync.strategy !== "string" || !VALID_SYNC_STRATEGIES.has(sync.strategy as RoleSyncStrategy)) {
    throw invalidRequest('profile.sync.strategy must be "full", "light", or "fast-sync"');
  }
  if (sync.allowCheckpoint !== undefined && typeof sync.allowCheckpoint !== "boolean") {
    throw invalidRequest("profile.sync.allowCheckpoint must be a boolean");
  }
}

export function createNodeRolesRouter(deps: NodeRolesRouterDeps): Router {
  const router = Router();

  router.get("/", (_req: Request, res: Response) => {
    try {
      res.json({ roles: deps.roleManager.listRoles() });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/:roleId", (req: Request, res: Response) => {
    try {
      const roleId = String(req.params.roleId);
      const role = deps.roleManager.getRole(roleId);
      if (!role) {
        throw nodeRoleNotFound(roleId);
      }
      res.json({ role });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/", (req: MaybeAuthenticatedRequest, res: Response) => {
    try {
      const role = deps.roleManager.createCustomRole(parseCreateCustomRoleInput(req.body, req.user?.id));
      res.status(201).json({ role });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/:roleId/plan", (req: Request, res: Response) => {
    try {
      const roleId = String(req.params.roleId);
      const { nodeId, options } = parseApplicationOptions(req.body);
      const plan = deps.applicationService.plan(roleId, nodeId, options);
      res.json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/:roleId/apply", async (req: MaybeAuthenticatedRequest, res: Response) => {
    try {
      const roleId = String(req.params.roleId);
      const { nodeId, options } = parseApplicationOptions(req.body);
      const result = await deps.applicationService.apply(roleId, nodeId, options, req.user?.id);
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}

export function createNodeRoleApplicationsRouter(deps: NodeRoleApplicationsRouterDeps): Router {
  const router = Router({ mergeParams: true });

  router.get("/", (req: Request<{ id: string }>, res: Response) => {
    try {
      const node = deps.nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      res.json({ applications: deps.roleManager.listApplications(req.params.id) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
