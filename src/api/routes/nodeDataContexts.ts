import { Router, type Request, type Response } from "express";
import type { NodeDataContextManager } from "../../core/NodeDataContextManager";
import type { NodeManager } from "../../core/NodeManager";
import type { RoleSyncStrategy, StorageEngine } from "../../types";
import { ApiError, Errors } from "../errors";
import { respondWithApiError } from "../respond";

interface NodeDataContextsRouterDeps {
  nodeManager: Pick<NodeManager, "getNode" | "updateNode" | "ensureStorageEngine">;
  dataContextManager: Pick<NodeDataContextManager, "listContexts" | "getActiveContext" | "createContext" | "activateContext" | "deleteContext">;
}

interface CreateContextRequestBody {
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
}

const VALID_STORAGE_ENGINES = new Set<StorageEngine>(["leveldb", "rocksdb"]);
const VALID_SYNC_STRATEGIES = new Set<RoleSyncStrategy>(["full", "light", "fast-sync"]);

function invalidDataContext(message: string, suggestion = "Fix the data context request and try again."): ApiError {
  return new ApiError("DATA_CONTEXT_INVALID", message, suggestion);
}

function parseCreateBody(body: unknown): CreateContextRequestBody {
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    throw Errors.missingFields("label", "storageEngine", "syncStrategy");
  }
  const record = body as Record<string, unknown>;
  if (typeof record.label !== "string" || record.label.trim() === "") {
    throw invalidDataContext("label must be a non-empty string");
  }
  if (typeof record.storageEngine !== "string" || !VALID_STORAGE_ENGINES.has(record.storageEngine as StorageEngine)) {
    throw invalidDataContext('storageEngine must be "leveldb" or "rocksdb"');
  }
  if (typeof record.syncStrategy !== "string" || !VALID_SYNC_STRATEGIES.has(record.syncStrategy as RoleSyncStrategy)) {
    throw invalidDataContext('syncStrategy must be "full", "light", or "fast-sync"');
  }
  const checkpointHeight = record.checkpointHeight;
  if (checkpointHeight !== undefined && (typeof checkpointHeight !== "number" || !Number.isInteger(checkpointHeight) || checkpointHeight <= 0)) {
    throw invalidDataContext("checkpointHeight must be a positive integer");
  }
  if (record.checkpointHash !== undefined && typeof record.checkpointHash !== "string") {
    throw invalidDataContext("checkpointHash must be a string");
  }
  if (record.snapshotId !== undefined && typeof record.snapshotId !== "string") {
    throw invalidDataContext("snapshotId must be a string");
  }

  return {
    label: record.label.trim(),
    storageEngine: record.storageEngine as StorageEngine,
    syncStrategy: record.syncStrategy as RoleSyncStrategy,
    checkpointHeight,
    checkpointHash: record.checkpointHash as string | undefined,
    snapshotId: record.snapshotId as string | undefined,
  };
}

function validateContextId(contextId: string): void {
  if (!/^ctx-[A-Za-z0-9._-]+$/.test(contextId)) {
    throw invalidDataContext("contextId is invalid", "Use a data context id returned by this API.");
  }
}

export function createNodeDataContextsRouter(deps: NodeDataContextsRouterDeps): Router {
  const router = Router({ mergeParams: true });

  router.get("/", (req: Request<{ id: string }>, res: Response) => {
    try {
      requireNode(req.params.id);
      res.json({
        contexts: deps.dataContextManager.listContexts(req.params.id),
        activeContext: deps.dataContextManager.getActiveContext(req.params.id),
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/", async (req: Request<{ id: string }>, res: Response) => {
    try {
      const node = requireStoppedNode(req.params.id);
      const input = parseCreateBody(req.body);
      const isFirstContext = deps.dataContextManager.listContexts(node.id).length === 0;
      let context = deps.dataContextManager.createContext(node.id, input, { active: false });

      if (isFirstContext) {
        const previousStorageEngine = node.settings.storageEngine ?? "leveldb";
        try {
          await deps.nodeManager.ensureStorageEngine(node.id, context.storageEngine);
          context = deps.dataContextManager.activateContext(node.id, context.id);
          await deps.nodeManager.updateNode(node.id, {
            settings: {
              activeDataContextId: context.id,
              syncStrategy: context.syncStrategy,
            },
          });
        } catch (error) {
          deps.dataContextManager.deleteContext(node.id, context.id);
          if (previousStorageEngine !== context.storageEngine) {
            await deps.nodeManager.ensureStorageEngine(node.id, previousStorageEngine).catch(() => undefined);
          }
          throw error;
        }
      }

      res.status(201).json({ context });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/:contextId/activate", async (req: Request<{ id: string; contextId: string }>, res: Response) => {
    try {
      const node = requireStoppedNode(req.params.id);
      validateContextId(req.params.contextId);
      const target = deps.dataContextManager
        .listContexts(node.id)
        .find((context) => context.id === req.params.contextId);
      if (!target) {
        throw invalidDataContext(`Data context ${req.params.contextId} does not exist for node ${node.id}`);
      }
      const previousActiveContext = deps.dataContextManager.getActiveContext(node.id);
      const previousStorageEngine = node.settings.storageEngine ?? "leveldb";
      let context = target;
      let updatedNode = node;
      try {
        await deps.nodeManager.ensureStorageEngine(node.id, target.storageEngine);
        context = deps.dataContextManager.activateContext(node.id, req.params.contextId);
        updatedNode = await deps.nodeManager.updateNode(node.id, {
          settings: {
            activeDataContextId: context.id,
            syncStrategy: context.syncStrategy,
          },
        });
      } catch (error) {
        if (previousActiveContext) {
          try {
            deps.dataContextManager.activateContext(node.id, previousActiveContext.id);
          } catch {
            // Keep the original error; the operator can inspect context history.
          }
        }
        if (previousStorageEngine !== target.storageEngine) {
          await deps.nodeManager.ensureStorageEngine(node.id, previousStorageEngine).catch(() => undefined);
        }
        throw error;
      }

      res.json({ context, node: updatedNode });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;

  function requireNode(nodeId: string) {
    const node = deps.nodeManager.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    return node;
  }

  function requireStoppedNode(nodeId: string) {
    const node = requireNode(nodeId);
    if (node.process.status !== "stopped") {
      throw new ApiError(
        "NODE_NOT_STOPPED",
        `Node must be stopped before changing data contexts; current status is ${node.process.status}`,
        "Stop the node, then retry the data context operation.",
        409,
      );
    }
    return node;
  }
}
