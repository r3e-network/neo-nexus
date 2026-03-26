import { useParams, Link, useNavigate } from 'react-router-dom';
import { ArrowLeft, Play, Square, RotateCw, Trash2, Pencil, Save, X } from 'lucide-react';
import { useNode, useStartNode, useStopNode, useDeleteNode, useNodeLogs, useNodeSignerHealth, useUpdateNode } from '../hooks/useNodes';
import { useState, useRef, useEffect } from 'react';
import { normalizeNodeUpsertPayload, toNodeFormValues } from '../utils/nodePayloads';
import { useWebSocket } from '../hooks/useWebSocket';
import { mergeNodeLogs } from '../utils/realtime';
import { useSecureSigners } from '../hooks/useSecureSigners';
import { FeedbackBanner } from '../components/FeedbackBanner';
import { formatBytes, formatDuration } from '../utils/format';

export default function NodeDetail() {
  const { id = '' } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: node } = useNode(id);
  const { data: signerHealth } = useNodeSignerHealth(id);
  const { data: logs = [] } = useNodeLogs(id, 50);
  const { connected, lastMessage } = useWebSocket();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const updateNode = useUpdateNode();
  const secureSigners = useSecureSigners();
  const [activeTab, setActiveTab] = useState<'overview' | 'logs'>('overview');
  const [isEditingConfig, setIsEditingConfig] = useState(false);
  const [configError, setConfigError] = useState('');
  const [configSuccess, setConfigSuccess] = useState('');
  const [realtimeLogs, setRealtimeLogs] = useState<Array<{ timestamp: number; level: string; message: string }>>([]);
  const [formData, setFormData] = useState(() =>
    toNodeFormValues({
      name: '',
    }),
  );
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll logs
  useEffect(() => {
    if (activeTab === 'logs') {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, realtimeLogs, activeTab]);

  useEffect(() => {
    if (!node || isEditingConfig) {
      return;
    }

    setFormData(toNodeFormValues(node));
  }, [node, isEditingConfig]);

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

  const getLogColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'debug': return 'text-slate-500';
      default: return 'text-slate-300';
    }
  };

  const handleSaveConfig = async () => {
    setConfigError('');
    setConfigSuccess('');

    if (!node) {
      return;
    }

    if (node.process.status === 'running') {
      setConfigError('Stop the node before editing its configuration.');
      return;
    }

    if (!formData.name.trim()) {
      setConfigError('Node name is required.');
      return;
    }

    try {
      await updateNode.mutateAsync({
        id: node.id,
        payload: normalizeNodeUpsertPayload({
          name: formData.name,
          maxConnections: formData.maxConnections,
          minPeers: formData.minPeers,
          maxPeers: formData.maxPeers,
          relay: formData.relay,
          debugMode: formData.debugMode,
          customConfig: formData.customConfig,
        }),
      });
      setIsEditingConfig(false);
      setConfigSuccess('Configuration updated successfully.');
    } catch (error) {
      setConfigError(error instanceof Error ? error.message : 'Failed to update configuration.');
    }
  };

  const displayedLogs = mergeNodeLogs(logs, realtimeLogs);

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
                    <p className="text-2xl font-bold text-white">{formatBytes(node.metrics.memoryUsage)}</p>
                  </div>
                </div>
              </div>
            )}

            {/* Configuration */}
            <div className="card">
              <div className="flex items-center justify-between gap-4 mb-4">
                <h3 className="text-lg font-semibold text-white">Configuration</h3>
                {isEditingConfig ? (
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => {
                        setIsEditingConfig(false);
                        setConfigError('');
                        setConfigSuccess('');
                        setFormData(toNodeFormValues(node));
                      }}
                      className="btn btn-secondary"
                    >
                      <X className="w-4 h-4" />
                      Cancel
                    </button>
                    <button
                      onClick={handleSaveConfig}
                      disabled={updateNode.isPending}
                      className="btn btn-primary"
                    >
                      <Save className="w-4 h-4" />
                      {updateNode.isPending ? 'Saving...' : 'Save'}
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={() => {
                      setConfigError('');
                      setConfigSuccess('');
                      setFormData(toNodeFormValues(node));
                      setIsEditingConfig(true);
                    }}
                    disabled={node.process.status === 'running'}
                    className="btn btn-secondary"
                    title={node.process.status === 'running' ? 'Stop the node to edit configuration' : 'Edit configuration'}
                  >
                    <Pencil className="w-4 h-4" />
                    Edit
                  </button>
                )}
              </div>

              <FeedbackBanner error={configError} success={configSuccess} />

              {isEditingConfig ? (
                <div className="space-y-4">
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Node Name</label>
                      <input
                        type="text"
                        value={formData.name}
                        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        className="input"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Version</label>
                      <input type="text" value={node.version} disabled className="input opacity-70" />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Type</label>
                      <input type="text" value={node.type} disabled className="input capitalize opacity-70" />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Network</label>
                      <input type="text" value={node.network} disabled className="input capitalize opacity-70" />
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Max Connections</label>
                      <input
                        type="number"
                        min="1"
                        value={formData.maxConnections}
                        onChange={(e) => setFormData({ ...formData, maxConnections: e.target.value })}
                        className="input"
                        placeholder="Optional"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Min Peers</label>
                      <input
                        type="number"
                        min="0"
                        value={formData.minPeers}
                        onChange={(e) => setFormData({ ...formData, minPeers: e.target.value })}
                        className="input"
                        placeholder="Optional"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-slate-400 mb-2">Max Peers</label>
                      <input
                        type="number"
                        min="0"
                        value={formData.maxPeers}
                        onChange={(e) => setFormData({ ...formData, maxPeers: e.target.value })}
                        className="input"
                        placeholder="Optional"
                      />
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <label className="flex items-center justify-between rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3">
                      <div>
                        <p className="text-sm font-medium text-white">Relay Transactions</p>
                        <p className="text-xs text-slate-400">Allow transaction relay</p>
                      </div>
                      <input
                        type="checkbox"
                        checked={formData.relay}
                        onChange={(e) => setFormData({ ...formData, relay: e.target.checked })}
                        className="h-4 w-4"
                      />
                    </label>

                    <label className="flex items-center justify-between rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3">
                      <div>
                        <p className="text-sm font-medium text-white">Debug Mode</p>
                        <p className="text-xs text-slate-400">Enable verbose debug settings</p>
                      </div>
                      <input
                        type="checkbox"
                        checked={formData.debugMode}
                        onChange={(e) => setFormData({ ...formData, debugMode: e.target.checked })}
                        className="h-4 w-4"
                      />
                    </label>
                  </div>

                  {node.type === 'neo-cli' && (
                    <div className="space-y-3 rounded-lg border border-cyan-500/20 bg-cyan-500/5 p-4">
                      <div>
                        <p className="text-sm font-medium text-cyan-200">Private Key Protection</p>
                        <p className="text-xs text-slate-400">
                          Bind this node to a secure signer profile instead of leaving signing to a local plaintext-oriented wallet flow.
                        </p>
                      </div>

                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <label
                          className={`cursor-pointer rounded-lg border-2 p-4 transition-all ${
                            formData.keyProtectionMode === 'standard'
                              ? 'border-cyan-500 bg-cyan-500/10'
                              : 'border-slate-700 hover:border-slate-600'
                          }`}
                        >
                          <input
                            type="radio"
                            name="keyProtectionMode"
                            value="standard"
                            checked={formData.keyProtectionMode === 'standard'}
                            onChange={() => setFormData({ ...formData, keyProtectionMode: 'standard', secureSignerProfileId: '' })}
                            className="sr-only"
                          />
                          <p className="font-medium text-white">Standard Local Wallet</p>
                          <p className="mt-1 text-xs text-slate-400">Keep this node on the regular local wallet path.</p>
                        </label>

                        <label
                          className={`cursor-pointer rounded-lg border-2 p-4 transition-all ${
                            formData.keyProtectionMode === 'secure-signer'
                              ? 'border-cyan-500 bg-cyan-500/10'
                              : 'border-slate-700 hover:border-slate-600'
                          }`}
                        >
                          <input
                            type="radio"
                            name="keyProtectionMode"
                            value="secure-signer"
                            checked={formData.keyProtectionMode === 'secure-signer'}
                            onChange={() => setFormData({ ...formData, keyProtectionMode: 'secure-signer' })}
                            className="sr-only"
                          />
                          <p className="font-medium text-white">Secure Signer / TEE</p>
                          <p className="mt-1 text-xs text-slate-400">Attach a managed signer endpoint and auto-wire SignClient.</p>
                        </label>
                      </div>

                      {formData.keyProtectionMode === 'secure-signer' && (
                        <div className="space-y-2">
                          <label className="block text-xs font-medium text-slate-400">Secure Signer Profile</label>
                          <select
                            value={formData.secureSignerProfileId}
                            onChange={(e) => setFormData({ ...formData, secureSignerProfileId: e.target.value })}
                            className="input"
                          >
                            <option value="">Select a secure signer profile</option>
                            {(secureSigners.data ?? [])
                              .filter((profile) => profile.enabled)
                              .map((profile) => (
                                <option key={profile.id} value={profile.id}>
                                  {profile.name} · {profile.mode} · {profile.endpoint}
                                </option>
                              ))}
                          </select>
                          {(secureSigners.data ?? []).length === 0 && (
                            <p className="text-xs text-amber-300">Create a signer profile in Settings before enabling secure-signer protection.</p>
                          )}
                        </div>
                      )}
                    </div>
                  )}

                  <div>
                    <label className="block text-xs font-medium text-slate-400 mb-2">Custom Config JSON</label>
                    <textarea
                      value={formData.customConfig}
                      onChange={(e) => setFormData({ ...formData, customConfig: e.target.value })}
                      className="input min-h-36 font-mono text-xs"
                      placeholder='Optional JSON, e.g. { "Trace": true }'
                      spellCheck={false}
                    />
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
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

                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-4 text-sm">
                    <div>
                      <p className="text-slate-400">Max Connections</p>
                      <p className="text-white">{node.settings?.maxConnections ?? 'Default'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Min Peers</p>
                      <p className="text-white">{node.settings?.minPeers ?? 'Default'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Max Peers</p>
                      <p className="text-white">{node.settings?.maxPeers ?? 'Default'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Relay</p>
                      <p className="text-white">{node.settings?.relay === false ? 'Disabled' : 'Enabled'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Debug Mode</p>
                      <p className="text-white">{node.settings?.debugMode ? 'Enabled' : 'Disabled'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Custom Config</p>
                      <p className="text-white">{node.settings?.customConfig ? 'Present' : 'None'}</p>
                    </div>
                    <div>
                      <p className="text-slate-400">Key Protection</p>
                      <p className="text-white">
                        {node.settings?.keyProtection?.mode === 'secure-signer' ? 'Secure signer / TEE' : 'Standard local wallet'}
                      </p>
                    </div>
                    <div>
                      <p className="text-slate-400">Signer Profile</p>
                      <p className="text-white">
                        {node.settings?.keyProtection?.signerProfileId || 'None'}
                      </p>
                    </div>
                  </div>
                </div>
              )}
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
        <div className="card">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-white">Logs</h3>
            <span className="text-sm text-slate-400">
              {connected ? 'Live stream connected' : 'Reconnecting live stream'} · Last 50 entries
            </span>
          </div>
          <div className="bg-slate-950 rounded-lg p-4 font-mono text-sm h-96 overflow-y-auto">
            {displayedLogs.length === 0 ? (
              <p className="text-slate-500">No logs available</p>
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
      )}
    </div>
  );
}
