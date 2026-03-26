import { 
  Activity, 
  Plus, 
  Play, 
  Square, 
  Trash2, 
  
  Search,
  FolderOpen
} from 'lucide-react';
import { useNodes, useStartNode, useStopNode, useDeleteNode, useNodeSignerHealth } from '../hooks/useNodes';
import { Link } from 'react-router-dom';
import { useState } from 'react';
import { getNodeProtectionLabel, signerReadinessColor } from '../utils/signerVisibility';

export default function Nodes() {
  const { data: nodes = [] } = useNodes();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const [searchTerm, setSearchTerm] = useState('');
  const [deleting, setDeleting] = useState<string | null>(null);

  const filteredNodes = nodes.filter(node => 
    node.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    node.type.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this node? This action cannot be undone.')) {
      return;
    }
    setDeleting(id);
    try {
      await deleteNode.mutateAsync(id);
    } finally {
      setDeleting(null);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white">Nodes</h1>
          <p className="text-slate-400 mt-1">Manage your Neo nodes</p>
        </div>
        <div className="flex gap-3">
          <Link to="/nodes/import" className="btn btn-secondary">
            <FolderOpen className="w-4 h-4" />
            Import Existing
          </Link>
          <Link to="/nodes/create" className="btn btn-primary">
            <Plus className="w-4 h-4" />
            Create Node
          </Link>
        </div>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
        <input
          type="text"
          placeholder="Search nodes..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="input pl-10"
        />
      </div>

      {/* Node List */}
      <div className="card">
        {filteredNodes.length === 0 ? (
          <div className="text-center py-12">
            <Activity className="w-12 h-12 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400">
              {searchTerm ? 'No nodes match your search' : 'No nodes yet'}
            </p>
            {!searchTerm && (
              <div className="flex gap-3 justify-center mt-4">
                <Link to="/nodes/import" className="btn btn-secondary inline-flex">
                  <FolderOpen className="w-4 h-4 mr-2" />
                  Import existing node
                </Link>
                <Link to="/nodes/create" className="btn btn-primary inline-flex">
                  <Plus className="w-4 h-4 mr-2" />
                  Create new node
                </Link>
              </div>
            )}
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">Node</th>
                  <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">Type</th>
                  <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">Network</th>
                  <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">Ports</th>
                  <th className="text-left py-3 px-4 text-slate-400 font-medium text-sm">Status</th>
                  <th className="text-right py-3 px-4 text-slate-400 font-medium text-sm">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredNodes.map((node) => (
                  <tr key={node.id} className="border-b border-slate-800/50 hover:bg-slate-800/30">
                    <td className="py-4 px-4">
                      <Link to={`/nodes/${node.id}`} className="flex items-center gap-3">
                        <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${
                          node.type === 'neo-cli' ? 'bg-blue-500/10' : 'bg-emerald-500/10'
                        }`}>
                          <Activity className={`w-4 h-4 ${
                            node.type === 'neo-cli' ? 'text-blue-400' : 'text-emerald-400'
                          }`} />
                        </div>
                        <div>
                          <p className="font-medium text-white">{node.name}</p>
                          <div className="mt-1 flex flex-wrap items-center gap-2">
                            <p className="text-xs text-slate-500">v{node.version}</p>
                            {(() => {
                              const protection = getNodeProtectionLabel(node);
                              return (
                                <span
                                  className={`inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium ${
                                    protection.tone === 'secure'
                                      ? 'bg-cyan-500/10 text-cyan-300'
                                      : 'bg-slate-700 text-slate-400'
                                  }`}
                                >
                                  {protection.label}
                                </span>
                              );
                            })()}
                          </div>
                        </div>
                      </Link>
                    </td>
                    <td className="py-4 px-4 text-slate-300 capitalize">{node.type}</td>
                    <td className="py-4 px-4 text-slate-300 capitalize">{node.network}</td>
                    <td className="py-4 px-4 text-slate-300 text-sm">
                      RPC: {node.ports.rpc}
                      <br />
                      P2P: {node.ports.p2p}
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex flex-col gap-2">
                        <span className={`status-badge status-${node.process.status}`}>
                          {node.process.status === 'running' && (
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                          )}
                          {node.process.status}
                        </span>
                        {node.settings?.keyProtection?.mode === 'secure-signer' && (
                          <NodeSignerStatus nodeId={node.id} />
                        )}
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex items-center justify-end gap-2">
                        {node.process.status === 'running' ? (
                          <button
                            onClick={() => stopNode.mutate({ id: node.id })}
                            disabled={stopNode.isPending}
                            className="btn btn-secondary p-2"
                            title="Stop"
                          >
                            <Square className="w-4 h-4" />
                          </button>
                        ) : (
                          <button
                            onClick={() => startNode.mutate(node.id)}
                            disabled={startNode.isPending || node.process.status === 'starting'}
                            className="btn btn-success p-2"
                            title="Start"
                          >
                            <Play className="w-4 h-4" />
                          </button>
                        )}
                        <button
                          onClick={() => handleDelete(node.id)}
                          disabled={deleting === node.id}
                          className="btn btn-error p-2"
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function NodeSignerStatus({ nodeId }: { nodeId: string }) {
  const { data: signerHealth } = useNodeSignerHealth(nodeId);

  if (!signerHealth) {
    return null;
  }

  const tone = signerReadinessColor(signerHealth.readiness.status);

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ${tone}`}>
      {signerHealth.readiness.status}
      {signerHealth.readiness.accountStatus ? ` · ${signerHealth.readiness.accountStatus}` : ''}
    </span>
  );
}
