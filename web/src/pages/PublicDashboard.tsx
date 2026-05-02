import { 
  Activity, 
  AlertCircle,
  Server, 
  Play, 
  
  CheckCircle,
  Lock,
  Cpu,
  
  Globe,
  Layers,
  RefreshCw,
  Users,
  Clock
} from 'lucide-react';
import { usePublicStatus, usePublicNodes, usePublicSystemMetrics } from '../hooks/usePublic';
import { Link } from 'react-router-dom';
import { formatBytes, formatDuration } from '../utils/format';
import { ProgressBar } from '../components/ProgressBar';
import { StatSkeleton, CardSkeleton } from '../components/LoadingSkeleton';
import { EmptyState } from '../components/EmptyState';
import { REFETCH_INTERVALS } from '../config/constants';

export const PUBLIC_DASHBOARD_STALE_AFTER_MS = REFETCH_INTERVALS.publicDashboard * 3;

function normalizeTimestamp(timestamp: number | undefined): number {
  if (!timestamp || !Number.isFinite(timestamp)) return 0;
  return timestamp < 1_000_000_000_000 ? timestamp * 1000 : timestamp;
}

function formatFreshnessAge(ageMs: number): string {
  const ageSeconds = Math.max(0, Math.floor(ageMs / 1000));
  if (ageSeconds < 10) return 'just now';
  if (ageSeconds < 60) return `${ageSeconds}s ago`;
  const ageMinutes = Math.floor(ageSeconds / 60);
  if (ageMinutes < 60) return `${ageMinutes}m ago`;
  const ageHours = Math.floor(ageMinutes / 60);
  return `${ageHours}h ago`;
}

export function getPublicDashboardFreshness({
  lastUpdatedAt,
  now,
  hasError,
  isFetching,
}: {
  lastUpdatedAt: number;
  now: number;
  hasError: boolean;
  isFetching: boolean;
}) {
  const normalizedLastUpdatedAt = normalizeTimestamp(lastUpdatedAt);
  if (!normalizedLastUpdatedAt) {
    return {
      stale: hasError,
      tone: hasError ? 'warning' : 'muted',
      label: isFetching ? 'Loading current data' : 'No update yet',
    };
  }

  const ageMs = Math.max(0, now - normalizedLastUpdatedAt);
  const ageLabel = formatFreshnessAge(ageMs);
  const stale = hasError || ageMs > PUBLIC_DASHBOARD_STALE_AFTER_MS;

  return {
    stale,
    tone: stale ? 'warning' : 'fresh',
    label: hasError
      ? `Stale, updated ${ageLabel}`
      : isFetching
        ? `Refreshing, updated ${ageLabel}`
        : `Updated ${ageLabel}`,
  };
}

function QueryStateCard({
  icon: Icon,
  title,
  description,
  actionLabel,
  onAction,
}: {
  icon: typeof AlertCircle;
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}) {
  return (
    <div className="card">
      <div className="flex flex-col items-start gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start gap-3">
          <div className="mt-0.5 flex h-10 w-10 items-center justify-center rounded-lg bg-red-50 text-red-700">
            <Icon className="h-5 w-5" />
          </div>
          <div>
            <h3 className="text-base font-semibold text-slate-950">{title}</h3>
            <p className="mt-1 text-sm text-slate-600">{description}</p>
          </div>
        </div>
        {actionLabel && onAction && (
          <button type="button" onClick={onAction} className="btn btn-secondary justify-center">
            <RefreshCw className="h-4 w-4" />
            {actionLabel}
          </button>
        )}
      </div>
    </div>
  );
}

export default function PublicDashboard() {
  const {
    data: status,
    error: statusError,
    isLoading: statusLoading,
    isFetching: statusFetching,
    dataUpdatedAt: statusUpdatedAt,
    refetch: refetchStatus,
  } = usePublicStatus();
  const {
    data: nodes,
    error: nodesError,
    isLoading: nodesLoading,
    isFetching: nodesFetching,
    dataUpdatedAt: nodesUpdatedAt,
    refetch: refetchNodes,
  } = usePublicNodes();
  const {
    data: systemMetrics,
    error: metricsError,
    isLoading: metricsLoading,
    isFetching: metricsFetching,
    refetch: refetchMetrics,
    dataUpdatedAt: metricsUpdatedAt,
  } = usePublicSystemMetrics();
  const lastUpdatedAt = Math.max(
    normalizeTimestamp(status?.timestamp),
    normalizeTimestamp(systemMetrics?.timestamp),
    normalizeTimestamp(statusUpdatedAt),
    normalizeTimestamp(nodesUpdatedAt),
    normalizeTimestamp(metricsUpdatedAt),
    ...(nodes ?? []).map((node) => normalizeTimestamp(node.lastUpdate)),
  );
  const freshness = getPublicDashboardFreshness({
    lastUpdatedAt,
    now: Date.now(),
    hasError: Boolean(statusError || nodesError || metricsError),
    isFetching: statusFetching || nodesFetching || metricsFetching,
  });

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <header className="border-b border-slate-200 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex min-h-16 flex-col gap-3 py-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-teal-50 rounded-lg border border-teal-200 flex items-center justify-center">
                <Activity className="w-6 h-6 text-teal-700" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-slate-950">NeoNexus</h1>
                <p className="text-xs text-slate-600">Node Status Monitor</p>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-3">
              <p
                className={`inline-flex items-center gap-1 rounded-full border px-3 py-1 text-xs font-medium ${
                  freshness.tone === 'warning'
                    ? 'border-amber-200 bg-amber-50 text-amber-800'
                    : freshness.tone === 'fresh'
                      ? 'border-emerald-200 bg-emerald-50 text-emerald-800'
                      : 'border-slate-200 bg-slate-50 text-slate-600'
                }`}
              >
                <Clock className="h-3.5 w-3.5" />
                {freshness.label}
              </p>
              <Link
                to="/login"
                className="btn btn-primary"
              >
                <Lock className="w-4 h-4" />
                Admin Login
              </Link>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Overview Stats */}
        {statusLoading ? (
          <div className="mb-8"><StatSkeleton /></div>
        ) : statusError ? (
          <div className="mb-8">
            <QueryStateCard
              icon={AlertCircle}
              title="Status overview unavailable"
              description={statusError instanceof Error ? statusError.message : 'The public status summary could not be loaded.'}
              actionLabel="Retry"
              onAction={() => void refetchStatus()}
            />
          </div>
        ) : status ? (
          <div className="grid grid-cols-1 gap-4 mb-8 sm:grid-cols-2 lg:grid-cols-4">
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-600 text-sm">Total Nodes</p>
                  <p className="text-2xl font-bold text-slate-950">{status.totalNodes}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center">
                  <Server className="w-6 h-6 text-blue-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-600 text-sm">Running</p>
                  <p className="text-2xl font-bold text-slate-950">{status.runningNodes}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-emerald-500/10 flex items-center justify-center">
                  <Play className="w-6 h-6 text-emerald-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-600 text-sm">Blocks Synced</p>
                  <p className="text-2xl font-bold text-slate-950">{status.totalBlocks.toLocaleString()}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-amber-500/10 flex items-center justify-center">
                  <Layers className="w-6 h-6 text-amber-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-600 text-sm">Total Peers</p>
                  <p className="text-2xl font-bold text-slate-950">{status.totalPeers}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-cyan-500/10 flex items-center justify-center">
                  <Globe className="w-6 h-6 text-cyan-400" />
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="mb-8">
            <QueryStateCard
              icon={AlertCircle}
              title="Status overview unavailable"
              description="No public status summary is available yet."
              actionLabel="Retry"
              onAction={() => void refetchStatus()}
            />
          </div>
        )}

        {/* System Metrics */}
        {metricsLoading ? (
          <div className="mb-8"><CardSkeleton count={1} /></div>
        ) : metricsError ? (
          <div className="mb-8">
            <QueryStateCard
              icon={AlertCircle}
              title="System metrics unavailable"
              description={metricsError instanceof Error ? metricsError.message : 'The public system metrics could not be loaded.'}
              actionLabel="Retry"
              onAction={() => void refetchMetrics()}
            />
          </div>
        ) : systemMetrics ? (
          <div className="card mb-8">
            <h2 className="text-lg font-semibold text-slate-950 mb-4 flex items-center gap-2">
              <Cpu className="w-5 h-5 text-blue-400" />
              System Resources
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-600">CPU Usage</span>
                  <span className="text-slate-950">{systemMetrics.cpu.usage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" />
                <p className="text-xs text-slate-500 mt-1">{systemMetrics.cpu.cores} cores</p>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-600">Memory</span>
                  <span className="text-slate-950">{systemMetrics.memory.percentage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" />
                <p className="text-xs text-slate-500 mt-1">
                  {formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}
                </p>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-600">Disk</span>
                  <span className="text-slate-950">{systemMetrics.disk.percentage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.disk.percentage} color="bg-amber-500" />
                <p className="text-xs text-slate-500 mt-1">
                  {formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}
                </p>
              </div>
            </div>
          </div>
        ) : (
          <div className="mb-8">
            <QueryStateCard
              icon={AlertCircle}
              title="System metrics unavailable"
              description="This public dashboard has not published system resource metrics yet."
              actionLabel="Retry"
              onAction={() => void refetchMetrics()}
            />
          </div>
        )}

        {/* Node List */}
        <div className="card">
          <div className="flex flex-col gap-3 mb-6 sm:flex-row sm:items-center sm:justify-between">
            <h2 className="text-lg font-semibold text-slate-950">Node Status</h2>
            <div className="flex items-center gap-2 text-sm text-slate-600">
              <Clock className="w-4 h-4" />
              Auto-refresh every 5s
            </div>
          </div>

          {nodesLoading ? (
            <CardSkeleton count={2} />
          ) : nodesError ? (
            <QueryStateCard
              icon={AlertCircle}
              title="Node status unavailable"
              description={nodesError instanceof Error ? nodesError.message : 'The public node list could not be loaded.'}
              actionLabel="Retry"
              onAction={() => void refetchNodes()}
            />
          ) : !nodes ? (
            <QueryStateCard
              icon={Server}
              title="Node status unavailable"
              description="This public dashboard has not published any node inventory yet."
              actionLabel="Retry"
              onAction={() => void refetchNodes()}
            />
          ) : nodes.length === 0 ? (
            <EmptyState icon={Server} title="No nodes configured" description="No Neo nodes are currently managed by this instance" />
          ) : (
            <div className="space-y-4">
              {nodes.map((node) => (
                <div
                  key={node.id}
                  className="p-4 bg-slate-50 rounded-lg border border-slate-200"
                >
                  <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                    <div className="flex items-center gap-4">
                      <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                        node.type === 'neo-cli' ? 'bg-blue-500/10' : 'bg-emerald-500/10'
                      }`}>
                        <Activity className={`w-5 h-5 ${
                          node.type === 'neo-cli' ? 'text-blue-400' : 'text-emerald-400'
                        }`} />
                      </div>
                      <div>
                        <h3 className="font-medium text-slate-950">{node.name}</h3>
                        <p className="text-sm text-slate-600">
                          {node.type} • {node.network} • v{node.version}
                        </p>
                      </div>
                    </div>

                    <div className="flex flex-wrap items-center gap-4 lg:gap-8">
                      {/* Status */}
                      <div className="flex items-center gap-2">
                        <span className={`status-badge status-${node.status}`}>
                          {node.status === 'running' && (
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                          )}
                          {node.status}
                        </span>
                      </div>

                      {/* Block Height */}
                      {node.metrics && (
                        <div className="flex items-center gap-2 text-sm">
                          <Layers className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-600">Block:</span>
                          <span className="text-slate-950 font-mono">
                            {node.metrics.blockHeight.toLocaleString()}
                          </span>
                        </div>
                      )}

                      {/* Peers */}
                      {node.metrics && (
                        <div className="flex items-center gap-2 text-sm">
                          <Users className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-600">Peers:</span>
                          <span className="text-slate-950">{node.metrics.connectedPeers}</span>
                        </div>
                      )}

                      {/* Sync Progress */}
                      {node.metrics && node.metrics.syncProgress > 0 && (
                        <div className="flex items-center gap-2 text-sm">
                          <CheckCircle className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-600">Sync:</span>
                          <span className="text-slate-950">{node.metrics.syncProgress.toFixed(1)}%</span>
                        </div>
                      )}

                      {/* Uptime */}
                      {node.uptime && (
                        <div className="flex items-center gap-2 text-sm">
                          <Clock className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-600">Uptime:</span>
                          <span className="text-slate-950">{formatDuration(node.uptime)}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Sync Progress Bar */}
                  {node.metrics && node.metrics.syncProgress > 0 && node.metrics.syncProgress < 100 && (
                    <div className="mt-4">
                      <div className="h-1.5 bg-slate-200 rounded-full overflow-hidden">
                        <div 
                          className="h-full bg-blue-500 transition-all duration-500"
                          style={{ width: `${node.metrics.syncProgress}%` }}
                        />
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="mt-8 text-center text-sm text-slate-500">
          <p>NeoNexus Node Manager v{__APP_VERSION__} • Public Status Page</p>
          <p className="mt-1">
            This is a read-only view.{' '}
            <Link to="/login" className="text-blue-700 hover:text-blue-900">
              Login
            </Link>{' '}
            to manage nodes.
          </p>
        </div>
      </main>
    </div>
  );
}
