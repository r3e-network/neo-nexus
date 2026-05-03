import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { Errors } from "../../src/api/errors";
import { createNodeDataContextsRouter } from "../../src/api/routes/nodeDataContexts";
import type { NodeDataContext, NodeInstance, UpdateNodeRequest } from "../../src/types";

function createNode(overrides: Partial<NodeInstance> = {}): NodeInstance {
  return {
    id: "node-1",
    name: "Node 1",
    chain: "n3",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.9.2",
    ports: { rpc: 10332, p2p: 10333 },
    paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
    settings: {},
    createdAt: 1,
    updatedAt: 1,
    process: { status: "stopped" },
    plugins: [],
    ...overrides,
  };
}

function createContext(overrides: Partial<NodeDataContext> = {}): NodeDataContext {
  return {
    id: "ctx-1",
    nodeId: "node-1",
    label: "default",
    storageEngine: "leveldb",
    syncStrategy: "full",
    active: true,
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

describe("Actual node data contexts router", () => {
  let app: express.Application;
  let node: NodeInstance | null;
  let contexts: NodeDataContext[];
  let dataContextManager: {
    listContexts: ReturnType<typeof vi.fn>;
    getActiveContext: ReturnType<typeof vi.fn>;
    createContext: ReturnType<typeof vi.fn>;
    activateContext: ReturnType<typeof vi.fn>;
    deleteContext: ReturnType<typeof vi.fn>;
  };
  let nodeManager: {
    getNode: ReturnType<typeof vi.fn>;
    updateNode: ReturnType<typeof vi.fn>;
    ensureStorageEngine: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());
    node = createNode();
    contexts = [createContext()];
    dataContextManager = {
      listContexts: vi.fn(() => contexts),
      getActiveContext: vi.fn(() => contexts.find((context) => context.active) ?? null),
      createContext: vi.fn((nodeId, input) => {
        const context = createContext({
          id: `ctx-${contexts.length + 1}`,
          nodeId,
          ...input,
          active: contexts.length === 0,
        });
        contexts = context.active
          ? [...contexts.map((candidate) => ({ ...candidate, active: false })), context]
          : [...contexts, context];
        return context;
      }),
      activateContext: vi.fn((nodeId: string, contextId: string) => {
        const context = contexts.find((candidate) => candidate.nodeId === nodeId && candidate.id === contextId);
        if (!context) throw new Error(`Data context ${contextId} not found for node ${nodeId}`);
        contexts = contexts.map((candidate) => ({ ...candidate, active: candidate.id === contextId }));
        return { ...context, active: true };
      }),
      deleteContext: vi.fn((nodeId: string, contextId: string) => {
        contexts = contexts.filter((candidate) => !(candidate.nodeId === nodeId && candidate.id === contextId));
      }),
    };
    nodeManager = {
      getNode: vi.fn(() => node),
      updateNode: vi.fn(async (_nodeId: string, request: UpdateNodeRequest) => {
        node = node ? { ...node, settings: { ...node.settings, ...(request.settings ?? {}) } } : null;
        return node;
      }),
      ensureStorageEngine: vi.fn(async (_nodeId: string, storageEngine: "leveldb" | "rocksdb") => {
        node = node ? { ...node, settings: { ...node.settings, storageEngine } } : null;
        return node;
      }),
    };

    app.use("/api/nodes/:id/data-contexts", createNodeDataContextsRouter({
      nodeManager: nodeManager as never,
      dataContextManager: dataContextManager as never,
    }));
  });

  it("lists contexts with the active context", async () => {
    const response = await request(app).get("/api/nodes/node-1/data-contexts");

    expect(response.status).toBe(200);
    expect(response.body.contexts).toHaveLength(1);
    expect(response.body.activeContext.id).toBe("ctx-1");
  });

  it("creates a context", async () => {
    contexts = [];
    const response = await request(app).post("/api/nodes/node-1/data-contexts").send({
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
      checkpointHeight: 100,
      checkpointHash: "0xabc",
      snapshotId: "snapshot-1",
    });

    expect(response.status).toBe(201);
    expect(response.body.context).toMatchObject({
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
    });
    expect(nodeManager.ensureStorageEngine).toHaveBeenCalledWith("node-1", "rocksdb");
    expect(nodeManager.updateNode).toHaveBeenCalledWith("node-1", {
      settings: {
        activeDataContextId: response.body.context.id,
        syncStrategy: "fast-sync",
      },
    });
  });

  it("does not switch node settings when creating a non-first context", async () => {
    const response = await request(app).post("/api/nodes/node-1/data-contexts").send({
      label: "archive",
      storageEngine: "leveldb",
      syncStrategy: "full",
    });

    expect(response.status).toBe(201);
    expect(nodeManager.ensureStorageEngine).not.toHaveBeenCalled();
    expect(nodeManager.updateNode).not.toHaveBeenCalled();
  });

  it("activates a context and updates node settings and storage", async () => {
    contexts = [
      createContext(),
      createContext({ id: "ctx-2", label: "state", storageEngine: "rocksdb", syncStrategy: "fast-sync", active: false }),
    ];

    const response = await request(app).post("/api/nodes/node-1/data-contexts/ctx-2/activate").send({});

    expect(response.status).toBe(200);
    expect(response.body.context.id).toBe("ctx-2");
    expect(response.body.node.settings.storageEngine).toBe("rocksdb");
    expect(nodeManager.ensureStorageEngine).toHaveBeenCalledWith("node-1", "rocksdb");
    expect(nodeManager.updateNode).toHaveBeenCalledWith("node-1", {
      settings: {
        activeDataContextId: "ctx-2",
        syncStrategy: "fast-sync",
      },
    });
  });

  it("returns structured 400 for invalid create input", async () => {
    const response = await request(app).post("/api/nodes/node-1/data-contexts").send({
      label: "",
      storageEngine: "bad",
      syncStrategy: "fast-sync",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("DATA_CONTEXT_INVALID");
    expect(dataContextManager.createContext).not.toHaveBeenCalled();
  });

  it("returns node-not-found for a missing node", async () => {
    node = null;

    const response = await request(app).get("/api/nodes/missing/data-contexts");

    expect(response.status).toBe(404);
    expect(response.body.code).toBe("NODE_NOT_FOUND");
    expect(dataContextManager.listContexts).not.toHaveBeenCalled();
  });

  it("rejects create and activate while the node is running", async () => {
    node = createNode({ process: { status: "running", pid: 123 } });

    const createResponse = await request(app).post("/api/nodes/node-1/data-contexts").send({
      label: "archive",
      storageEngine: "leveldb",
      syncStrategy: "full",
    });
    const activateResponse = await request(app).post("/api/nodes/node-1/data-contexts/ctx-1/activate").send({});

    expect(createResponse.status).toBe(409);
    expect(createResponse.body.code).toBe("NODE_NOT_STOPPED");
    expect(activateResponse.status).toBe(409);
    expect(activateResponse.body.code).toBe("NODE_NOT_STOPPED");
    expect(dataContextManager.createContext).not.toHaveBeenCalled();
    expect(dataContextManager.activateContext).not.toHaveBeenCalled();
  });

  it("passes ApiError storage failures through", async () => {
    contexts = [];
    nodeManager.ensureStorageEngine.mockRejectedValueOnce(Errors.nodeRunning());

    const response = await request(app).post("/api/nodes/node-1/data-contexts").send({
      label: "default",
      storageEngine: "leveldb",
      syncStrategy: "full",
    });

    expect(response.status).toBe(409);
    expect(response.body.code).toBe("NODE_RUNNING");
    expect(dataContextManager.deleteContext).toHaveBeenCalledWith("node-1", "ctx-1");
    expect(contexts).toEqual([]);
  });

  it("rolls back activation when node settings update fails", async () => {
    contexts = [
      createContext({ id: "ctx-1", active: true, storageEngine: "leveldb", syncStrategy: "full" }),
      createContext({ id: "ctx-2", active: false, storageEngine: "rocksdb", syncStrategy: "fast-sync" }),
    ];
    nodeManager.updateNode.mockRejectedValueOnce(Errors.nodeRunning());

    const response = await request(app).post("/api/nodes/node-1/data-contexts/ctx-2/activate").send({});

    expect(response.status).toBe(409);
    expect(response.body.code).toBe("NODE_RUNNING");
    expect(contexts.find((context) => context.id === "ctx-1")?.active).toBe(true);
    expect(contexts.find((context) => context.id === "ctx-2")?.active).toBe(false);
  });
});
