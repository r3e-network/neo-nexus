import { useState } from "react";
import { ClipboardList, ChevronLeft, ChevronRight } from "lucide-react";
import { useAuditLog } from "../../hooks/useAuditLog";

const PAGE_SIZE = 50;

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const normalizedTimestamp = timestamp > 1_000_000_000_000 ? timestamp : timestamp * 1000;
  const diffMs = Math.max(0, now - normalizedTimestamp);
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return `${diffSec}s ago`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay}d ago`;
}

function ActionBadge({ action }: { action: string }) {
  let colorClasses = "bg-blue-50 text-blue-700 border border-blue-200";

  if (action === "node.start") {
    colorClasses = "bg-emerald-50 text-emerald-700 border border-emerald-200";
  } else if (action === "node.stop") {
    colorClasses = "bg-amber-50 text-amber-700 border border-amber-200";
  }

  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-mono font-medium ${colorClasses}`}>
      {action}
    </span>
  );
}

export function AuditLogSection() {
  const [offset, setOffset] = useState(0);
  const { data: entries = [], isLoading } = useAuditLog(PAGE_SIZE, offset);

  const hasPrev = offset > 0;
  const hasNext = entries.length === PAGE_SIZE;

  return (
    <div className="card">
      <div className="mb-4 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center">
            <ClipboardList className="w-5 h-5 text-violet-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-slate-950">Audit Log</h2>
            <p className="text-slate-600 text-sm">Recent system activity and events</p>
          </div>
        </div>

        {/* Pagination controls */}
        <div className="flex items-center gap-2">
          <button
            className="btn btn-secondary flex items-center gap-1 text-sm"
            onClick={() => setOffset((o) => Math.max(0, o - PAGE_SIZE))}
            disabled={!hasPrev || isLoading}
          >
            <ChevronLeft className="w-4 h-4" />
            Newer
          </button>
          <button
            className="btn btn-secondary flex items-center gap-1 text-sm"
            onClick={() => setOffset((o) => o + PAGE_SIZE)}
            disabled={!hasNext || isLoading}
          >
            Older
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="text-slate-600 text-sm py-4 text-center">Loading audit log...</div>
      ) : entries.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-10 text-slate-500">
          <ClipboardList className="w-10 h-10 mb-3 opacity-40" />
          <p className="text-sm">No audit log entries found.</p>
        </div>
      ) : (
        <>
          <div className="space-y-3 sm:hidden">
            {entries.map((entry) => (
              <div key={entry.id} className="rounded-lg border border-slate-200 bg-slate-50 p-3">
                <div className="flex items-center justify-between gap-3">
                  <span className="text-sm text-slate-600 tabular-nums">{formatRelativeTime(entry.timestamp)}</span>
                  <ActionBadge action={entry.action} />
                </div>
                {(entry.resourceType || entry.resourceId) && (
                  <div className="mt-3 grid grid-cols-[5rem_1fr] gap-x-3 gap-y-1 text-xs">
                    {entry.resourceType && (
                      <>
                        <span className="font-medium uppercase text-slate-500">Resource</span>
                        <span className="text-slate-700">{entry.resourceType}</span>
                      </>
                    )}
                    {entry.resourceId && (
                      <>
                        <span className="font-medium uppercase text-slate-500">ID</span>
                        <span className="break-all font-mono text-slate-600">{entry.resourceId}</span>
                      </>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="hidden overflow-x-auto sm:block">
            <table className="w-full min-w-[640px] text-sm">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="pb-2 text-left text-xs font-medium text-slate-500 uppercase tracking-wide w-28">Time</th>
                  <th className="pb-2 text-left text-xs font-medium text-slate-500 uppercase tracking-wide">Action</th>
                  <th className="pb-2 text-left text-xs font-medium text-slate-500 uppercase tracking-wide">Resource</th>
                  <th className="pb-2 text-left text-xs font-medium text-slate-500 uppercase tracking-wide">ID</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-200">
                {entries.map((entry) => (
                  <tr key={entry.id} className="group hover:bg-slate-50 transition-colors">
                    <td className="py-2.5 text-slate-600 tabular-nums whitespace-nowrap">
                      {formatRelativeTime(entry.timestamp)}
                    </td>
                    <td className="py-2.5">
                      <ActionBadge action={entry.action} />
                    </td>
                    <td className="py-2.5 text-slate-700">{entry.resourceType}</td>
                    <td className="py-2.5 text-slate-600 font-mono text-xs truncate max-w-[12rem]" title={entry.resourceId}>
                      {entry.resourceId}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
