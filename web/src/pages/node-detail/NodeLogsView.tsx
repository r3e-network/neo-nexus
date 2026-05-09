import { useRef, useEffect, useMemo, useState } from 'react';
import { ClipboardCopy, FileText, Pause, Play, Trash2 } from 'lucide-react';
import { useNodeLogs } from '../../hooks/useNodes';
import { mergeNodeLogs } from '../../utils/realtime';
import { EmptyState } from '../../components/EmptyState';
import { isSidecarNodeType } from '../../utils/nodeKind';

interface LogEntry {
  timestamp: number;
  level: string;
  message: string;
}

interface NodeLogsViewProps {
  nodeId: string;
  realtimeLogs: LogEntry[];
  connected: boolean;
  // Sidecars (e.g. neofura) run as external systemd services — NeoNexus
  // doesn't own their stdout, so live logs aren't available. The empty
  // state tells the operator where to look instead of falsely promising
  // logs that will never appear.
  nodeType?: string;
}

function getLogColor(level: string) {
  switch (level.toLowerCase()) {
    case 'error': return 'text-red-300';
    case 'warn': return 'text-amber-300';
    case 'debug': return 'text-slate-500';
    default: return 'text-slate-200';
  }
}

export function NodeLogsToolbar({
  connected,
  following,
  onToggleFollow,
  onCopy,
  onClear,
  isSidecar = false,
}: {
  connected: boolean;
  following: boolean;
  onToggleFollow: () => void;
  onCopy: () => void;
  onClear: () => void;
  isSidecar?: boolean;
}) {
  const statusText = isSidecar
    ? 'External process — log capture unavailable'
    : connected
      ? 'Live stream connected · Last 50 entries'
      : 'Reconnecting live stream · Last 50 entries';
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h3 className="text-lg font-semibold text-slate-950">Logs</h3>
        <span className="text-sm text-slate-600">{statusText}</span>
      </div>
      <div className="flex flex-wrap gap-2">
        <button type="button" className="btn btn-secondary" onClick={onToggleFollow}>
          {following ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
          {following ? 'Pause follow' : 'Resume follow'}
        </button>
        <button type="button" className="btn btn-secondary" onClick={onCopy}>
          <ClipboardCopy className="h-4 w-4" />
          Copy logs
        </button>
        <button type="button" className="btn btn-secondary" onClick={onClear}>
          <Trash2 className="h-4 w-4" />
          Clear view
        </button>
      </div>
    </div>
  );
}

export function NodeLogsView({ nodeId, realtimeLogs, connected, nodeType }: NodeLogsViewProps) {
  const isSidecar = isSidecarNodeType(nodeType);
  const { data: logs = [] } = useNodeLogs(nodeId, 50);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [following, setFollowing] = useState(true);
  const [hiddenThroughTimestamp, setHiddenThroughTimestamp] = useState(0);
  const mergedLogs = useMemo(() => mergeNodeLogs(logs, realtimeLogs), [logs, realtimeLogs]);
  const displayedLogs = mergedLogs.filter((log) => log.timestamp > hiddenThroughTimestamp);

  useEffect(() => {
    if (following) {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [displayedLogs, following]);

  const handleScroll = () => {
    const element = scrollContainerRef.current;
    if (!element) return;
    const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight;
    setFollowing(distanceFromBottom < 24);
  };

  const handleCopy = () => {
    const text = displayedLogs
      .map((log) => `${new Date(log.timestamp).toISOString()} ${log.level.toUpperCase()} ${log.message}`)
      .join('\n');
    void navigator.clipboard?.writeText(text);
  };

  const handleClear = () => {
    const latest = mergedLogs.reduce((max, log) => Math.max(max, log.timestamp), hiddenThroughTimestamp);
    setHiddenThroughTimestamp(latest);
  };

  return (
    <div className="card animate-fade-in">
      <div className="mb-4">
        <NodeLogsToolbar
          connected={connected}
          following={following}
          onToggleFollow={() => setFollowing((value) => !value)}
          onCopy={handleCopy}
          onClear={handleClear}
          isSidecar={isSidecar}
        />
      </div>
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className="log-console rounded-lg p-4 font-mono text-xs max-h-[500px] overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent shadow-inner"
      >
        {displayedLogs.length === 0 ? (
          isSidecar ? (
            <EmptyState
              icon={FileText}
              title="Log capture not supported"
              description="This is an observe-only sidecar — NeoNexus doesn't own its stdout. View logs on the host with: journalctl -u <service> -f"
            />
          ) : (
            <EmptyState
              icon={FileText}
              title="No logs yet"
              description="Logs will appear here when the node is running"
            />
          )
        ) : (
          <div className="space-y-1">
            {displayedLogs.map((log) => (
              <div key={`${log.timestamp}:${log.level}:${log.message}`} className="flex gap-3">
                <span className="text-slate-600 shrink-0">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span className={`uppercase text-xs font-bold shrink-0 w-12 ${getLogColor(log.level)}`}>
                  {log.level}
                </span>
                <span className={`${getLogColor(log.level)} whitespace-pre-wrap break-words`}>
                  {log.message}
                </span>
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>
    </div>
  );
}
