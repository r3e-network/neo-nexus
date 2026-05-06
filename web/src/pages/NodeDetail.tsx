import { useParams, Link, useNavigate } from 'react-router-dom';
import { ArrowLeft, Play, Square, RotateCw, Trash2, Loader2, ChevronDown, ChevronUp, ShieldCheck, Eye, KeyRound } from 'lucide-react';
import { useNode, useStartNode, useStopNode, useDeleteNode, useNodeSignerHealth, useNetworkHeight, useUpdateNodeOwnership, type ImportedNodeOwnershipMode } from '../hooks/useNodes';
import { useState, useEffect } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { mergeNodeLogs } from '../utils/realtime';
import { formatBytes, formatDuration, formatVersion } from '../utils/format';
import { NodeConfigEditor } from './node-detail/NodeConfigEditor';
import { NodeLogsView } from './node-detail/NodeLogsView';
import { NodeOrchestrationPanel } from './node-detail/NodeOrchestrationPanel';
import { NodeProtectionLabel } from '../components/NodeProtectionLabel';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { ApiRequestError } from '../utils/api';
import {
  blockHeightBadgeClass,
  blockHeightDetailClass,
  blockHeightProgressClass,
  getBlockHeightStatus,
} from '../utils/blockHeightStatus';

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
  const { data: node, error: nodeError, isLoading: nodeLoading, refetch: refetchNode } = useNode(id);
  const { data: signerHealth } = useNodeSignerHealth(id);
  const { connected, lastMessage } = useWebSocket();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const updateOwnership = useUpdateNodeOwnership();
  const [activeTab, setActiveTab] = useState<'overview' | 'logs'>('overview');
  const [realtimeLogs, setRealtimeLogs] = useState<Array<{ timestamp: number; level: string; message: string }>>([]);
  const [isRestarting, setIsRestarting] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
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

  if (nodeLoading) {
    return (
      <div className="space-y-6 animate-fade-in">
        <div className="card flex min-h-72 flex-col items-center justify-center gap-4 text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
          <div>
            <h1 className="text-xl font-semibold text-slate-950">Loading node details</h1>
            <p className="mt-2 text-sm text-slate-600">Fetching the latest lifecycle, metrics, and configuration state.</p>
          </div>
          <Link to="/nodes" className="btn btn-secondary">
            <ArrowLeft className="h-4 w-4" />
            Back to nodes
          </Link>
        </div>
      </div>
    );
  }

  if (!node) {
    const isNotFound = !id || (nodeError instanceof ApiRequestError && nodeError.status === 404);
    const title = isNotFound ? 'Node not found' : 'Unable to load node';
    const description = isNotFound
      ? 'This node no longer exists or the link is invalid.'
      : nodeError instanceof Error
        ? nodeError.message
        : 'NeoNexus could not load this node right now.';

    return (
      <div className="space-y-6 animate-fade-in">
        <div className="card flex min-h-72 flex-col items-center justify-center gap-4 text-center">
          <div className="rounded-lg bg-red-50 p-3 text-red-700">
            <Trash2 className="h-6 w-6" />
          </div>
          <div>
            <h1 className="text-xl font-semibold text-slate-950">{title}</h1>
            <p className="mt-2 max-w-xl text-sm text-slate-600">{description}</p>
          </div>
          <div className="flex flex-col gap-3 sm:flex-row">
            <Link to="/nodes" className="btn btn-secondary">
              <ArrowLeft className="h-4 w-4" />
              Back to nodes
            </Link>
            {!isNotFound && (
              <button type="button" onClick={() => void refetchNode()} className="btn btn-primary">
                <RotateCw className="h-4 w-4" />
                Retry
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }

  const blockHeightStatus = node.metrics ? getBlockHeightStatus(node, networkHeightQuery.data) : null;
  const canControlLifecycle = lifecycleAllowed(node);

  const confirmDelete = async () => {
    await deleteNode.mutateAsync(id);
    setDeleteDialogOpen(false);
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
      <section className="page-hero pb-5">
        <div className="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
          <div className="flex items-start gap-4">
            <Link to="/nodes" className="mt-2 rounded-lg p-2 text-slate-600 transition-colors hover:bg-slate-100 hover:text-slate-950" aria-label="Back to nodes">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <p className="console-kicker">Node detail</p>
              <div className="mt-2 flex flex-wrap items-center gap-3">
                <h1 className="text-3xl font-semibold text-slate-950">{node.name}</h1>
                <span className={`status-badge status-${node.process.status}`}>
                  {node.process.status === 'running' && (
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                  )}
                  {node.process.status}
                </span>
                <NodeProtectionLabel node={node} />
              </div>
              <p className="mt-2 text-sm text-slate-600">
                {node.type} · {node.network} · {formatVersion(node.version)} · {ownershipLabel(node)}
              </p>
              {!canControlLifecycle && (
                <p className="mt-3 inline-flex items-center gap-2 rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
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
              onClick={() => setDeleteDialogOpen(true)}
              disabled={deleteNode.isPending}
              className="btn btn-error"
              aria-label="Delete node"
            >
              <Trash2 className="w-4 h-4" />
              Delete
            </button>
          </div>
        </div>

        <div className="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Block height</p>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{node.metrics?.blockHeight?.toLocaleString() ?? '—'}</p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Peers</p>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{node.metrics?.connectedPeers ?? '—'}</p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Private keys</p>
            <p className="mt-2 inline-flex items-center gap-2 text-sm font-medium text-teal-800">
              <KeyRound className="h-4 w-4" /> {node.settings?.keyProtection?.mode === 'secure-signer' ? 'Secure signer' : 'Local wallet'}
            </p>
          </div>
          <div className="stat-tile">
            <p className="text-xs uppercase tracking-[0.16em] text-slate-500">Ownership</p>
            <p className="mt-2 inline-flex items-center gap-2 text-sm font-medium text-slate-700">
              <ShieldCheck className="h-4 w-4 text-emerald-600" /> {ownershipLabel(node)}
            </p>
          </div>
        </div>
      </section>

      {node.settings?.import && (
        <section className="rounded-lg border border-amber-200 bg-amber-50 p-5">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <p className="console-kicker text-amber-700">Imported native node ownership</p>
              <h2 className="mt-2 text-xl font-semibold text-amber-950">Choose how much control NeoNexus may take</h2>
              <p className="mt-2 max-w-3xl text-sm leading-6 text-amber-900">
                Keep native Neo compatibility by defaulting to observe-only. Upgrade only when you want NeoNexus to write config/plugins or control the native process.
              </p>
              {ownershipError && <p className="mt-3 text-sm text-red-700">{ownershipError}</p>}
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
                    className={`rounded-lg border p-4 text-left transition-colors ${
                      active
                        ? 'border-teal-500 bg-emerald-50 text-slate-950'
                        : 'border-amber-200 bg-white text-slate-700 hover:border-amber-300 hover:bg-amber-50/60'
                    }`}
                  >
                    <p className="font-semibold">{label}</p>
                    <p className="mt-2 text-xs leading-5 text-slate-600">{description}</p>
                    {active && <p className="mt-3 text-xs font-semibold uppercase text-teal-800">Current mode</p>}
                  </button>
                );
              })}
            </div>
          </div>
        </section>
      )}

      {/* Tabs */}
      <div className="flex items-center justify-between gap-3 rounded-lg border border-slate-200 bg-white p-1.5">
        <div className="flex gap-1">
          <button
            onClick={() => setActiveTab('overview')}
            className={`rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === 'overview'
                ? 'bg-teal-50 text-teal-950'
                : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950'
            }`}
          >
            Overview
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === 'logs'
                ? 'bg-teal-50 text-teal-950'
                : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950'
            }`}
          >
            Logs
          </button>
        </div>
        <button
          type="button"
          onClick={toggleDetails}
          className="hidden items-center gap-2 rounded-md px-3 py-2 text-xs font-medium text-slate-600 transition-colors hover:bg-slate-100 hover:text-slate-950 sm:inline-flex"
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
                <h3 className="text-lg font-semibold text-slate-950 mb-4">Metrics</h3>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div className="metric-tile group">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <p className="text-slate-600 text-sm font-medium">Block Height</p>
                      {blockHeightStatus && (
                        <span
                          className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[11px] font-medium ${blockHeightBadgeClass(blockHeightStatus.status)}`}
                          title={blockHeightStatus.detail}
                        >
                          {blockHeightStatus.label}
                        </span>
                      )}
                    </div>
                    <p className="mt-1 text-2xl font-bold text-slate-950">{node.metrics.blockHeight.toLocaleString()}</p>
                    {blockHeightStatus && (
                      <div className="mt-2 space-y-1.5">
                        <p className={`text-xs leading-5 ${blockHeightDetailClass(blockHeightStatus.status)}`}>
                          {blockHeightStatus.detail}
                        </p>
                        {blockHeightStatus.networkHeight !== null && (
                          <p className="text-xs text-slate-600">
                            Latest: {blockHeightStatus.networkHeight.toLocaleString()}
                            {blockHeightStatus.remainingBlocks !== null && blockHeightStatus.remainingBlocks > 0
                              ? ` · Remaining: ${blockHeightStatus.remainingBlocks.toLocaleString()}`
                              : ""}
                          </p>
                        )}
                        <div className="h-1.5 overflow-hidden rounded-full bg-slate-200">
                          <div
                            className={`h-full rounded-full transition-all duration-500 ${blockHeightProgressClass(blockHeightStatus.status)}`}
                            style={{ width: `${blockHeightStatus.progressPercent}%` }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-600 text-sm font-medium">Peers</p>
                    <p className="text-2xl font-bold text-slate-950">{node.metrics.connectedPeers}</p>
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-600 text-sm font-medium">CPU</p>
                    <p className="text-2xl font-bold text-slate-950">{node.metrics.cpuUsage.toFixed(1)}%</p>
                  </div>
                  <div className="metric-tile group">
                    <p className="text-slate-600 text-sm font-medium">Memory</p>
                    <p className="text-2xl font-bold text-slate-950">{formatBytes(node.metrics.memoryUsage)}</p>
                  </div>
                </div>
              </div>
            )}

            <NodeOrchestrationPanel node={node} />

            <NodeConfigEditor node={node} />
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            <button
              type="button"
              onClick={toggleDetails}
              className="flex w-full items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm text-slate-600 transition-colors hover:bg-slate-50 hover:text-slate-950 sm:hidden"
            >
              {showDetails ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
              {showDetails ? "Hide technical details" : "Show technical details"}
            </button>

            {showDetails && (
              <>
                {/* Ports */}
                <div className="card">
                  <h3 className="text-lg font-semibold text-slate-950 mb-4">Ports</h3>
                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-slate-600">RPC</span>
                      <span className="text-slate-950 font-mono">{node.ports.rpc}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-600">P2P</span>
                      <span className="text-slate-950 font-mono">{node.ports.p2p}</span>
                    </div>
                    {node.ports.websocket && (
                      <div className="flex justify-between">
                        <span className="text-slate-600">WebSocket</span>
                        <span className="text-slate-950 font-mono">{node.ports.websocket}</span>
                      </div>
                    )}
                    {node.ports.metrics && (
                      <div className="flex justify-between">
                        <span className="text-slate-600">Metrics</span>
                        <span className="text-slate-950 font-mono">{node.ports.metrics}</span>
                      </div>
                    )}
                  </div>
                </div>

                {/* Process Info */}
                <div className="card">
                  <h3 className="text-lg font-semibold text-slate-950 mb-4">Process</h3>
                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-slate-600">PID</span>
                      <span className="text-slate-950 font-mono">{node.process.pid || 'N/A'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-600">Status</span>
                      <span className="text-slate-950 capitalize">{node.process.status}</span>
                    </div>
                    {node.process.uptime && (
                      <div className="flex justify-between">
                        <span className="text-slate-600">Uptime</span>
                        <span className="text-slate-950">{formatDuration(node.process.uptime)}</span>
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}

            {signerHealth && (
              <div className="card">
                <h3 className="text-lg font-semibold text-slate-950 mb-4">Secure Signer</h3>
                <div className="space-y-3 text-sm">
                  <div>
                    <p className="text-slate-600">Profile</p>
                    <p className="text-slate-950">{signerHealth.profile.name}</p>
                  </div>
                  <div>
                    <p className="text-slate-600">Mode</p>
                    <p className="text-slate-950 capitalize">{signerHealth.profile.mode}</p>
                  </div>
                  <div>
                    <p className="text-slate-600">Endpoint</p>
                    <p className="text-slate-950 break-words">{signerHealth.profile.endpoint || 'Restricted'}</p>
                  </div>
                  <div>
                    <p className="text-slate-600">Readiness</p>
                    <p
                      className={
                        signerHealth.readiness.status === 'reachable'
                          ? 'text-emerald-700'
                          : signerHealth.readiness.status === 'warning'
                            ? 'text-amber-700'
                            : 'text-red-700'
                      }
                    >
                      {signerHealth.readiness.status}
                      {signerHealth.readiness.accountStatus ? ` · ${signerHealth.readiness.accountStatus}` : ''}
                    </p>
                  </div>
                  <div>
                    <p className="text-slate-600">Source</p>
                    <p className="text-slate-950">{signerHealth.readiness.source}</p>
                  </div>
                  <div>
                    <p className="text-slate-600">Details</p>
                    <p className="text-slate-700">{signerHealth.readiness.message}</p>
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

      <ConfirmDialog
        open={deleteDialogOpen}
        title="Delete node?"
        description="This removes the node registration. Managed files are removed only when NeoNexus owns the node directory."
        confirmLabel="Delete node"
        isConfirming={deleteNode.isPending}
        onCancel={() => setDeleteDialogOpen(false)}
        onConfirm={() => void confirmDelete()}
      />
    </div>
  );
}
