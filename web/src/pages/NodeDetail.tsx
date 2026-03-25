import { useParams, Link } from 'react-router-dom';
import { 
  ArrowLeft, 
  Play, 
  Square, 
  RotateCw, 
  Trash2,
  
  
  
  
} from 'lucide-react';
import { useNode, useStartNode, useStopNode, useDeleteNode, useNodeLogs } from '../hooks/useNodes';
import { useState, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { formatDistanceToNow } from 'date-fns';

export default function NodeDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: node } = useNode(id!);
  const { data: logs = [] } = useNodeLogs(id!, 50);
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const [activeTab, setActiveTab] = useState<'overview' | 'logs'>('overview');
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll logs
  useEffect(() => {
    if (activeTab === 'logs') {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, activeTab]);

  if (!node) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  const handleDelete = async () => {
    if (!confirm('Are you sure you want to delete this node? This cannot be undone.')) return;
    await deleteNode.mutateAsync(id!);
    navigate('/nodes');
  };

  const getLogColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'debug': return 'text-slate-500';
      default: return 'text-slate-300';
    }
  };

  return (
    <div className="space-y-6">
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
            >
              <Square className="w-4 h-4" />
              Stop
            </button>
          ) : (
            <button
              onClick={() => startNode.mutate(node.id)}
              disabled={startNode.isPending}
              className="btn btn-success"
            >
              <Play className="w-4 h-4" />
              Start
            </button>
          )}
          <button
            onClick={() => stopNode.mutate({ id: node.id }, { onSuccess: () => startNode.mutate(node.id) })}
            disabled={node.process.status !== 'running'}
            className="btn btn-secondary"
          >
            <RotateCw className="w-4 h-4" />
            Restart
          </button>
          <button
            onClick={handleDelete}
            disabled={deleteNode.isPending}
            className="btn btn-error"
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
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Metrics */}
          <div className="lg:col-span-2 space-y-6">
            {node.metrics && (
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-4">Metrics</h3>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                  <div className="bg-slate-800/50 rounded-lg p-4">
                    <p className="text-slate-400 text-sm">Block Height</p>
                    <p className="text-2xl font-bold text-white">{node.metrics.blockHeight.toLocaleString()}</p>
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
                    <p className="text-2xl font-bold text-white">{(node.metrics.memoryUsage / 1024 / 1024).toFixed(0)} MB</p>
                  </div>
                </div>
              </div>
            )}

            {/* Configuration */}
            <div className="card">
              <h3 className="text-lg font-semibold text-white mb-4">Configuration</h3>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-slate-400">Type</p>
                  <p className="text-white capitalize">{node.type}</p>
                </div>
                <div>
                  <p className="text-slate-400">Network</p>
                  <p className="text-white capitalize">{node.network}</p>
                </div>
                <div>
                  <p className="text-slate-400">Sync Mode</p>
                  <p className="text-white capitalize">{node.syncMode}</p>
                </div>
                <div>
                  <p className="text-slate-400">Version</p>
                  <p className="text-white">{node.version}</p>
                </div>
              </div>
            </div>
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
                    <span className="text-white">{formatDistanceToNow(Date.now() - node.process.uptime)}</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <div className="card">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-white">Logs</h3>
            <span className="text-sm text-slate-400">Last 50 entries</span>
          </div>
          <div className="bg-slate-950 rounded-lg p-4 font-mono text-sm h-96 overflow-y-auto">
            {logs.length === 0 ? (
              <p className="text-slate-500">No logs available</p>
            ) : (
              <div className="space-y-1">
                {logs.map((log, index) => (
                  <div key={index} className="flex gap-3">
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
      )}
    </div>
  );
}
