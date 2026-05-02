import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createMetricsRouter } from "../../src/api/routes/metrics";

describe("Actual metrics router", () => {
  let app: express.Application;
  let nodeManager: {
    updateMetrics: ReturnType<typeof vi.fn>;
    getNode: ReturnType<typeof vi.fn>;
  };
  let metricsCollector: {
    collectSystemMetrics: ReturnType<typeof vi.fn>;
    getProcessMetrics: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());
    nodeManager = {
      updateMetrics: vi.fn().mockResolvedValue(undefined),
      getNode: vi.fn(),
    };
    metricsCollector = {
      collectSystemMetrics: vi.fn().mockResolvedValue({
        cpu: { usage: 12.5, cores: 8 },
        memory: { percentage: 50, used: 100, total: 200 },
        disk: { percentage: 25, used: 250, total: 1000 },
        network: { rx: 10, tx: 20 },
      }),
      getProcessMetrics: vi.fn().mockResolvedValue({ cpu: 1.5, memory: 1024 }),
    };
    app.use("/api/metrics", createMetricsRouter(nodeManager as never, metricsCollector as never));
  });

  it("returns system metrics from the collector", async () => {
    const response = await request(app).get("/api/metrics/system");

    expect(response.status).toBe(200);
    expect(response.body.metrics.cpu.usage).toBe(12.5);
    expect(response.body.metrics.network).toEqual({ rx: 10, tx: 20 });
  });

  it("returns 500 when system metric collection fails", async () => {
    metricsCollector.collectSystemMetrics.mockRejectedValue(new Error("collector down"));

    const response = await request(app).get("/api/metrics/system");

    expect(response.status).toBe(500);
    expect(response.body.error).toBe("collector down");
  });

  it("returns 404 for node metrics without refreshing when the node does not exist", async () => {
    nodeManager.getNode.mockReturnValue(null);

    const response = await request(app).get("/api/metrics/nodes/missing/metrics");

    expect(response.status).toBe(404);
    expect(nodeManager.updateMetrics).not.toHaveBeenCalled();
  });

  it("returns refreshed node metrics after updating the node", async () => {
    const staleMetrics = { blockHeight: 10, headerHeight: 12, connectedPeers: 1 };
    const refreshedMetrics = { blockHeight: 11, headerHeight: 12, connectedPeers: 2 };
    nodeManager.getNode
      .mockReturnValueOnce({ id: "node-1", metrics: staleMetrics })
      .mockReturnValueOnce({ id: "node-1", metrics: refreshedMetrics });

    const response = await request(app).get("/api/metrics/nodes/node-1/metrics");

    expect(response.status).toBe(200);
    expect(nodeManager.updateMetrics).toHaveBeenCalledWith("node-1");
    expect(response.body.metrics).toEqual(refreshedMetrics);
  });

  it("returns 404 when a node disappears after metrics refresh", async () => {
    nodeManager.getNode
      .mockReturnValueOnce({ id: "node-1", metrics: { blockHeight: 10 } })
      .mockReturnValueOnce(null);

    const response = await request(app).get("/api/metrics/nodes/node-1/metrics");

    expect(response.status).toBe(404);
    expect(nodeManager.updateMetrics).toHaveBeenCalledWith("node-1");
  });

  it("reports healthy only when the node is running with peers", async () => {
    nodeManager.getNode.mockReturnValue({
      process: { status: "running" },
      metrics: { blockHeight: 10, connectedPeers: 2 },
    });

    const response = await request(app).get("/api/metrics/nodes/node-1/health");

    expect(response.status).toBe(200);
    expect(response.body.healthy).toBe(true);
    expect(response.body.metrics).toEqual({ blockHeight: 10, peers: 2 });
  });

  it("returns 404 when process metrics are requested for a node without pid", async () => {
    nodeManager.getNode.mockReturnValue({ process: { status: "stopped" } });

    const response = await request(app).get("/api/metrics/nodes/node-1/process");

    expect(response.status).toBe(404);
    expect(metricsCollector.getProcessMetrics).not.toHaveBeenCalled();
  });
});
