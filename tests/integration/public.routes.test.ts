import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createPublicRouter } from "../../src/api/routes/public";
import type { NodeInstance, NodeMetrics, SystemMetrics } from "../../src/types";

type MockNodeManager = {
  getAllNodes: ReturnType<typeof vi.fn<() => NodeInstance[]>>;
  getNode: ReturnType<typeof vi.fn<(id: string) => NodeInstance | null>>;
};

type MockMetricsCollector = {
  collectSystemMetrics: ReturnType<typeof vi.fn<() => Promise<SystemMetrics>>>;
};

const baseMetrics: NodeMetrics = {
  blockHeight: 5_000_000,
  headerHeight: 5_000_000,
  connectedPeers: 10,
  unconnectedPeers: 2,
  syncProgress: 100,
  cpuUsage: 25.5,
  memoryUsage: 512,
  lastUpdate: 1_700_000_000_000,
};

function createNode(overrides: Partial<NodeInstance> = {}): NodeInstance {
  return {
    id: "node-1",
    name: "Public Node",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.9.2",
    ports: { rpc: 10332, p2p: 10333 },
    paths: {
      base: "/secret/node",
      data: "/secret/node/data",
      logs: "/secret/node/logs",
      config: "/secret/node/config.json",
      wallet: "/secret/node/wallet.json",
    },
    settings: {
      customConfig: { RpcPassword: "secret" },
      keyProtection: { mode: "secure-signer", signerProfileId: "signer-1" },
    },
    createdAt: 1_700_000_000_000,
    updatedAt: 1_700_000_000_001,
    process: { status: "running", uptime: 3600 },
    metrics: baseMetrics,
    plugins: [{ id: "RpcServer", version: "3.9.2", config: { Password: "plugin-secret" }, installedAt: 1, enabled: true }],
    ...overrides,
  };
}

describe("Public routes", () => {
  let app: express.Application;
  let nodeManager: MockNodeManager;
  let metricsCollector: MockMetricsCollector;

  beforeEach(() => {
    app = express();
    app.use(express.json());
    nodeManager = {
      getAllNodes: vi.fn(() => []),
      getNode: vi.fn(() => null),
    };
    metricsCollector = {
      collectSystemMetrics: vi.fn(async () => ({
        cpu: { usage: 45.5, cores: 8 },
        memory: { total: 16_000_000_000, used: 8_000_000_000, free: 8_000_000_000, percentage: 50 },
        disk: { total: 1_000_000_000_000, used: 500_000_000_000, free: 500_000_000_000, percentage: 50 },
        network: { rx: 1024, tx: 2048 },
      })),
    };
    app.use("/api/public", createPublicRouter(nodeManager as never, metricsCollector as never));
  });

  it("returns public node summaries without sensitive fields", async () => {
    const node = createNode() as NodeInstance & { privateKey: string };
    node.privateKey = "raw-private-key";
    nodeManager.getAllNodes.mockReturnValue([node]);

    const response = await request(app).get("/api/public/nodes");

    expect(response.status).toBe(200);
    expect(response.body.nodes).toHaveLength(1);
    expect(response.body.nodes[0]).toEqual({
      id: "node-1",
      name: "Public Node",
      type: "neo-cli",
      network: "mainnet",
      status: "running",
      version: "3.9.2",
      metrics: {
        blockHeight: 5_000_000,
        headerHeight: 5_000_000,
        connectedPeers: 10,
        syncProgress: 100,
      },
      uptime: 3600,
      lastUpdate: 1_700_000_000_000,
    });
    expect(JSON.stringify(response.body)).not.toContain("/secret");
    expect(JSON.stringify(response.body)).not.toContain("raw-private-key");
    expect(JSON.stringify(response.body)).not.toContain("plugin-secret");
    expect(JSON.stringify(response.body)).not.toContain("signer-1");
  });

  it("returns public node details without management paths or settings", async () => {
    nodeManager.getNode.mockReturnValue(createNode());

    const response = await request(app).get("/api/public/nodes/node-1");

    expect(response.status).toBe(200);
    expect(response.body.node.id).toBe("node-1");
    expect(response.body.node.syncMode).toBe("full");
    expect(response.body.node.metrics.cpuUsage).toBe(25.5);
    expect(response.body.node.paths).toBeUndefined();
    expect(response.body.node.settings).toBeUndefined();
    expect(response.body.node.plugins).toBeUndefined();
  });

  it("returns 404 for missing public nodes", async () => {
    nodeManager.getNode.mockReturnValue(null);

    const response = await request(app).get("/api/public/nodes/missing");

    expect(response.status).toBe(404);
    expect(response.body.error).toBe("Node not found");
  });

  it("reports health only when a node is running with peers", async () => {
    nodeManager.getNode.mockReturnValue(createNode({
      metrics: { ...baseMetrics, connectedPeers: 0 },
    }));

    const response = await request(app).get("/api/public/nodes/node-1/health");

    expect(response.status).toBe(200);
    expect(response.body.healthy).toBe(false);
    expect(response.body.status).toBe("running");
    expect(response.body.peers).toBe(0);
  });

  it("returns status summary derived from node process status and metrics", async () => {
    nodeManager.getAllNodes.mockReturnValue([
      createNode({ id: "running", process: { status: "running" }, metrics: { ...baseMetrics, blockHeight: 100, connectedPeers: 3 } }),
      createNode({ id: "syncing", process: { status: "syncing" }, metrics: { ...baseMetrics, blockHeight: 50, connectedPeers: 2 } }),
      createNode({ id: "error", process: { status: "error" }, metrics: undefined }),
    ]);

    const response = await request(app).get("/api/public/status");

    expect(response.status).toBe(200);
    expect(response.body.status).toEqual({
      totalNodes: 3,
      runningNodes: 1,
      syncingNodes: 1,
      errorNodes: 1,
      totalBlocks: 150,
      totalPeers: 5,
      timestamp: expect.any(Number),
    });
  });

  it("returns simplified public system metrics and omits network counters", async () => {
    const response = await request(app).get("/api/public/metrics/system");

    expect(response.status).toBe(200);
    expect(metricsCollector.collectSystemMetrics).toHaveBeenCalledOnce();
    expect(response.body.metrics).toEqual({
      cpu: { usage: 45.5, cores: 8 },
      memory: { percentage: 50, used: 8_000_000_000, total: 16_000_000_000 },
      disk: { percentage: 50, used: 500_000_000_000, total: 1_000_000_000_000 },
      timestamp: expect.any(Number),
    });
    expect(response.body.metrics.network).toBeUndefined();
  });

  it("returns public node metrics summaries", async () => {
    nodeManager.getAllNodes.mockReturnValue([createNode()]);

    const response = await request(app).get("/api/public/metrics/nodes");

    expect(response.status).toBe(200);
    expect(response.body.metrics).toEqual([
      {
        id: "node-1",
        name: "Public Node",
        status: "running",
        blockHeight: 5_000_000,
        peers: 10,
        cpuUsage: 25.5,
        memoryUsage: 512,
        lastUpdate: 1_700_000_000_000,
      },
    ]);
  });
});
