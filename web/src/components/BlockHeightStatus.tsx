import { AlertTriangle, CheckCircle2, HelpCircle, LockKeyhole, RotateCw } from "lucide-react";
import {
  blockHeightBadgeClass,
  blockHeightDetailClass,
  formatBlockHeight,
  type BlockHeightStatusDisplay,
} from "../utils/blockHeightStatus";

interface BlockHeightStatusProps {
  status: BlockHeightStatusDisplay;
  className?: string;
  showDetail?: boolean;
  showLatest?: boolean;
}

function statusIcon(status: BlockHeightStatusDisplay["status"]) {
  if (status === "synced") return CheckCircle2;
  if (status === "syncing") return RotateCw;
  if (status === "private") return LockKeyhole;
  return HelpCircle;
}

export function BlockHeightStatus({
  status,
  className = "",
  showDetail = true,
  showLatest = false,
}: BlockHeightStatusProps) {
  const Icon = statusIcon(status.status);
  const latestText = showLatest && status.networkHeight !== null
    ? `Latest ${status.networkHeight.toLocaleString()}`
    : null;
  const detailTone = blockHeightDetailClass(status.status);
  const shouldShowDetail = showDetail && (status.status !== "synced" || latestText !== null);

  return (
    <div className={`min-w-0 ${className}`}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-medium text-slate-900">Block {formatBlockHeight(status)}</span>
        <span
          className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium ${blockHeightBadgeClass(status.status)}`}
          title={status.detail}
        >
          <Icon className={`h-3 w-3 ${status.status === "syncing" ? "animate-spin" : ""}`} />
          {status.label}
        </span>
      </div>
      {shouldShowDetail && (
        <p className={`mt-1 flex items-start gap-1 text-xs leading-5 ${detailTone}`}>
          {!status.safeToUseAsLatest && status.status !== "private" && <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />}
          <span>{latestText ? `${status.detail} · ${latestText}` : status.detail}</span>
        </p>
      )}
    </div>
  );
}
