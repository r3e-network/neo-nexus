import { describe, expect, it } from "vitest";
import { collectRuntimeMetrics } from "../../src/core/nodeRuntimeMetrics";

describe("collectRuntimeMetrics", () => {
  it("collects block, peer, and resource metrics into the persisted metric shape", async () => {
    const metrics = await collectRuntimeMetrics({
      getBlockHeight: async () => 123,
      getPeersCount: async () => 7,
      getResourceUsage: async () => ({ memory: 64, cpu: 12.5 }),
    });

    expect(metrics).toMatchObject({
      blockHeight: 123,
      headerHeight: 123,
      connectedPeers: 7,
      unconnectedPeers: 0,
      syncProgress: 0,
      memoryUsage: 64 * 1024 * 1024,
      cpuUsage: 12.5,
    });
    expect(metrics.lastUpdate).toEqual(expect.any(Number));
  });

  it("defaults missing node runtime values to zero", async () => {
    const metrics = await collectRuntimeMetrics({
      getBlockHeight: async () => null,
      getPeersCount: async () => undefined,
      getResourceUsage: async () => null,
    });

    expect(metrics).toMatchObject({
      blockHeight: 0,
      headerHeight: 0,
      connectedPeers: 0,
      memoryUsage: 0,
      cpuUsage: 0,
    });
  });
});
