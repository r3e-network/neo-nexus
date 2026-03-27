import type { NodeNetwork } from '../types/index';
import { getSeedList } from '../utils/network';

interface NodeHeightRecord {
  height: number;
  firstSeenAt: number;
}

const STALL_THRESHOLD_MS = 5 * 60 * 1000; // 5 minutes

export class NetworkHeightTracker {
  private networkHeights: Map<string, number> = new Map();
  private nodeHeights: Map<string, NodeHeightRecord> = new Map();

  /**
   * Returns cached network height or 0 if unknown.
   */
  getHeight(network: string): number {
    return this.networkHeights.get(network) ?? 0;
  }

  /**
   * Updates the cached height for a network, but only if the new value is higher.
   */
  setHeight(network: string, height: number): void {
    const current = this.networkHeights.get(network) ?? 0;
    if (height > current) {
      this.networkHeights.set(network, height);
    }
  }

  /**
   * Returns a 0–1.0 sync progress ratio based on localHeight vs network height.
   */
  getSyncProgress(localHeight: number, network: string): number {
    const networkHeight = this.getHeight(network);
    if (networkHeight === 0 || localHeight === 0) return 0;
    if (localHeight >= networkHeight) return 1.0;
    return localHeight / networkHeight;
  }

  /**
   * Records the current height for a node. Resets the firstSeenAt timestamp
   * when the height changes so stalling detection restarts.
   */
  recordNodeHeight(nodeId: string, height: number): void {
    const existing = this.nodeHeights.get(nodeId);
    if (existing && existing.height === height) {
      // Same height — keep the existing record (don't reset timer)
      return;
    }
    // Height changed (or first record) — reset timer
    this.nodeHeights.set(nodeId, { height, firstSeenAt: Date.now() });
  }

  /**
   * Returns true if the node has been at the same height for longer than
   * STALL_THRESHOLD_MS (5 minutes).
   */
  isStalled(nodeId: string, currentHeight: number): boolean {
    const record = this.nodeHeights.get(nodeId);
    if (!record) return false;
    if (record.height !== currentHeight) return false;
    return Date.now() - record.firstSeenAt > STALL_THRESHOLD_MS;
  }

  /**
   * Fetches the current block count from seed nodes for the given network.
   * Tries each seed in order with a 3-second timeout per seed.
   * Returns the highest height found, or null if all seeds fail.
   */
  async fetchNetworkHeight(network: NodeNetwork): Promise<number | null> {
    if (network === 'private') return null;

    const seeds = getSeedList(network);
    const rpcPort = network === 'mainnet' ? 10332 : 20332;

    for (const seed of seeds) {
      const host = seed.split(':')[0];
      const url = `http://${host}:${rpcPort}`;

      try {
        const height = await this.fetchBlockCount(url);
        if (height !== null) {
          return height;
        }
      } catch {
        // Try next seed
      }
    }

    return null;
  }

  private async fetchBlockCount(rpcUrl: string): Promise<number | null> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 3000);

    try {
      const response = await fetch(rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'getblockcount',
          params: [],
          id: 1,
        }),
        signal: controller.signal,
      });

      if (!response.ok) return null;

      const data = await response.json() as { result?: number; error?: unknown };
      if (typeof data.result === 'number') {
        return data.result;
      }
      return null;
    } catch {
      return null;
    } finally {
      clearTimeout(timeoutId);
    }
  }
}
