import { useParams, Link, useNavigate } from 'react-router-dom';
import { ArrowLeft, Play, Square, RotateCw, Trash2, Loader2, ChevronDown, ChevronUp, ShieldCheck, Eye, KeyRound } from 'lucide-react';
import { useNode, useStartNode, useStopNode, useDeleteNode, useNodeSignerHealth, useNetworkHeight, useUpdateNodeOwnership, type ImportedNodeOwnershipMode } from '../hooks/useNodes';
import { useState, useEffect } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { mergeNodeLogs } from '../utils/realtime';
import { formatBytes, formatDuration } from '../utils/format';
import { NodeConfigEditor } from './node-detail/NodeConfigEditor';
import { NodeLogsView } from './node-detail/NodeLogsView';
import { NodeProtectionLabel } from '../components/NodeProtectionLabel';

function ownershipLabel(node: NonNullable<ReturnType<typeof useNode>['data']>) {
  if (!node.settings?.import) return 'NeoNexus managed';
  const mode = node.settings.import.ownershipMode ?? 'observe-only';
  if (mode === 'managed-process') return 'Imported · process managed';
  if (mode === 'managed-config') return 'Imported · config managed';
  return 'Imported · observe only';
}

function lifecycleAllowed(node: NonNullable<ReturnType<typeof useNode>['data']>) {
  return !node.settings?.import || node.settings.import.ownershipMode === 'managed-process';
}

export default function NodeDetail() {
  const { id = '' } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: node } = useNode(id);
  const { data: signerHealth } = useNodeSignerHealth(id);
  const { connected, lastMessage } = useWebSocket();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const updateOwnership = useUpdateNodeOwnership();
  const [activeTab, setActiveTab] = useState<'overview' | 'logs'>('overview');
  const [realtimeLogs, setRealtimeLogs] = useState<Array<{ timestamp: number; level: string; message: string }>>([]);
  const [isRestarting, setIsRestarting] = useState(false);
  const networkHeightQuery = useNetworkHeight();
  const [showDetails, setShowDetails] = useState(() => {
    return localStorage.getItem("neo-nexus-show-node-details") === "true";
  });
  const [ownershipError, setOwnershipError] = useState('');

  const toggleDetails = () => {
    setShowDetails((prev) => {
      localStorage.setItem("neo-nexus-show-node-details", String(!prev));
      return !prev;
    });
  };

  useEffect(() => {
    if (!id || typeof lastMessage === 'string' || !lastMessage) {
      return;
    }

    if (lastMessage.type === 'log' && lastMessage.nodeId === id) {
      const entry = lastMessage.data as { timestamp: number; level: string; message: string };
      setRealtimeLogs((current) => {
        const merged = mergeNodeLogs(current, [entry]);
        // Cap buffer to prevent unbounded memory growth
        return merged.length > 1000 ? merged.slice(-1000) : merged;
      });
    }
  }, [id, lastMessage]);

  useEffect(() => {
    setRealtimeLogs([]);
  }, [id]);

  if (!node) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  const canControlLifecycle = lifecycleAllowed(node);

  const handleDelete = async () => {
    if (!confirm('Are you sure you want to delete this node? This cannot be undone.')) return;
    await deleteNode.mutateAsync(id);
    navigate('/nodes');
  };

  const handleRestart = async () => {
    setIsRestarting(true);
    try {
      await stopNode.mutateAsync({ id: node.id });
      await startNode.mutateAsync(node.id);
    } catch {
      // errors are surfaced via mutation state; just ensure we reset
    } finally {
      setIsRestarting(false);
    }
  };

  const handleOwnershipChange = async (ownershipMode: ImportedNodeOwnershipMode) => {
    setOwnershipError('');
    try {
      await updateOwnership.mutateAsync({ id: node.id, ownershipMode });
    } catch (error) {
      setOwnershipError(error instanceof Error ? error.message : 'Failed to update ownership mode.');
    }
  };

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero p-7 lg:p-8">
        <div className="relative z-10 flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
          <div className="flex items-start gap-4">
            <Link to="/nodes" className="mt-2 rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-800 hover:text-white" aria-label="Back to nodes">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <p className="console-kicker">Node detail</p>
              <div className="mt-2 flex flex-wrap items-center gap-3">
                <h1 className="text-3xl font-semibold tracking-tight text-white">{node.name}</h1>
                <span className={`status-badge status-${node.process.status}`}>
                  {node.process.status === 'running' && (
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                  )}
                  {node.process.status}
                </span>
                <NodeProtectionLabel node={node} />
              </div>
              <p className="mt-2 text-sm text-slate-400">
                {node.type} · {node.network} · v{node.version} · {ownershipLabel(node)}
              </p>
              {!canControlLifecycle && (
                <p className="mt-3 inline-flex items-center gap-2 rounded-lg border border-amber-400/20 bg-amber-500/10 px-3 py-2 text-sm text-amber-100">
                  <Eye className="h-4 w-4" /> Lifecycle actions are locked by imported ownership mode.
                </p>
              )}
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {node.process.status === 'running' ? (
              <button
                onClick={() => stopNode.mutate({ id: node.id })}
                disabled={stopNode.isPending || !canControlLifecycle}
                className="btn btn-error"
                title={canControlLifecycle ? 'Stop node' : 'Lifecycle locked by imported ownership mode'}
                aria-label="Stop node"
              >
                <Square className="w-4 h-4" />
                Stop
              </button>
            ) : (
              <button
                onClick={() => startNode.mutate(node.id)}
                disabled={startNode.isPending || !canControlLifecycle}
                className="btn btn-success"
                title={canControlLifecycle ? 'Start node' : 'Lifecycle locked by imported ownership mode'}
                aria-label="Start node"
              >
                <Play className="w-4 h-4" />
                Start
              </button>
            )}
            <button
              onClick={handleRestart}
              disabled={node.process.status !== 'running' || isRestarting || !canControlLifecycle}
              className="btn btn-secondary"
              title={canControlLifecycle ? 'Restart node' : 'Lifecycle locked by imported ownership mode'}
              aria-label="Restart node"
            >
              {isRestarting ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Restarting...
                </>
              ) : (
                <>
                  <RotateCw className="w-4 h-4" />
                  Restart
                </>
              )}
            </button>
            <button
              onClick={handleDelete}
              disabled={deleteNode.isPending}
              className="btn btn-error"
              aria-label="Delete node"
            >
              <Trash2 className="w-4 h-4" />
              Delete
            </button>
          </div>
        </div>

        <div className="relative z-10 mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Block height</p>
            <p className="mt-2 text-2xl font-semibold text-white">{node.metrics?.blockHeight?.toLocaleString() ?? '—'}</p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Peers</p>
            <p className="mt-2 text-2xl font-semibold text-white">{node.metrics?.connectedPeers ?? '—'}</p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Private keys</p>
            <p className="mt-2 inline-flex items-center gap-2 text-sm font-medium text-cyan-200">
              <KeyRound className="h-4 w-4" /> {node.settings?.keyProtection?.mode === 'secure-signer' ? 'Secure signer' : 'Local wallet'}
            </p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Ownership</p>
            <p className="mt-2 inline-flex items-center gap-2 text-sm font-medium text-slate-200">
              <ShieldCheck className="h-4 w-4 text-emerald-300" /> {ownershipLabel(node)}
            </p>
          </div>
        </div>
      </section>

      {node.settings?.import && (
        <section className="rounded-[1.25rem] border border-amber-300/20 bg-[linear-gradient(135deg,rgba(245,158,11,0.10),rgba(255,255,255,0.025))] p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.035)]">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <p className="console-kicker text-amber-200">Imported native node ownership</p>
              <h2 className="mt-2 text-xl font-semibold text-white">Choose how much control NeoNexus may take</h2>
              <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-300">
                Keep native Neo compatibility by defaulting to observe-only. Upgrade only when you want NeoNexus to write config/plugins or control the native process.
              </p>
              {ownershipError && <p className="mt-3 text-sm text-red-300">{ownershipError}</p>}
            </div>
            <div className="grid min-w-[min(100%,42rem)] gap-3 sm:grid-cols-3">
              {([
                ['observe-only', 'Observe only', 'Read metrics/logs; never writes config, plugins, or process state.'],
                ['managed-config', 'Managed config', 'Allow config and plugin changes; process lifecycle remains external.'],
                ['managed-process', 'Managed process', 'Allow start/stop/restart after PID/path safety validation.'],
              ] as Array<[ImportedNodeOwnershipMode, string, string]>).map(([mode, label, description]) => {
                const active = (node.settings?.import?.ownershipMode ?? 'observe-only') === mode;
                return (
                  <button
                    key={mode}
                    type="button"
                    onClick={() => handleOwnershipChange(mode)}
                    disabled={active || updateOwnership.isPending}
                    className={`rounded-xl border p-4 text-left transition-all ${
                      active
                        ? 'border-indigo-300/40 bg-indigo-400/15 text-white shadow-[0_18px_40px_-30px_rgba(113,112,255,0.9)]'
                        : 'border-white/[0.08] bg-white/[0.03] text-slate-300 hover:border-white/[0.16] hover:bg-white/[0.055]'
                    }`}
                  >
                    <p className="font-semibold">{label}</p>
                    <p className="mt-2 text-xs leading-5 text-slate-400">{description}</p>
                    {active && <p className="mt-3 text-xs font-semibold uppercase tracking-[0.16em] text-blue-200">Current mode</p>}
                  </button>
                );
              })}
            </div>
          </div>
        </section>
      )}

      {/* Tabs */}
      <div className="flex items-center justify-between gap-3 rounded-2xl border border-white/[0.07] bg-white/[0.025] p-1.5">
        <div className="flex gap-1">
          <button
            onClick={() => setActiveTab('overview')}
            className={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === 'overview'
                ? 'bg-white/[0.075] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
                : 'text-slate-400 hover:bg-white/[0.04] hover:text-white'
            }`}
          >
            Overview
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === 'logs'
                ? 'bg-white/[0.075] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
                : 'text-slate-400 hover:bg-white/[0.04] hover:text-white'
            }`}
          >
            Logs
          </button>
        </div>
        <button
          type="button"
          onClick={toggleDetails}
          className="hidden items-center gap-2 rounded-xl px-3 py-2 text-xs font-medium text-slate-400 transition-colors hover:bg-white/[0.04] hover:text-slate-200 sm:inline-flex"
        >
          {showDetails ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          {showDetails ? "Hide technical details" : "Show technical details"}
        </button>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className={`grid grid-cols-1 gap-6 animate-fade-in ${showDetails || signerHealth ? 'lg:grid-cols-3' : ''}`}>
          {/* Metrics + Config */}
          <div className={`${showDetails || signerHealth ? 'lg:col-span-2' : ''} space-y-6`}>
            {node.metrics && (
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-4">Metrics</h3>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div className="metric-tile group">
                    <p className="text-slate-400 text-sm font-medium group-hover:text-slate-300 transition-colors">Block Height</p>
                    <p className="text-2xl font-bold text-white group-hover:text-blue-50 transition-colors">{node.metrics.blockHeight.toLocaleString()}</p>
                    {(node.network === 'mainnet' || node.network === 'testnet') && (() => {
                      const networkHeight = node.network === 'mainnet'
                        ? networkHeightQuery.data?.mainnet
                        : networkHeightQuery.data?.testnet;
                      if (!networkHeight || networkHeight <= 0) return null;
                      const syncPct = (node.metrics!.blockHeight / networkHeight * 100).toFixed(1);
                      return (
                        <div className="mt-2 space-y-1">
                          <p className="text-xs text-slate-400 group-hover:text-slate-300 transition-colors">
                            Network: {networkHeight.toLocaleString()}
                          </p>
                          <p className="text-xs text-slate-400 group-hover:text-slate-300 transition-colors">Sync: {syncPct}%</p>
                          <div className="h-1 rounded-full bg-slate-700/50 overflow-hidden">
                            <div
                              className="h-full rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)] transition-all duration-500"
                              style={{ width: `${Math.min(100, parseFloat(syncPct))}%` }}
                            />
                          </div>
                        </div>
                      );
                    })()}
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-400 text-sm font-medium group-hover:text-slate-300 transition-colors">Peers</p>
                    <p className="text-2xl font-bold text-white group-hover:text-blue-50 transition-colors">{node.metrics.connectedPeers}</p>
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-400 text-sm font-medium group-hover:text-slate-300 transition-colors">CPU</p>
                    <p className="text-2xl font-bold text-white group-hover:text-blue-50 transition-colors">{node.metrics.cpuUsage.toFixed(1)}%</p>
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-400 text-sm font-medium group-hover:text-slate-300 transition-colors">Memory</p>
                    <p className="text-2xl font-bold text-white group-hover:text-blue-50 transition-colors">{formatBytes(node.metrics.memoryUsage)}</p>
                  </div>
                </div>
              </div>
            )}

            <NodeConfigEditor node={node} />
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            <button
              type="button"
              onClick={toggleDetails}
              className="flex w-full items-center gap-2 rounded-xl border border-white/[0.07] bg-white/[0.025] px-3 py-2 text-sm text-slate-400 transition-colors hover:bg-white/[0.045] hover:text-slate-200 sm:hidden"
            >
              {showDetails ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
              {showDetails ? "Hide technical details" : "Show technical details"}
            </button>

            {showDetails && (
              <>
                {/* Ports */}
                <div className="card">
                  <h3 className="text-lg font-semibold text-white mb-4">Ports</h3>
                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-slate-400">RPC</span>
                      <span className="text-white font-mono">{node.ports.rpc}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">P2P</span>
                      <span className="text-white font-mono">{node.ports.p2p}</span>
                    </div>
                    {node.ports.websocket && (
                      <div className="flex justify-between">
                        <span className="text-slate-400">WebSocket</span>
                        <span className="text-white font-mono">{node.ports.websocket}</span>
                      </div>
                    )}
                    {node.ports.metrics && (
                      <div className="flex justify-between">
                        <span className="text-slate-400">Metrics</span>
                        <span className="text-white font-mono">{node.ports.metrics}</span>
                      </div>
                    )}
                  </div>
                </div>

                {/* Process Info */}
                <div className="card">
                  <h3 className="text-lg font-semibold text-white mb-4">Process</h3>
                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-slate-400">PID</span>
                      <span className="text-white font-mono">{node.process.pid || 'N/A'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Status</span>
                      <span className="text-white capitalize">{node.process.status}</span>
                    </div>
                    {node.process.uptime && (
                      <div className="flex justify-between">
                        <span className="text-slate-400">Uptime</span>
                        <span className="text-white">{formatDuration(node.process.uptime)}</span>
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}

            {signerHealth && (
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-4">Secure Signer</h3>
                <div className="space-y-3 text-sm">
                  <div>
                    <p className="text-slate-400">Profile</p>
                    <p className="text-white">{signerHealth.profile.name}</p>
                  </div>
                  <div>
                    <p className="text-slate-400">Mode</p>
                    <p className="text-white capitalize">{signerHealth.profile.mode}</p>
                  </div>
                  <div>
                    <p className="text-slate-400">Endpoint</p>
                    <p className="text-white break-all">{signerHealth.profile.endpoint}</p>
                  </div>
                  <div>
                    <p className="text-slate-400">Readiness</p>
                    <p
                      className={
                        signerHealth.readiness.status === 'reachable'
                          ? 'text-emerald-300'
                          : signerHealth.readiness.status === 'warning'
                            ? 'text-amber-300'
                            : 'text-red-300'
                      }
                    >
                      {signerHealth.readiness.status}
                      {signerHealth.readiness.accountStatus ? ` · ${signerHealth.readiness.accountStatus}` : ''}
                    </p>
                  </div>
                  <div>
                    <p className="text-slate-400">Source</p>
                    <p className="text-white">{signerHealth.readiness.source}</p>
                  </div>
                  <div>
                    <p className="text-slate-400">Details</p>
                    <p className="text-slate-300">{signerHealth.readiness.message}</p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <NodeLogsView nodeId={id} realtimeLogs={realtimeLogs} connected={connected} />
      )}
    </div>
  );
}
