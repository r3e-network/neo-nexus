import { beforeEach, describe, expect, it, vi } from "vitest";
import { NodeManager } from "../../src/core/NodeManager";
import { StorageManager } from "../../src/core/StorageManager";
import { paths } from "../../src/utils/paths";

describe("NodeManager system actions", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("stops only the nodes that are currently running", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", process: { status: "running" } },
      { id: "node-2", process: { status: "stopped" } },
      { id: "node-3", process: { status: "running" } },
    ]);
    manager.stopNode = vi.fn().mockResolvedValue(undefined);

    const result = await manager.stopAllNodes();

    expect(result).toEqual({
      stoppedCount: 2,
      alreadyStoppedCount: 1,
    });
    expect(manager.stopNode).toHaveBeenCalledTimes(2);
    expect(manager.stopNode).toHaveBeenCalledWith("node-1");
    expect(manager.stopNode).toHaveBeenCalledWith("node-3");
  });

  it("aggregates cleaned log files across all nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", paths: { logs: "/tmp/node-1/logs" } },
      { id: "node-2", paths: { logs: "/tmp/node-2/logs" } },
    ]);

    vi.spyOn(StorageManager, "cleanOldLogs")
      .mockResolvedValueOnce(5)
      .mockResolvedValueOnce(0);

    const result = await manager.cleanOldLogs(14);

    expect(result).toEqual({
      cleanedFiles: 5,
      nodesAffected: 1,
      maxAgeDays: 14,
    });
  });

  it("exports a configuration snapshot without process metrics noise", () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      {
        id: "node-1",
        name: "Mainnet Node",
        type: "neo-cli",
        network: "mainnet",
        version: "3.9.2",
        syncMode: "full",
        ports: { rpc: 10332, p2p: 10333 },
        paths: {
          base: "/nodes/node-1",
          data: "/nodes/node-1/data",
          logs: "/nodes/node-1/logs",
          config: "/nodes/node-1/config",
        },
        settings: { Example: true },
        createdAt: 1,
        updatedAt: 2,
        process: { status: "running", pid: 42 },
        metrics: { blockHeight: 1000 },
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    ]);

    const snapshot = manager.exportConfiguration();

    expect(snapshot.version).toBe("2.0.0");
    expect(snapshot.nodes).toEqual([
      {
        id: "node-1",
        name: "Mainnet Node",
        type: "neo-cli",
        network: "mainnet",
        version: "3.9.2",
        syncMode: "full",
        ports: { rpc: 10332, p2p: 10333 },
        paths: {
          base: "/nodes/node-1",
          data: "/nodes/node-1/data",
          logs: "/nodes/node-1/logs",
          config: "/nodes/node-1/config",
        },
        settings: { Example: true },
        createdAt: 1,
        updatedAt: 2,
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    ]);
    expect(snapshot.nodes[0]).not.toHaveProperty("process");
    expect(snapshot.nodes[0]).not.toHaveProperty("metrics");
  });

  it("resets node data by stopping running nodes and deleting each node", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopAllNodes: ReturnType<typeof vi.fn>;
      deleteNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", paths: { base: `${paths.nodes}/node-1` } },
      { id: "node-2", paths: { base: `${paths.nodes}/node-2` } },
    ]);
    manager.stopAllNodes = vi.fn().mockResolvedValue({
      stoppedCount: 1,
      alreadyStoppedCount: 1,
    });
    manager.deleteNode = vi.fn().mockResolvedValue(undefined);

    const result = await manager.resetAllNodeData();

    expect(manager.stopAllNodes).toHaveBeenCalledOnce();
    expect(manager.deleteNode).toHaveBeenCalledTimes(2);
    expect(result).toEqual({
      deletedNodeCount: 2,
      removedDirectoryCount: 2,
      stoppedCount: 1,
      alreadyStoppedCount: 1,
    });
  });

  it("restores nodes from a snapshot and reinstalls exported plugins", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      createNode: ReturnType<typeof vi.fn>;
      installPlugin: ReturnType<typeof vi.fn>;
      resetAllNodeData: ReturnType<typeof vi.fn>;
    };

    manager.createNode = vi
      .fn()
      .mockResolvedValueOnce({ id: "new-node-1", type: "neo-cli" })
      .mockResolvedValueOnce({ id: "new-node-2", type: "neo-go" });
    manager.installPlugin = vi.fn().mockResolvedValue(undefined);
    manager.resetAllNodeData = vi.fn().mockResolvedValue(undefined);

    const result = await manager.restoreConfiguration(
      {
        version: "2.0.0",
        nodes: [
          {
            name: "Node A",
            type: "neo-cli",
            network: "mainnet",
            syncMode: "full",
            version: "3.9.2",
            ports: { rpc: 10332, p2p: 10333 },
            settings: { debugMode: true },
            plugins: [{ id: "RpcServer", config: { Port: 10332 } }],
          },
          {
            name: "Node B",
            type: "neo-go",
            network: "testnet",
            syncMode: "light",
            version: "0.106.0",
            ports: { rpc: 20332, p2p: 20333 },
            settings: { relay: false },
            plugins: [],
          },
        ],
      },
      { replaceExisting: true },
    );

    expect(manager.resetAllNodeData).toHaveBeenCalledOnce();
    expect(manager.createNode).toHaveBeenCalledTimes(2);
    expect(manager.createNode).toHaveBeenNthCalledWith(1, {
      name: "Node A",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      version: "3.9.2",
      customPorts: { rpc: 10332, p2p: 10333 },
      settings: { debugMode: true },
    });
    expect(manager.installPlugin).toHaveBeenCalledWith("new-node-1", "RpcServer", { Port: 10332 });
    expect(result).toEqual({
      restoredCount: 2,
      skippedCount: 0,
      failedCount: 0,
    });
  });
});
