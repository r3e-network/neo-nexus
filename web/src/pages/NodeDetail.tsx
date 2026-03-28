import { useParams, Link, useNavigate } from 'react-router-dom';
import { ArrowLeft, Play, Square, RotateCw, Trash2, Loader2 } from 'lucide-react';
import { useNode, useStartNode, useStopNode, useDeleteNode, useNodeSignerHealth, useNetworkHeight } from '../hooks/useNodes';
import { useState, useEffect } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { mergeNodeLogs } from '../utils/realtime';
import { formatBytes, formatDuration } from '../utils/format';
import { NodeConfigEditor } from './node-detail/NodeConfigEditor';
import { NodeLogsView } from './node-detail/NodeLogsView';

export default function NodeDetail() {
  const { id = '' } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: node } = useNode(id);
  const { data: signerHealth } = useNodeSignerHealth(id);
  const { connected, lastMessage } = useWebSocket();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const [activeTab, setActiveTab] = useState<'overview' | 'logs'>('overview');
  const [realtimeLogs, setRealtimeLogs] = useState<Array<{ timestamp: number; level: string; message: string }>>([]);
  const [isRestarting, setIsRestarting] = useState(false);
  const networkHeightQuery = useNetworkHeight();

  useEffect(() => {
    if (!id || typeof lastMessage === 'string' || !lastMessage) {
      return;
    }

    if (lastMessage.type === 'log' && lastMessage.nodeId === id) {
      const entry = lastMessage.data as { timestamp: number; level: string; message: string };
      setRealtimeLogs((current) => mergeNodeLogs(current, [entry]));
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

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div className="flex items-center gap-4">
          <Link to="/nodes" className="text-slate-400 hover:text-white">
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <div className="flex items-center gap-3">
              <h1 className="text-2xl font-bold text-white">{node.name}</h1>
              <span className={`status-badge status-${node.process.status}`}>
                {node.process.status === 'running' && (
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                )}
                {node.process.status}
              </span>
            </div>
            <p className="text-slate-400 text-sm mt-1">
              {node.type} • {node.network} • v{node.version}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {node.process.status === 'running' ? (
            <button
              onClick={() => stopNode.mutate({ id: node.id })}
              disabled={stopNode.isPending}
              className="btn btn-error"
              aria-label="Stop node"
            >
              <Square className="w-4 h-4" />
              Stop
            </button>
          ) : (
            <button
              onClick={() => startNode.mutate(node.id)}
              disabled={startNode.isPending}
              className="btn btn-success"
              aria-label="Start node"
            >
              <Play className="w-4 h-4" />
              Start
            </button>
          )}
          <button
            onClick={handleRestart}
            disabled={node.process.status !== 'running' || isRestarting}
            className="btn btn-secondary"
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

      {/* Tabs */}
      <div className="border-b border-slate-700">
        <div className="flex gap-6">
          <button
            onClick={() => setActiveTab('overview')}
            className={`pb-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'overview'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-slate-400 hover:text-white'
            }`}
          >
            Overview
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`pb-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'logs'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-slate-400 hover:text-white'
            }`}
          >
            Logs
          </button>
        </div>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-fade-in">
          {/* Metrics + Config */}
          <div className="lg:col-span-2 space-y-6">
            {node.metrics && (
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-4">Metrics</h3>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div className="bg-slate-800/50 rounded-lg p-4">
                    <p className="text-slate-400 text-sm">Block Height</p>
                    <p className="text-2xl font-bold text-white">{node.metrics.blockHeight.toLocaleString()}</p>
                    {(node.network === 'mainnet' || node.network === 'testnet') && (() => {
                      const networkHeight = node.network === 'mainnet'
                        ? networkHeightQuery.data?.mainnet
                        : networkHeightQuery.data?.testnet;
                      if (!networkHeight || networkHeight <= 0) return null;
                      const syncPct = (node.metrics!.blockHeight / networkHeight * 100).toFixed(1);
                      return (
                        <div className="mt-2 space-y-1">
                          <p className="text-xs text-slate-400">
                            Network: {networkHeight.toLocaleString()}
                          </p>
                          <p className="text-xs text-slate-400">Sync: {syncPct}%</p>
                          <div className="h-1 rounded-full bg-slate-700 overflow-hidden">
                            <div
                              className="h-full rounded-full bg-blue-500 transition-all duration-500"
                              style={{ width: `${Math.min(100, parseFloat(syncPct))}%` }}
                            />
                          </div>
                        </div>
                      );
                    })()}
                  </div>
                  <div className="bg-slate-800/50 rounded-lg p-4">
                    <p className="text-slate-400 text-sm">Peers</p>
                    <p className="text-2xl font-bold text-white">{node.metrics.connectedPeers}</p>
                  </div>
                  <div className="bg-slate-800/50 rounded-lg p-4">
                    <p className="text-slate-400 text-sm">CPU</p>
                    <p className="text-2xl font-bold text-white">{node.metrics.cpuUsage.toFixed(1)}%</p>
                  </div>
                  <div className="bg-slate-800/50 rounded-lg p-4">
                    <p className="text-slate-400 text-sm">Memory</p>
                    <p className="text-2xl font-bold text-white">{formatBytes(node.metrics.memoryUsage)}</p>
                  </div>
                </div>
              </div>
            )}

            <NodeConfigEditor node={node} />
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
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
