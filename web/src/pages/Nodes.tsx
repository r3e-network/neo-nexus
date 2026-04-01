import {
  Activity,
  Plus,
  Play,
  Square,
  Trash2,
  Server,
  Search,
  FolderOpen
} from 'lucide-react';
import { useNodes, useStartNode, useStopNode, useDeleteNode } from '../hooks/useNodes';
import { Link } from 'react-router-dom';
import { useState } from 'react';
import { SignerStatus } from '../components/SignerStatus';
import { NodeProtectionLabel } from '../components/NodeProtectionLabel';
import { TableRowSkeleton } from '../components/LoadingSkeleton';
import { EmptyState } from '../components/EmptyState';
import { SpinnerButton } from '../components/SpinnerButton';

export default function Nodes() {
  const { data: nodes = [], isLoading } = useNodes();
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
    <div className="space-y-6 animate-fade-in">
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
        {isLoading ? (
          <TableRowSkeleton rows={4} />
        ) : filteredNodes.length === 0 ? (
          searchTerm ? (
            <div className="text-center py-12 animate-fade-in">
              <Activity className="w-12 h-12 text-slate-600 mx-auto mb-4" />
              <p className="text-slate-400">No nodes match your search</p>
            </div>
          ) : (
            <EmptyState
              icon={Server}
              title="No nodes configured"
              description="Create or import a Neo node to get started"
              action={{ label: "Create Node", href: "/nodes/create" }}
            />
          )
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
                {filteredNodes.map((node, i) => (
                  <tr key={node.id} className={`border-b border-slate-700/30 bg-slate-800/10 backdrop-blur-sm hover:bg-slate-800/30 transition-colors duration-200 animate-fade-in-up`} style={{ animationDelay: `${i * 0.05}s` }}>
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
                            <NodeProtectionLabel node={node} padding="px-2 py-0.5" />
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
                          <SignerStatus nodeId={node.id} textSize="text-xs" />
                        )}
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex items-center justify-end gap-2">
                        {node.process.status === 'running' ? (
                          <SpinnerButton
                            onClick={() => stopNode.mutate({ id: node.id })}
                            loading={stopNode.isPending}
                            className="btn btn-secondary p-2"
                            title="Stop"
                            aria-label="Stop node"
                          >
                            <Square className="w-4 h-4" />
                          </SpinnerButton>
                        ) : (
                          <SpinnerButton
                            onClick={() => startNode.mutate(node.id)}
                            loading={startNode.isPending}
                            disabled={node.process.status === 'starting'}
                            className="btn btn-success p-2"
                            title="Start"
                            aria-label="Start node"
                          >
                            <Play className="w-4 h-4" />
                          </SpinnerButton>
                        )}
                        <SpinnerButton
                          onClick={() => handleDelete(node.id)}
                          loading={deleting === node.id}
                          className="btn btn-error p-2"
                          title="Delete"
                          aria-label="Delete node"
                        >
                          <Trash2 className="w-4 h-4" />
                        </SpinnerButton>
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
