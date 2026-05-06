export type BlockHeightSyncStatus = "synced" | "syncing" | "unknown" | "private";

export interface NetworkHeightSnapshot {
  mainnet?: number | null;
  testnet?: number | null;
  timestamp?: number;
}

export interface BlockHeightStatusDisplay {
  status: BlockHeightSyncStatus;
  label: string;
  detail: string;
  localHeight: number | null;
  networkHeight: number | null;
  remainingBlocks: number | null;
  progressPercent: number;
  stale: boolean;
  safeToUseAsLatest: boolean;
  checkedAt: number;
}

export interface BlockHeightStatusSource {
  network: string;
  metrics?: {
    blockHeight?: number | null;
    syncProgress?: number | null;
    blockHeightStatus?: Partial<BlockHeightStatusDisplay> & {
      message?: string;
      progressPercent?: number | null;
    };
  };
}

function normalizeHeight(value: number | null | undefined): number | null {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    return null;
  }
  return Math.floor(value);
}

function normalizeReferenceHeight(value: number | null | undefined): number | null {
  const height = normalizeHeight(value);
  return height !== null && height > 0 ? height : null;
}

function progressFromRatio(ratio: number | null | undefined): number {
  if (typeof ratio !== "number" || !Number.isFinite(ratio)) {
    return 0;
  }
  const normalized = ratio <= 1 ? ratio * 100 : ratio;
  return Math.max(0, Math.min(100, Math.round(normalized * 10) / 10));
}

function progressFromHeights(localHeight: number | null, networkHeight: number | null, fallbackRatio?: number | null): number {
  if (localHeight === null || networkHeight === null) {
    return progressFromRatio(fallbackRatio);
  }
  if (localHeight >= networkHeight) {
    return 100;
  }
  return Math.max(0, Math.min(100, Math.round((localHeight / networkHeight) * 1000) / 10));
}

function latestHeightForNetwork(network: string, snapshot?: NetworkHeightSnapshot | null): number | null {
  if (!snapshot) {
    return null;
  }
  if (network === "mainnet") {
    return normalizeReferenceHeight(snapshot.mainnet);
  }
  if (network === "testnet") {
    return normalizeReferenceHeight(snapshot.testnet);
  }
  return null;
}

export function getBlockHeightStatus(
  node: BlockHeightStatusSource,
  networkHeights?: NetworkHeightSnapshot | null,
  checkedAt = Date.now(),
): BlockHeightStatusDisplay {
  const serverStatus = node.metrics?.blockHeightStatus;
  const localHeight = normalizeHeight(node.metrics?.blockHeight ?? serverStatus?.localHeight ?? null);
  const networkHeight = latestHeightForNetwork(node.network, networkHeights)
    ?? normalizeReferenceHeight(serverStatus?.networkHeight ?? null);

  if (node.network === "private") {
    return {
      status: "private",
      label: "Private",
      detail: "Private network has no public reference height",
      localHeight,
      networkHeight: null,
      remainingBlocks: null,
      progressPercent: progressFromRatio(node.metrics?.syncProgress ?? serverStatus?.progressPercent ?? null),
      stale: false,
      safeToUseAsLatest: false,
      checkedAt,
    };
  }

  if (localHeight === null) {
    return {
      status: "unknown",
      label: "No height",
      detail: "Block height has not been reported yet",
      localHeight,
      networkHeight,
      remainingBlocks: null,
      progressPercent: progressFromRatio(node.metrics?.syncProgress ?? serverStatus?.progressPercent ?? null),
      stale: false,
      safeToUseAsLatest: false,
      checkedAt,
    };
  }

  if (networkHeight === null) {
    return {
      status: "unknown",
      label: "Unknown latest",
      detail: "Latest network height unavailable",
      localHeight,
      networkHeight: null,
      remainingBlocks: null,
      progressPercent: progressFromRatio(node.metrics?.syncProgress ?? serverStatus?.progressPercent ?? null),
      stale: false,
      safeToUseAsLatest: false,
      checkedAt,
    };
  }

  const remainingBlocks = Math.max(0, networkHeight - localHeight);
  const synced = remainingBlocks === 0;

  return {
    status: synced ? "synced" : "syncing",
    label: synced ? "Synced" : "Syncing",
    detail: synced
      ? "Caught up to latest network height"
      : `${remainingBlocks.toLocaleString()} block${remainingBlocks === 1 ? "" : "s"} behind latest`,
    localHeight,
    networkHeight,
    remainingBlocks,
    progressPercent: progressFromHeights(localHeight, networkHeight, node.metrics?.syncProgress ?? serverStatus?.progressPercent ?? null),
    stale: !synced,
    safeToUseAsLatest: synced,
    checkedAt,
  };
}

export function formatBlockHeight(status: Pick<BlockHeightStatusDisplay, "localHeight">): string {
  return status.localHeight === null ? "—" : status.localHeight.toLocaleString();
}

export function blockHeightBadgeClass(status: BlockHeightSyncStatus): string {
  if (status === "synced") {
    return "border-emerald-200 bg-emerald-50 text-emerald-700";
  }
  if (status === "syncing") {
    return "border-blue-200 bg-blue-50 text-blue-700";
  }
  if (status === "private") {
    return "border-slate-200 bg-slate-50 text-slate-600";
  }
  return "border-amber-200 bg-amber-50 text-amber-800";
}

export function blockHeightDetailClass(status: BlockHeightSyncStatus): string {
  if (status === "syncing") {
    return "text-blue-700";
  }
  if (status === "unknown") {
    return "text-amber-700";
  }
  return "text-slate-500";
}

export function blockHeightProgressClass(status: BlockHeightSyncStatus): string {
  if (status === "synced") {
    return "bg-emerald-500";
  }
  if (status === "syncing") {
    return "bg-blue-500";
  }
  return "bg-slate-400";
}
