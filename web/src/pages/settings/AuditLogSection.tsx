import { useState } from "react";
import { ClipboardList, ChevronLeft, ChevronRight } from "lucide-react";
import { useAuditLog } from "../../hooks/useAuditLog";

const PAGE_SIZE = 50;

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diffMs = now - timestamp * 1000;
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
  let colorClasses = "bg-blue-500/15 text-blue-300";

  if (action === "node.start") {
    colorClasses = "bg-emerald-500/15 text-emerald-300";
  } else if (action === "node.stop") {
    colorClasses = "bg-amber-500/15 text-amber-300";
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
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center">
            <ClipboardList className="w-5 h-5 text-violet-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Audit Log</h2>
            <p className="text-slate-400 text-sm">Recent system activity and events</p>
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
        <div className="text-slate-400 text-sm py-4 text-center">Loading audit log...</div>
      ) : entries.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-10 text-slate-500">
          <ClipboardList className="w-10 h-10 mb-3 opacity-40" />
          <p className="text-sm">No audit log entries found.</p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-700/50">
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider w-28">Time</th>
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Action</th>
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Resource</th>
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">ID</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/30">
              {entries.map((entry) => (
                <tr key={entry.id} className="group hover:bg-slate-800/30 transition-colors">
                  <td className="py-2.5 text-slate-400 tabular-nums whitespace-nowrap">
                    {formatRelativeTime(entry.timestamp)}
                  </td>
                  <td className="py-2.5">
                    <ActionBadge action={entry.action} />
                  </td>
                  <td className="py-2.5 text-slate-300">{entry.resourceType}</td>
                  <td className="py-2.5 text-slate-400 font-mono text-xs truncate max-w-[12rem]" title={entry.resourceId}>
                    {entry.resourceId}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
