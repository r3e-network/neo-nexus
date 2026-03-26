type NodeLike = {
  settings?: {
    keyProtection?: {
      mode?: "standard" | "secure-signer";
      signerName?: string;
      signerProfileId?: string;
    };
  };
};

export function countProtectedNodes(nodes: NodeLike[]): number {
  return nodes.filter((node) => node.settings?.keyProtection?.mode === "secure-signer").length;
}

export function getNodeProtectionLabel(node: NodeLike): {
  label: string;
  detail?: string;
  tone: "muted" | "secure";
} {
  if (node.settings?.keyProtection?.mode === "secure-signer") {
    return {
      label: "Secure Signer",
      detail: node.settings.keyProtection.signerName || node.settings.keyProtection.signerProfileId,
      tone: "secure",
    };
  }

  return {
    label: "Standard",
    tone: "muted",
  };
}

export function signerReadinessColor(status: string): string {
  if (status === "reachable") return "bg-emerald-500/10 text-emerald-300";
  if (status === "warning") return "bg-amber-500/10 text-amber-300";
  return "bg-red-500/10 text-red-300";
}
