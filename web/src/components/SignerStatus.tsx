import { useNodeSignerHealth } from "../hooks/useNodes";
import { signerReadinessColor } from "../utils/signerVisibility";

interface SignerStatusProps {
  nodeId: string;
  /** Whether to prefix the status with "Signer " (default: false) */
  showPrefix?: boolean;
  /** Font-size class to apply (default: "text-[11px]") */
  textSize?: string;
}

export function SignerStatus({ nodeId, showPrefix = false, textSize = "text-[11px]" }: SignerStatusProps) {
  const { data: signerHealth } = useNodeSignerHealth(nodeId);

  if (!signerHealth) {
    return null;
  }

  const tone = signerReadinessColor(signerHealth.readiness.status);

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-1 ${textSize} font-medium ${tone}`}>
      {showPrefix ? `Signer ${signerHealth.readiness.status}` : signerHealth.readiness.status}
      {signerHealth.readiness.accountStatus ? ` · ${signerHealth.readiness.accountStatus}` : ""}
    </span>
  );
}
