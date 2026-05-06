import type { BlockHeightStatus, NodeMetrics, NodeNetwork } from '../types/index';

export interface NetworkHeightReader {
  getHeight(network: string): number;
}

type MetricsSnapshot = Pick<NodeMetrics, 'blockHeight'> & Partial<Pick<NodeMetrics, 'syncProgress'>>;

function normalizeHeight(value: number | null | undefined): number {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) {
    return 0;
  }
  return Math.floor(value);
}

function normalizeNetworkHeight(value: number | null | undefined): number | null {
  const height = normalizeHeight(value);
  return height > 0 ? height : null;
}

function progressFromRatio(ratio: number | null | undefined): number {
  if (typeof ratio !== 'number' || !Number.isFinite(ratio)) {
    return 0;
  }
  const normalized = ratio <= 1 ? ratio * 100 : ratio;
  return Math.max(0, Math.min(100, Math.round(normalized * 10) / 10));
}

function progressFromHeights(localHeight: number, networkHeight: number | null, fallbackRatio?: number): number {
  if (networkHeight === null) {
    return progressFromRatio(fallbackRatio);
  }
  if (localHeight >= networkHeight) {
    return 100;
  }
  return Math.max(0, Math.min(100, Math.round((localHeight / networkHeight) * 1000) / 10));
}

export function buildBlockHeightStatus(
  metrics: MetricsSnapshot | undefined,
  network: NodeNetwork | string,
  networkHeight: number | null | undefined,
  checkedAt = Date.now(),
): BlockHeightStatus {
  const localHeight = normalizeHeight(metrics?.blockHeight);
  const referenceHeight = normalizeNetworkHeight(networkHeight);

  if (network === 'private') {
    return {
      status: 'private',
      localHeight,
      networkHeight: null,
      remainingBlocks: null,
      progressPercent: progressFromRatio(metrics?.syncProgress),
      stale: false,
      safeToUseAsLatest: false,
      checkedAt,
      message: 'Private network has no public reference height',
    };
  }

  if (referenceHeight === null) {
    return {
      status: 'unknown',
      localHeight,
      networkHeight: null,
      remainingBlocks: null,
      progressPercent: progressFromRatio(metrics?.syncProgress),
      stale: false,
      safeToUseAsLatest: false,
      checkedAt,
      message: 'Latest network height unavailable',
    };
  }

  const remainingBlocks = Math.max(0, referenceHeight - localHeight);
  const synced = remainingBlocks === 0;

  return {
    status: synced ? 'synced' : 'syncing',
    localHeight,
    networkHeight: referenceHeight,
    remainingBlocks,
    progressPercent: progressFromHeights(localHeight, referenceHeight, metrics?.syncProgress),
    stale: !synced,
    safeToUseAsLatest: synced,
    checkedAt,
    message: synced
      ? 'Caught up to latest network height'
      : `${remainingBlocks} block${remainingBlocks === 1 ? '' : 's'} behind latest`,
  };
}

export function enrichMetricsWithBlockHeightStatus(
  metrics: NodeMetrics | undefined,
  network: NodeNetwork | string,
  networkHeightReader?: NetworkHeightReader,
): NodeMetrics | undefined {
  if (!metrics) {
    return metrics;
  }

  return {
    ...metrics,
    blockHeightStatus: buildBlockHeightStatus(
      metrics,
      network,
      networkHeightReader?.getHeight(network) ?? null,
    ),
  };
}
