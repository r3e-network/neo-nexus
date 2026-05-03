import { beforeEach, describe, expect, it, vi } from "vitest";

vi.unmock("better-sqlite3");

import Database from "better-sqlite3";
import { ApiError } from "../../src/api/errors";
import { NodeDataContextManager } from "../../src/core/NodeDataContextManager";
import { NodeRoleApplicationService } from "../../src/core/NodeRoleApplicationService";
import { NodeRoleManager } from "../../src/core/NodeRoleManager";
import type { InstalledPlugin, NodeInstance, PluginId, UpdateNodeRequest } from "../../src/types";

function createDatabase() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE node_role_profiles (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      kind TEXT NOT NULL,
      node_types TEXT NOT NULL,
      profile TEXT NOT NULL,
      created_by TEXT,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
    CREATE TABLE node_role_applications (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      role_id TEXT NOT NULL,
      role_name TEXT NOT NULL,
      application_plan TEXT NOT NULL,
      previous_state TEXT,
      applied_at INTEGER NOT NULL,
      applied_by TEXT,
      status TEXT NOT NULL,
      error_message TEXT
    );
    CREATE TABLE node_data_contexts (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      label TEXT NOT NULL,
      storage_engine TEXT NOT NULL,
      sync_strategy TEXT NOT NULL,
      checkpoint_height INTEGER,
      checkpoint_hash TEXT,
      snapshot_id TEXT,
      active INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
    CREATE UNIQUE INDEX idx_node_data_contexts_one_active
      ON node_data_contexts(node_id)
      WHERE active = 1;
  `);
  return db;
}

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

describe("NodeRoleApplicationService", () => {
  let roleManager: NodeRoleManager;
  let dataContextManager: NodeDataContextManager;
  let node: NodeInstance;
  let nodeManager: {
    getNode: ReturnType<typeof vi.fn>;
    ensureStorageEngine: ReturnType<typeof vi.fn>;
    installPlugin: ReturnType<typeof vi.fn>;
    updatePluginConfig: ReturnType<typeof vi.fn>;
    setPluginEnabled: ReturnType<typeof vi.fn>;
    updateNode: ReturnType<typeof vi.fn>;
  };
  let service: NodeRoleApplicationService;

  beforeEach(() => {
    const db = createDatabase();
    roleManager = new NodeRoleManager(db);
    dataContextManager = new NodeDataContextManager(db);
    node = createNode();
    nodeManager = {
      getNode: vi.fn(() => node),
      ensureStorageEngine: vi.fn(async (_nodeId: string, storageEngine: "leveldb" | "rocksdb") => {
        node = { ...node, settings: { ...node.settings, storageEngine } };
        return node;
      }),
      installPlugin: vi.fn(async (_nodeId: string, pluginId: PluginId, config?: Record<string, unknown>) => {
        const plugin: InstalledPlugin = {
          id: pluginId,
          version: node.version,
          config: config ?? {},
          installedAt: Date.now(),
          enabled: true,
        };
        node = { ...node, plugins: [...(node.plugins ?? []), plugin] };
      }),
      updatePluginConfig: vi.fn((_nodeId: string, pluginId: PluginId, config?: Record<string, unknown>) => {
        node = {
          ...node,
          plugins: (node.plugins ?? []).map((plugin) =>
            plugin.id === pluginId ? { ...plugin, config: config ?? {} } : plugin),
        };
      }),
      setPluginEnabled: vi.fn(async (_nodeId: string, pluginId: PluginId, enabled: boolean) => {
        node = {
          ...node,
          plugins: (node.plugins ?? []).map((plugin) => plugin.id === pluginId ? { ...plugin, enabled } : plugin),
        };
      }),
      updateNode: vi.fn(async (_nodeId: string, request: UpdateNodeRequest) => {
        node = { ...node, settings: { ...node.settings, ...(request.settings ?? {}) } };
        return node;
      }),
    };
    service = new NodeRoleApplicationService({ roleManager, dataContextManager, nodeManager: nodeManager as never });
  });

  it("plans with the active data context", () => {
    const active = dataContextManager.createContext("node-1", {
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
    });
    node = createNode({
      settings: {
        activeDataContextId: active.id,
        relay: true,
        maxConnections: 80,
        minPeers: 10,
        maxPeers: 60,
      },
      plugins: [
        { id: "StateService", version: "3.9.2", config: { AutoStart: true, FullState: false }, installedAt: 1, enabled: true },
        { id: "RpcServer", version: "3.9.2", config: {}, installedAt: 1, enabled: true },
      ],
    });

    const plan = service.plan("builtin-state", "node-1", {});

    expect(plan.changes).toEqual([]);
    expect(plan.requiresRestart).toBe(false);
  });

  it("applies the built-in state role by creating context, switching storage, plugins, and role marker", async () => {
    const result = await service.apply("builtin-state", "node-1", {}, "admin-1");

    expect(result.application.status).toBe("applied");
    expect(result.application.appliedBy).toBe("admin-1");
    expect(result.node.settings.storageEngine).toBe("rocksdb");
    expect(result.node.settings.syncStrategy).toBe("fast-sync");
    expect(result.node.settings.activeDataContextId).toMatch(/^ctx-/);
    expect(result.node.settings.role).toMatchObject({ id: "builtin-state", name: "State Node" });
    expect(dataContextManager.getActiveContext("node-1")).toMatchObject({
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
      active: true,
    });
    expect(nodeManager.ensureStorageEngine).toHaveBeenCalledWith("node-1", "rocksdb");
    expect(nodeManager.installPlugin).toHaveBeenCalledWith("node-1", "StateService", { AutoStart: true, FullState: false });
    expect(nodeManager.installPlugin).toHaveBeenCalledWith("node-1", "RpcServer", {});
    expect(nodeManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "StateService", true);
    expect(nodeManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "RpcServer", true);
    expect(roleManager.listApplications("node-1")).toHaveLength(1);
    expect(roleManager.listApplications("node-1")[0]?.status).toBe("applied");
  });

  it("rejects running nodes before mutating anything", async () => {
    node = createNode({ process: { status: "running", pid: 123 } });

    await expect(service.apply("builtin-state", "node-1", {})).rejects.toMatchObject({
      code: "NODE_NOT_STOPPED",
    });

    expect(dataContextManager.listContexts("node-1")).toEqual([]);
    expect(nodeManager.ensureStorageEngine).not.toHaveBeenCalled();
    expect(nodeManager.installPlugin).not.toHaveBeenCalled();
    expect(nodeManager.updateNode).not.toHaveBeenCalled();
    expect(roleManager.listApplications("node-1")).toEqual([]);
  });

  it.each(["starting", "stopping", "syncing", "error"] as const)("rejects %s nodes before mutating anything", async (status) => {
    node = createNode({ process: { status } });

    await expect(service.apply("builtin-state", "node-1", {})).rejects.toMatchObject({
      code: "NODE_NOT_STOPPED",
    });

    expect(dataContextManager.listContexts("node-1")).toEqual([]);
    expect(nodeManager.ensureStorageEngine).not.toHaveBeenCalled();
    expect(nodeManager.installPlugin).not.toHaveBeenCalled();
    expect(nodeManager.updateNode).not.toHaveBeenCalled();
    expect(roleManager.listApplications("node-1")).toEqual([]);
  });

  it("preflights secure signer roles before plugin or data context mutations", async () => {
    await expect(service.apply("builtin-secure-signer-client", "node-1", {}, "admin-1")).rejects.toMatchObject({
      code: "NODE_ROLE_PREREQUISITE_MISSING",
    });

    expect(dataContextManager.listContexts("node-1")).toEqual([]);
    expect(nodeManager.installPlugin).not.toHaveBeenCalled();
    expect(nodeManager.setPluginEnabled).not.toHaveBeenCalled();
    expect(nodeManager.updateNode).not.toHaveBeenCalled();
    expect(roleManager.listApplications("node-1")).toEqual([]);
  });

  it("applies secure signer client roles when the node already has a signer profile", async () => {
    node = createNode({
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-prod",
          signerName: "Production signer",
        },
      },
    });

    const result = await service.apply("builtin-secure-signer-client", "node-1", {}, "admin-1");

    expect(result.application.status).toBe("applied");
    expect(result.node.settings.keyProtection).toMatchObject({
      mode: "secure-signer",
      signerProfileId: "signer-prod",
      signerName: "Production signer",
    });
    expect(nodeManager.installPlugin).toHaveBeenCalledWith("node-1", "SignClient", {});
    expect(nodeManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "SignClient", true);
  });

  it("records a failed application when plugin installation fails", async () => {
    nodeManager.installPlugin.mockRejectedValueOnce(new ApiError("PLUGIN_INSTALL_FAILED", "plugin install failed", "Try again."));

    await expect(service.apply("builtin-state", "node-1", {}, "admin-1")).rejects.toMatchObject({
      code: "PLUGIN_INSTALL_FAILED",
    });

    const applications = roleManager.listApplications("node-1");
    expect(applications).toHaveLength(1);
    expect(applications[0]).toMatchObject({
      status: "failed",
      roleId: "builtin-state",
      appliedBy: "admin-1",
      errorMessage: "plugin install failed",
    });
    expect(applications[0]?.previousState).toMatchObject({
      resultingState: expect.any(Object),
    });
  });

  it("rejects plugin-bearing roles for neo-go nodes instead of silently applying them", async () => {
    const role = roleManager.createCustomRole({
      name: "Bad Neo Go Plugin Role",
      nodeTypes: ["neo-go"],
      profile: {
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    });
    node = createNode({ type: "neo-go", plugins: [] });

    expect(() => service.plan(role.id, "node-1", {})).toThrow(/plugin changes/i);
    await expect(service.apply(role.id, "node-1", {})).rejects.toMatchObject({
      code: "NODE_ROLE_PLUGIN_UNSUPPORTED",
    });
  });
});
