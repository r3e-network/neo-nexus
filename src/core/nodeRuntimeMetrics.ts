import type { NodeMetrics } from "../types";

export interface RuntimeMetricsSource {
  getBlockHeight(): Promise<number | null | undefined>;
  getPeersCount(): Promise<number | null | undefined>;
  getResourceUsage(): Promise<{ memory?: number; cpu?: number } | null | undefined>;
}

export async function collectRuntimeMetrics(source: RuntimeMetricsSource): Promise<NodeMetrics> {
  const [blockHeight, peersCount] = await Promise.all([
    source.getBlockHeight(),
    source.getPeersCount(),
  ]);

  const resources = await source.getResourceUsage();

  return {
    blockHeight: blockHeight ?? 0,
    headerHeight: blockHeight ?? 0,
    connectedPeers: peersCount ?? 0,
    unconnectedPeers: 0,
    syncProgress: 0,
    memoryUsage: resources?.memory ? resources.memory * 1024 * 1024 : 0,
    cpuUsage: resources?.cpu ?? 0,
    lastUpdate: Date.now(),
  };
}
