import { getNodeProtectionLabel } from "../utils/signerVisibility";

type NodeLike = {
  settings?: {
    keyProtection?: {
      mode?: "standard" | "secure-signer";
      signerName?: string;
      signerProfileId?: string;
    };
  };
};

interface NodeProtectionLabelProps {
  node: NodeLike;
  /** Extra padding to apply to the badge (default: "px-2.5 py-1") */
  padding?: string;
}

export function NodeProtectionLabel({ node, padding = "px-2.5 py-1" }: NodeProtectionLabelProps) {
  const protection = getNodeProtectionLabel(node);

  return (
    <span
      className={`inline-flex items-center rounded-full ${padding} text-[11px] font-medium ${
        protection.tone === "secure"
          ? "bg-cyan-500/10 text-cyan-300"
          : "bg-slate-700 text-slate-400"
      }`}
    >
      {protection.label}
    </span>
  );
}
