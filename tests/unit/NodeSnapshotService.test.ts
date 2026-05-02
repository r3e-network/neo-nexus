import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";

import { NodeSnapshotService, type NodeSnapshotServiceDependencies } from "../../src/core/NodeSnapshotService";
import type { NodeInstance } from "../../src/types/index";

const pkg = JSON.parse(
  readFileSync(new URL("../../package.json", import.meta.url), "utf-8"),
) as { version: string };

function makeNode(overrides: Partial<NodeInstance> = {}): NodeInstance {
  return {
    id: "node-1",
    name: "Existing",
    chain: "n3",
    type: "neo-cli",
    network: "testnet",
    syncMode: "full",
    version: "3.9.2",
    ports: { rpc: 20332, p2p: 20333, websocket: 20334, metrics: 20335 },
    paths: {
      base: "/nodes/node-1",
      data: "/nodes/node-1/data",
      logs: "/nodes/node-1/logs",
      config: "/nodes/node-1/config",
    },
    settings: { debugMode: true },
    createdAt: 1,
    updatedAt: 2,
    process: { status: "running", pid: 1234 },
    metrics: {
      blockHeight: 10,
      headerHeight: 10,
      connectedPeers: 2,
      unconnectedPeers: 0,
      syncProgress: 100,
      memoryUsage: 128,
      cpuUsage: 1,
      lastUpdate: 3,
    },
    plugins: [{ id: "RpcServer", version: "3.9.2", config: { Port: 20332 }, installedAt: 3, enabled: true }],
    ...overrides,
  };
}

function makeDependencies(overrides: Partial<NodeSnapshotServiceDependencies> = {}): NodeSnapshotServiceDependencies {
  return {
    getAllNodes: vi.fn(() => []),
    createNode: vi.fn(async () => makeNode({ id: "restored-node" })),
    installPlugin: vi.fn(async () => undefined),
    setPluginEnabled: vi.fn(async () => undefined),
    resetAllNodeData: vi.fn(async () => undefined),
    ...overrides,
  };
}

describe("NodeSnapshotService", () => {
  it("exports configuration snapshots without process or runtime metrics", () => {
    const deps = makeDependencies({
      getAllNodes: vi.fn(() => [makeNode()]),
    });
    const service = new NodeSnapshotService(deps);

    const snapshot = service.exportConfiguration();

    expect(snapshot).toMatchObject({
      generatedAt: expect.any(Number),
      version: pkg.version,
      nodes: [
        {
          id: "node-1",
          name: "Existing",
          type: "neo-cli",
          network: "testnet",
          settings: { debugMode: true },
          plugins: [{ id: "RpcServer", version: "3.9.2", config: { Port: 20332 }, installedAt: 3, enabled: true }],
        },
      ],
    });
    expect(snapshot.nodes[0]).not.toHaveProperty("process");
    expect(snapshot.nodes[0]).not.toHaveProperty("metrics");
  });

  it("restores valid snapshot nodes, skips invalid entries, and strips reserved import settings", async () => {
    const deps = makeDependencies({
      createNode: vi
        .fn()
        .mockResolvedValueOnce(makeNode({ id: "restored-a" }))
        .mockResolvedValueOnce(makeNode({ id: "restored-b" })),
    });
    const service = new NodeSnapshotService(deps);

    const result = await service.restoreConfiguration({
      version: "2.0.0",
      nodes: [
        {
          name: "Restored A",
          type: "neo-cli",
          network: "testnet",
          syncMode: "full",
          version: "3.9.2",
          ports: { rpc: 20332, p2p: 20333 },
          settings: {
            debugMode: true,
            import: { imported: true, ownershipMode: "observe-only", existingPath: "/opt/neo", importedAt: 1 },
          },
          plugins: [{ id: "RpcServer", config: { Port: 20332 }, enabled: false }],
        },
        { name: "Missing network", type: "neo-go" } as never,
        {
          name: "Restored B",
          type: "neo-go",
          network: "mainnet",
          syncMode: "light",
          version: "0.106.0",
          ports: { rpc: 10332, p2p: 10333 },
          settings: { relay: false },
        },
      ],
    });

    expect(result).toEqual({ restoredCount: 2, skippedCount: 1, failedCount: 0 });
    expect(deps.createNode).toHaveBeenNthCalledWith(1, {
      name: "Restored A",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.9.2",
      customPorts: { rpc: 20332, p2p: 20333 },
      settings: { debugMode: true },
    });
    expect(deps.createNode).toHaveBeenNthCalledWith(2, {
      name: "Restored B",
      type: "neo-go",
      network: "mainnet",
      syncMode: "light",
      version: "0.106.0",
      customPorts: { rpc: 10332, p2p: 10333 },
      settings: { relay: false },
    });
    expect(deps.installPlugin).toHaveBeenCalledWith("restored-a", "RpcServer", { Port: 20332 });
    expect(deps.setPluginEnabled).toHaveBeenCalledWith("restored-a", "RpcServer", false);
  });

  it("validates replace-existing snapshots before deleting current data", async () => {
    const deps = makeDependencies();
    const service = new NodeSnapshotService(deps);

    await expect(service.restoreConfiguration({
      version: "2.0.0",
      nodes: [
        { name: "Valid", type: "neo-cli", network: "testnet" },
        { name: "Missing network", type: "neo-go" } as never,
      ],
    }, { replaceExisting: true })).rejects.toMatchObject({ code: "SNAPSHOT_INVALID" });

    expect(deps.resetAllNodeData).not.toHaveBeenCalled();
    expect(deps.createNode).not.toHaveBeenCalled();
  });

  it("rolls back replace-existing restores when fail-fast creation fails", async () => {
    const deps = makeDependencies({
      getAllNodes: vi.fn(() => [makeNode({ name: "Rollback Node" })]),
      createNode: vi
        .fn()
        .mockRejectedValueOnce(new Error("port already in use"))
        .mockResolvedValueOnce(makeNode({ id: "rollback-node" })),
    });
    const service = new NodeSnapshotService(deps);

    await expect(service.restoreConfiguration({
      version: "2.0.0",
      nodes: [{ name: "Broken Restore", type: "neo-cli", network: "testnet" }],
    }, { replaceExisting: true })).rejects.toMatchObject({
      code: "SNAPSHOT_RESTORE_FAILED",
      message: expect.stringContaining("port already in use"),
    });

    expect(deps.resetAllNodeData).toHaveBeenCalledTimes(2);
    expect(deps.createNode).toHaveBeenNthCalledWith(1, expect.objectContaining({ name: "Broken Restore" }));
    expect(deps.createNode).toHaveBeenNthCalledWith(2, expect.objectContaining({ name: "Rollback Node" }));
  });

  it("awaits disabled plugin writes before reporting restore completion", async () => {
    let disabledWriteCompleted = false;
    const deps = makeDependencies({
      createNode: vi.fn(async () => makeNode({ id: "restored-a" })),
      setPluginEnabled: vi.fn(async () => {
        await new Promise((resolve) => setTimeout(resolve, 10));
        disabledWriteCompleted = true;
      }),
    });
    const service = new NodeSnapshotService(deps);

    await service.restoreConfiguration({
      version: "2.0.0",
      nodes: [{
        name: "Restored A",
        type: "neo-cli",
        network: "testnet",
        plugins: [{ id: "RpcServer", enabled: false }],
      }],
    });

    expect(disabledWriteCompleted).toBe(true);
  });
});
