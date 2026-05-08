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

  it("preserves null for fields that the node type doesn't expose", async () => {
    // Sidecars (e.g. neofura) have no peer concept and no in-process
    // resource usage. Coercing null to 0 was misleading the dashboard
    // ("0 peers / 0% CPU / 0 B" looked like a measured value rather than
    // "not applicable"). Block height keeps its 0 default — that's a
    // semantically meaningful "genesis-or-unknown" reading.
    const metrics = await collectRuntimeMetrics({
      getBlockHeight: async () => null,
      getPeersCount: async () => undefined,
      getResourceUsage: async () => null,
    });

    expect(metrics).toMatchObject({
      blockHeight: 0,
      headerHeight: 0,
      connectedPeers: null,
      memoryUsage: null,
      cpuUsage: null,
    });
  });
});
