/**
 * Sidecar node types — observe-only adapters that NeoNexus monitors but
 * doesn't directly own the lifecycle of (e.g. neofura, an indexer that
 * runs externally and exposes /summary). Role orchestration, data
 * contexts, snapshots, and plugin management don't apply to sidecars,
 * so a few panes hide themselves when isSidecarNodeType(type) is true.
 */
const SIDECAR_NODE_TYPES = new Set<string>(["neofura"]);

export function isSidecarNodeType(type: string | undefined | null): boolean {
  return type != null && SIDECAR_NODE_TYPES.has(type);
}
