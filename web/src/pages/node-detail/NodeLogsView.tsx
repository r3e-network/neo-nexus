import { useRef, useEffect } from 'react';
import { FileText } from 'lucide-react';
import { useNodeLogs } from '../../hooks/useNodes';
import { mergeNodeLogs } from '../../utils/realtime';
import { EmptyState } from '../../components/EmptyState';

interface LogEntry {
  timestamp: number;
  level: string;
  message: string;
}

interface NodeLogsViewProps {
  nodeId: string;
  realtimeLogs: LogEntry[];
  connected: boolean;
}

function getLogColor(level: string) {
  switch (level.toLowerCase()) {
    case 'error': return 'text-red-400';
    case 'warn': return 'text-yellow-400';
    case 'debug': return 'text-slate-500';
    default: return 'text-slate-300';
  }
}

export function NodeLogsView({ nodeId, realtimeLogs, connected }: NodeLogsViewProps) {
  const { data: logs = [] } = useNodeLogs(nodeId, 50);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const displayedLogs = mergeNodeLogs(logs, realtimeLogs);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs, realtimeLogs]);

  return (
    <div className="card animate-fade-in">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-white">Logs</h3>
        <span className="text-sm text-slate-400">
          {connected ? 'Live stream connected' : 'Reconnecting live stream'} · Last 50 entries
        </span>
      </div>
      <div className="bg-slate-950 rounded-lg p-4 font-mono text-xs max-h-[500px] overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
        {displayedLogs.length === 0 ? (
          <EmptyState
            icon={FileText}
            title="No logs yet"
            description="Logs will appear here when the node is running"
          />
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
                <span className={`${getLogColor(log.level)} break-all`}>
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
