import { beforeEach, describe, expect, it, vi } from "vitest";
import type { NodeConfig } from "../../src/types";

const hoisted = vi.hoisted(() => ({
  NeofuraNode: vi.fn(),
}));

vi.mock("../../src/nodes/NeofuraNode", () => ({
  NeofuraNode: hoisted.NeofuraNode,
}));

class FakeNeofura {
  public events = new Map<string, ((...args: unknown[]) => void)[]>();
  public started = false;
  on(event: string, listener: (...args: unknown[]) => void): this {
    const list = this.events.get(event) ?? [];
    list.push(listener);
    this.events.set(event, list);
    return this;
  }
  async start(): Promise<void> {
    this.started = true;
  }
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

function makeNode(overrides: Partial<NodeConfig> = {}): NodeConfig {
  return {
    id: "node-side-1",
    name: "indexer mainnet",
    chain: "n3",
    type: "neofura",
    network: "mainnet",
    syncMode: "full",
    version: "external",
    ports: { rpc: 10342, p2p: 10343 },
    paths: { base: "/tmp/x", data: "/tmp/x/data", logs: "/tmp/x/logs", config: "/tmp/x/config" },
    settings: { customConfig: { endpoint: "https://api.example.dev/mainnet" } },
    process: { status: "running" },
    metrics: undefined,
    plugins: [],
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

describe("NodeManager.resumeSidecarNodes", () => {
  beforeEach(() => {
    vi.resetModules();
    hoisted.NeofuraNode.mockReset();
  });

  it("re-instantiates and starts neofura sidecars whose persisted status was running", async () => {
    const fake = new FakeNeofura();
    hoisted.NeofuraNode.mockImplementation(() => fake);

    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      getAllNodes: () => NodeConfig[];
      nodes: Map<string, FakeNeofura>;
      resumeSidecarNodes: () => Promise<void>;
    };
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ id: "node-running", process: { status: "running" } }),
    ]);

    await manager.resumeSidecarNodes();

    expect(hoisted.NeofuraNode).toHaveBeenCalledTimes(1);
    expect(fake.started).toBe(true);
    expect(manager.nodes.has("node-running")).toBe(true);
  });

  it("skips sidecars whose status is stopped", async () => {
    hoisted.NeofuraNode.mockImplementation(() => new FakeNeofura());
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      nodes: Map<string, FakeNeofura>;
      resumeSidecarNodes: () => Promise<void>;
    };
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ process: { status: "stopped" } }),
    ]);

    await manager.resumeSidecarNodes();

    expect(hoisted.NeofuraNode).not.toHaveBeenCalled();
    expect(manager.nodes.size).toBe(0);
  });

  it("skips non-sidecar node types", async () => {
    hoisted.NeofuraNode.mockImplementation(() => new FakeNeofura());
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      nodes: Map<string, FakeNeofura>;
      resumeSidecarNodes: () => Promise<void>;
    };
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ type: "neo-cli", process: { status: "running" } }),
    ]);

    await manager.resumeSidecarNodes();

    expect(hoisted.NeofuraNode).not.toHaveBeenCalled();
    expect(manager.nodes.size).toBe(0);
  });

  it("logs (does not throw) if a sidecar fails to start", async () => {
    hoisted.NeofuraNode.mockImplementation(() => {
      const f = new FakeNeofura();
      f.start = async () => { throw new Error("boom"); };
      return f;
    });
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      nodes: Map<string, FakeNeofura>;
      resumeSidecarNodes: () => Promise<void>;
    };
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ process: { status: "running" } }),
    ]);
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

    await expect(manager.resumeSidecarNodes()).resolves.toBeUndefined();
    expect(warnSpy).toHaveBeenCalled();
    expect(manager.nodes.size).toBe(0);
    warnSpy.mockRestore();
  });

  it("stopAllNodes leaves sidecar status untouched so resume can find it next boot", async () => {
    hoisted.NeofuraNode.mockImplementation(() => new FakeNeofura());
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      stopAllNodes: () => Promise<{ stoppedCount: number; alreadyStoppedCount: number }>;
      stopNode: (id: string) => Promise<void>;
    };
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ id: "side", type: "neofura", process: { status: "running" } }),
    ]);
    const stopSpy = vi.spyOn(manager, "stopNode").mockResolvedValue();

    const result = await manager.stopAllNodes();

    expect(stopSpy).not.toHaveBeenCalled();
    expect(result.stoppedCount).toBe(0);
    expect(result.alreadyStoppedCount).toBe(1);
  });

  it("does not double-instantiate a sidecar already in the map", async () => {
    const existing = new FakeNeofura();
    hoisted.NeofuraNode.mockImplementation(() => new FakeNeofura());
    const { NodeManager } = await import("../../src/core/NodeManager");
    const manager = new NodeManager(createMockDb() as never) as unknown as {
      nodes: Map<string, FakeNeofura>;
      resumeSidecarNodes: () => Promise<void>;
    };
    manager.nodes.set("node-side-1", existing);
    vi.spyOn(manager as unknown as { getAllNodes: () => NodeConfig[] }, "getAllNodes").mockReturnValue([
      makeNode({ process: { status: "running" } }),
    ]);

    await manager.resumeSidecarNodes();

    expect(hoisted.NeofuraNode).not.toHaveBeenCalled();
    expect(manager.nodes.get("node-side-1")).toBe(existing);
  });
});
