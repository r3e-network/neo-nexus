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
          ? "bg-cyan-50 text-cyan-700 border border-cyan-200"
          : "bg-slate-100 text-slate-600 border border-slate-200"
      }`}
    >
      {protection.label}
    </span>
  );
}
