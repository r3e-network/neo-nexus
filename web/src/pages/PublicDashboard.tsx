import { 
  Activity, 
  Server, 
  Play, 
  
  CheckCircle,
  Lock,
  Cpu,
  
  Globe,
  Layers,
  Users,
  Clock
} from 'lucide-react';
import { usePublicStatus, usePublicNodes, usePublicSystemMetrics } from '../hooks/usePublic';
import { Link } from 'react-router-dom';
import { formatBytes, formatDuration } from '../utils/format';
import { ProgressBar } from '../components/ProgressBar';
import { StatSkeleton, CardSkeleton } from '../components/LoadingSkeleton';
import { EmptyState } from '../components/EmptyState';

export default function PublicDashboard() {
  const { data: status, isLoading: statusLoading } = usePublicStatus();
  const { data: nodes = [], isLoading: nodesLoading } = usePublicNodes();
  const { data: systemMetrics, isLoading: metricsLoading } = usePublicSystemMetrics();

  return (
    <div className="min-h-screen bg-slate-950">
      {/* Header */}
      <header className="bg-slate-900 border-b border-slate-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-blue-500/10 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-blue-500" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-white">NeoNexus</h1>
                <p className="text-xs text-slate-400">Node Status Monitor</p>
              </div>
            </div>
            <Link 
              to="/login" 
              className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg text-sm font-medium transition-colors"
            >
              <Lock className="w-4 h-4" />
              Admin Login
            </Link>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Overview Stats */}
        {statusLoading ? (
          <div className="mb-8"><StatSkeleton /></div>
        ) : status ? (
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-400 text-sm">Total Nodes</p>
                  <p className="text-2xl font-bold text-white">{status.totalNodes}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center">
                  <Server className="w-6 h-6 text-blue-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-400 text-sm">Running</p>
                  <p className="text-2xl font-bold text-white">{status.runningNodes}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-emerald-500/10 flex items-center justify-center">
                  <Play className="w-6 h-6 text-emerald-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-400 text-sm">Blocks Synced</p>
                  <p className="text-2xl font-bold text-white">{status.totalBlocks.toLocaleString()}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-purple-500/10 flex items-center justify-center">
                  <Layers className="w-6 h-6 text-purple-400" />
                </div>
              </div>
            </div>
            <div className="card">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-slate-400 text-sm">Total Peers</p>
                  <p className="text-2xl font-bold text-white">{status.totalPeers}</p>
                </div>
                <div className="w-12 h-12 rounded-lg bg-cyan-500/10 flex items-center justify-center">
                  <Globe className="w-6 h-6 text-cyan-400" />
                </div>
              </div>
            </div>
          </div>
        ) : null}

        {/* System Metrics */}
        {metricsLoading ? (
          <div className="mb-8"><CardSkeleton count={1} /></div>
        ) : systemMetrics ? (
          <div className="card mb-8">
            <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <Cpu className="w-5 h-5 text-blue-400" />
              System Resources
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-400">CPU Usage</span>
                  <span className="text-white">{systemMetrics.cpu.usage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" />
                <p className="text-xs text-slate-500 mt-1">{systemMetrics.cpu.cores} cores</p>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-400">Memory</span>
                  <span className="text-white">{systemMetrics.memory.percentage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" />
                <p className="text-xs text-slate-500 mt-1">
                  {formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}
                </p>
              </div>
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-slate-400">Disk</span>
                  <span className="text-white">{systemMetrics.disk.percentage.toFixed(1)}%</span>
                </div>
                <ProgressBar value={systemMetrics.disk.percentage} color="bg-purple-500" />
                <p className="text-xs text-slate-500 mt-1">
                  {formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}
                </p>
              </div>
            </div>
          </div>
        ) : null}

        {/* Node List */}
        <div className="card">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-lg font-semibold text-white">Node Status</h2>
            <div className="flex items-center gap-2 text-sm text-slate-400">
              <Clock className="w-4 h-4" />
              Auto-refresh every 5s
            </div>
          </div>

          {nodesLoading ? (
            <CardSkeleton count={2} />
          ) : nodes.length === 0 ? (
            <EmptyState icon={Server} title="No nodes configured" description="No Neo nodes are currently managed by this instance" />
          ) : (
            <div className="space-y-4">
              {nodes.map((node) => (
                <div
                  key={node.id}
                  className="p-4 bg-slate-800/50 rounded-lg border border-slate-700/50"
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
                        <h3 className="font-medium text-white">{node.name}</h3>
                        <p className="text-sm text-slate-400">
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
                          <span className="text-slate-400">Block:</span>
                          <span className="text-white font-mono">
                            {node.metrics.blockHeight.toLocaleString()}
                          </span>
                        </div>
                      )}

                      {/* Peers */}
                      {node.metrics && (
                        <div className="flex items-center gap-2 text-sm">
                          <Users className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-400">Peers:</span>
                          <span className="text-white">{node.metrics.connectedPeers}</span>
                        </div>
                      )}

                      {/* Sync Progress */}
                      {node.metrics && node.metrics.syncProgress > 0 && (
                        <div className="flex items-center gap-2 text-sm">
                          <CheckCircle className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-400">Sync:</span>
                          <span className="text-white">{node.metrics.syncProgress.toFixed(1)}%</span>
                        </div>
                      )}

                      {/* Uptime */}
                      {node.uptime && (
                        <div className="flex items-center gap-2 text-sm">
                          <Clock className="w-4 h-4 text-slate-500" />
                          <span className="text-slate-400">Uptime:</span>
                          <span className="text-white">{formatDuration(node.uptime)}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Sync Progress Bar */}
                  {node.metrics && node.metrics.syncProgress > 0 && node.metrics.syncProgress < 100 && (
                    <div className="mt-4">
                      <div className="h-1.5 bg-slate-700 rounded-full overflow-hidden">
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
            <Link to="/login" className="text-blue-400 hover:text-blue-300">
              Login
            </Link>{' '}
            to manage nodes.
          </p>
        </div>
      </main>
    </div>
  );
}
