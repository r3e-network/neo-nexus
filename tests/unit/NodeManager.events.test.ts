import { EventEmitter } from "node:events";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { LogEntry, NodeMetrics, NodeStatus } from "../../src/types";

const hoisted = vi.hoisted(() => ({
  NeoCliNode: vi.fn(),
  NeoGoNode: vi.fn(),
}));

vi.mock("../../src/nodes/NeoCliNode", () => ({
  NeoCliNode: hoisted.NeoCliNode,
}));

vi.mock("../../src/nodes/NeoGoNode", () => ({
  NeoGoNode: hoisted.NeoGoNode,
}));

class FakeRuntimeNode extends EventEmitter {
  private running = true;

  async start(): Promise<void> {}

  async stop(): Promise<void> {
    this.running = false;
  }

  async restart(): Promise<void> {
    this.running = true;
  }

  isRunning(): boolean {
    return this.running;
  }

  getStatus() {
    return { pid: 12345, status: "running" as NodeStatus };
  }

  async getBlockHeight(): Promise<number> {
    return 100;
  }

  async getPeersCount(): Promise<number> {
    return 8;
  }

  async getResourceUsage(): Promise<{ cpu: number; memory: number }> {
    return { cpu: 12.5, memory: 256 };
  }

  destroy(): void {}
}

function createMockDb() {
  return {
    prepare: vi.fn(() => ({
      all: vi.fn(() => []),
      get: vi.fn(() => undefined),
      run: vi.fn(),
    })),
  };
}

describe("NodeManager realtime events", () => {
  beforeEach(() => {
    vi.resetModules();
    hoisted.NeoCliNode.mockReset();
    hoisted.NeoGoNode.mockReset();
  });

  it("re-emits node status and log events from the runtime node", async () => {
    const runtimeNode = new FakeRuntimeNode();
    hoisted.NeoCliNode.mockImplementation(() => runtimeNode);

    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never);
    vi.spyOn(manager, "getNode").mockReturnValue({
      id: "node-1",
      name: "Node 1",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      version: "3.9.2",
      ports: { rpc: 10332, p2p: 10333 },
      paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config.json" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/tmp/node-1", importedAt: 1 } },
      createdAt: 1,
      updatedAt: 1,
      process: { status: "stopped" },
      metrics: undefined,
      plugins: [],
    });

    const statuses: Array<{ nodeId: string; status: NodeStatus; previousStatus: NodeStatus }> = [];
    const logs: Array<{ nodeId: string; entry: LogEntry }> = [];

    manager.on("nodeStatus", (event) => statuses.push(event));
    manager.on("nodeLog", (event) => logs.push(event));

    await manager.startNode("node-1");

    runtimeNode.emit("status", "running", "starting");
    runtimeNode.emit("log", {
      timestamp: 123,
      level: "info",
      source: "neo-cli",
      message: "persisted block",
    } satisfies LogEntry);

    expect(statuses).toContainEqual({
      nodeId: "node-1",
      status: "running",
      previousStatus: "starting",
    });
    expect(logs).toContainEqual({
      nodeId: "node-1",
      entry: {
        timestamp: 123,
        level: "info",
        source: "neo-cli",
        message: "persisted block",
      },
    });
  });

  it("emits node metrics after a metrics refresh", async () => {
    const runtimeNode = new FakeRuntimeNode();
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as any;
    manager.nodes.set("node-1", runtimeNode);

    const metricsEvents: Array<{ nodeId: string; metrics: NodeMetrics }> = [];
    manager.on("nodeMetrics", (event: { nodeId: string; metrics: NodeMetrics }) => metricsEvents.push(event));

    await manager.updateMetrics("node-1");

    expect(metricsEvents).toHaveLength(1);
    expect(metricsEvents[0].nodeId).toBe("node-1");
    expect(metricsEvents[0].metrics.blockHeight).toBe(100);
    expect(metricsEvents[0].metrics.connectedPeers).toBe(8);
  });
});
