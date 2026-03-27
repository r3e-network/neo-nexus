import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCreateNode } from '../hooks/useNodes';
import { ArrowLeft, Server, Loader2 } from 'lucide-react';
import { FeedbackBanner } from '../components/FeedbackBanner';
import { Link } from 'react-router-dom';
import { normalizeNodeUpsertPayload, toNodeFormValues } from '../utils/nodePayloads';
import { useSecureSigners } from '../hooks/useSecureSigners';

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
  const secureSigners = useSecureSigners();
  
  const [error, setError] = useState('');

  const [formData, setFormData] = useState(() =>
    toNodeFormValues({
      name: '',
      type: 'neo-go',
      network: 'mainnet',
      syncMode: 'full',
      settings: {
        relay: true,
        debugMode: false,
      },
    }),
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!formData.name.trim()) {
      setError('Please enter a node name');
      return;
    }

    try {
      await createNode.mutateAsync(normalizeNodeUpsertPayload(formData));
      navigate('/nodes');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create node');
    }
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
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
          <div className="animate-fade-in-up" style={{ animationDelay: '0.05s' }}>
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
          <div className="animate-fade-in-up" style={{ animationDelay: '0.1s' }}>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Node Type
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {NODE_TYPES.map((type) => (
                <label
                  key={type.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
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
                    onChange={(e) =>
                      setFormData({
                        ...formData,
                        type: e.target.value as 'neo-cli' | 'neo-go',
                        ...(e.target.value === 'neo-go'
                          ? {
                              keyProtectionMode: 'standard',
                              secureSignerProfileId: '',
                            }
                          : {}),
                      })
                    }
                    className="sr-only"
                  />
                  <p className="font-medium text-white">{type.label}</p>
                  <p className="text-sm text-slate-400 mt-1">{type.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Network */}
          <div className="animate-fade-in-up" style={{ animationDelay: '0.15s' }}>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Network
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {NETWORKS.map((network) => (
                <label
                  key={network.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
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
          <div className="animate-fade-in-up" style={{ animationDelay: '0.2s' }}>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Sync Mode
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {SYNC_MODES.map((mode) => (
                <label
                  key={mode.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
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

          <div className="space-y-4 rounded-lg border border-cyan-500/20 bg-cyan-500/5 p-4">
            <div>
              <h2 className="text-sm font-medium text-cyan-200">Private Key Protection</h2>
              <p className="mt-1 text-sm text-slate-400">
                NeoNexus can bind `neo-cli` nodes to a secure signer profile so node configs reference a signing endpoint instead of raw private keys.
              </p>
            </div>

            {formData.type === 'neo-cli' ? (
              <>
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
                    <p className="mt-1 text-sm text-slate-400">Use the node's regular wallet flow without NeoNexus-managed signer binding.</p>
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
                    <p className="mt-1 text-sm text-slate-400">Attach a software, SGX, or Nitro signer profile and auto-wire Neo SignClient.</p>
                  </label>
                </div>

                {formData.keyProtectionMode === 'secure-signer' && (
                  <div className="space-y-3">
                    <div>
                      <label className="block text-sm font-medium text-slate-300 mb-2">Secure Signer Profile</label>
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
                    </div>

                    {(secureSigners.data ?? []).length === 0 && (
                      <p className="text-sm text-amber-300">
                        No secure signer profiles are configured yet. Add one in <Link to="/settings" className="underline">Settings</Link> before enabling TEE-backed signing.
                      </p>
                    )}

                    <p className="text-xs text-slate-500">
                      NeoNexus stores only the signer profile reference and endpoint metadata here. It does not store raw WIF or plaintext unlock material.
                    </p>
                  </div>
                )}
              </>
            ) : (
              <p className="text-sm text-slate-400">
                Secure signer auto-wiring currently targets `neo-cli` nodes via the Neo `SignClient` plugin. `neo-go` remains standard-wallet only for now.
              </p>
            )}
          </div>

          {/* Advanced Settings */}
          <div className="space-y-4">
            <div>
              <h2 className="text-sm font-medium text-slate-300 mb-3">Advanced Settings</h2>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
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
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <label className="flex items-center justify-between rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3">
                <div>
                  <p className="text-sm font-medium text-white">Relay Transactions</p>
                  <p className="text-xs text-slate-400">Allow the node to relay transactions</p>
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
                  <p className="text-xs text-slate-400">Enable verbose debugging where supported</p>
                </div>
                <input
                  type="checkbox"
                  checked={formData.debugMode}
                  onChange={(e) => setFormData({ ...formData, debugMode: e.target.checked })}
                  className="h-4 w-4"
                />
              </label>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">Custom Config JSON</label>
              <textarea
                value={formData.customConfig}
                onChange={(e) => setFormData({ ...formData, customConfig: e.target.value })}
                className="input min-h-36 font-mono text-xs"
                placeholder='Optional JSON merged into node settings, e.g. { "Trace": true }'
                spellCheck={false}
              />
            </div>
          </div>

          <FeedbackBanner error={error} />

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
