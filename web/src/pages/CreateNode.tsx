import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCreateNode } from '../hooks/useNodes';
import { ArrowLeft, Server, Loader2 } from 'lucide-react';
import { Link } from 'react-router-dom';

const NODE_TYPES = [
  { value: 'neo-cli', label: 'Neo CLI (C#)', description: 'Official Neo implementation with plugin support' },
  { value: 'neo-go', label: 'Neo Go', description: 'High-performance Go implementation' },
];

const NETWORKS = [
  { value: 'mainnet', label: 'Mainnet', description: 'Production Neo N3 network' },
  { value: 'testnet', label: 'Testnet', description: 'Test network for development' },
  { value: 'private', label: 'Private', description: 'Local private network' },
];

const SYNC_MODES = [
  { value: 'full', label: 'Full Sync', description: 'Download and verify entire blockchain' },
  { value: 'light', label: 'Light Sync', description: 'Fast sync with partial verification' },
];

export default function CreateNode() {
  const navigate = useNavigate();
  const createNode = useCreateNode();
  
  const [formData, setFormData] = useState({
    name: '',
    type: 'neo-go' as 'neo-cli' | 'neo-go',
    network: 'mainnet' as 'mainnet' | 'testnet' | 'private',
    syncMode: 'full' as 'full' | 'light',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      alert('Please enter a node name');
      return;
    }

    try {
      await createNode.mutateAsync({
        name: formData.name.trim(),
        type: formData.type,
        network: formData.network,
        syncMode: formData.syncMode,
      });
      navigate('/nodes');
    } catch (error) {
      alert(error instanceof Error ? error.message : 'Failed to create node');
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="mb-6">
        <Link to="/nodes" className="text-slate-400 hover:text-white flex items-center gap-2 text-sm">
          <ArrowLeft className="w-4 h-4" />
          Back to Nodes
        </Link>
      </div>

      <div className="card">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
            <Server className="w-5 h-5 text-blue-400" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">Create Node</h1>
            <p className="text-slate-400 text-sm">Configure your new Neo node</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Node Name */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Node Name
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="e.g., My Mainnet Node"
              className="input"
              required
            />
          </div>

          {/* Node Type */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Node Type
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {NODE_TYPES.map((type) => (
                <label
                  key={type.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all ${
                    formData.type === type.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-700 hover:border-slate-600'
                  }`}
                >
                  <input
                    type="radio"
                    name="type"
                    value={type.value}
                    checked={formData.type === type.value}
                    onChange={(e) => setFormData({ ...formData, type: e.target.value as 'neo-cli' | 'neo-go' })}
                    className="sr-only"
                  />
                  <p className="font-medium text-white">{type.label}</p>
                  <p className="text-sm text-slate-400 mt-1">{type.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Network */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Network
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {NETWORKS.map((network) => (
                <label
                  key={network.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all ${
                    formData.network === network.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-700 hover:border-slate-600'
                  }`}
                >
                  <input
                    type="radio"
                    name="network"
                    value={network.value}
                    checked={formData.network === network.value}
                    onChange={(e) => setFormData({ ...formData, network: e.target.value as 'mainnet' | 'testnet' | 'private' })}
                    className="sr-only"
                  />
                  <p className="font-medium text-white">{network.label}</p>
                  <p className="text-xs text-slate-400 mt-1">{network.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Sync Mode */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Sync Mode
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {SYNC_MODES.map((mode) => (
                <label
                  key={mode.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all ${
                    formData.syncMode === mode.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-700 hover:border-slate-600'
                  }`}
                >
                  <input
                    type="radio"
                    name="syncMode"
                    value={mode.value}
                    checked={formData.syncMode === mode.value}
                    onChange={(e) => setFormData({ ...formData, syncMode: e.target.value as 'full' | 'light' })}
                    className="sr-only"
                  />
                  <p className="font-medium text-white">{mode.label}</p>
                  <p className="text-sm text-slate-400 mt-1">{mode.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Info Box */}
          <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
            <p className="text-sm text-blue-300">
              <strong>Note:</strong> The node will be created with automatically assigned ports. 
              Ports will be allocated starting from 10332 (RPC) and 10333 (P2P).
            </p>
          </div>

          {/* Submit */}
          <div className="flex gap-4">
            <Link to="/nodes" className="btn btn-secondary flex-1 justify-center">
              Cancel
            </Link>
            <button
              type="submit"
              disabled={createNode.isPending}
              className="btn btn-primary flex-1 justify-center"
            >
              {createNode.isPending ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Creating...
                </>
              ) : (
                'Create Node'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
