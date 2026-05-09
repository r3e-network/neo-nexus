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

interface NodeOwnershipShape {
  type?: string;
  settings?: {
    import?: {
      ownershipMode?: "managed-process" | "managed-config" | "observe-only";
    };
    customConfig?: {
      binaryPath?: string;
    };
  };
}

/**
 * Single source of truth for the "ownership" badge shown on the dashboard,
 * the nodes list, and the detail page header.
 *
 * Native node case: "NeoNexus managed" if not imported; otherwise reflects
 * the imported-node ownershipMode.
 *
 * Sidecar case: NeoNexus polls but doesn't necessarily own the process —
 * "NeoNexus managed" was misleading. Distinguish observe-only (just
 * polling /summary) from managed-process (NeoNexus spawned the binary).
 */
export function nodeOwnershipLabel(node: NodeOwnershipShape): string {
  if (isSidecarNodeType(node.type)) {
    return node.settings?.customConfig?.binaryPath
      ? "Sidecar · managed process"
      : "Sidecar · observe only";
  }
  if (!node.settings?.import) return "NeoNexus managed";
  return node.settings.import.ownershipMode === "managed-process"
    ? "Imported · managed process"
    : node.settings.import.ownershipMode === "managed-config"
      ? "Imported · managed config"
      : "Imported · observe only";
}
