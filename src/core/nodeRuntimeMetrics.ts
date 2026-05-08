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

  // Preserve null for fields that don't apply to a given node type. E.g.
  // sidecars (neofura) have no peer concept and no NeoNexus-managed process
  // to sample CPU/memory from. The UI falls back to "—" when null instead
  // of showing a misleading "0 peers / 0% CPU / 0 B".
  return {
    blockHeight: blockHeight ?? 0,
    headerHeight: blockHeight ?? 0,
    connectedPeers: typeof peersCount === 'number' ? peersCount : null,
    unconnectedPeers: 0,
    syncProgress: 0,
    memoryUsage: typeof resources?.memory === 'number' ? resources.memory * 1024 * 1024 : null,
    cpuUsage: typeof resources?.cpu === 'number' ? resources.cpu : null,
    lastUpdate: Date.now(),
  };
}
