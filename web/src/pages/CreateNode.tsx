import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCreateNode } from '../hooks/useNodes';
import { ArrowLeft, Server, Loader2 } from 'lucide-react';
import { FeedbackBanner } from '../components/FeedbackBanner';
import { Link } from 'react-router-dom';
import { ApiRequestError } from '../utils/api';
import { normalizeNodeUpsertPayload, toNodeFormValues } from '../utils/nodePayloads';
import { useSecureSigners } from '../hooks/useSecureSigners';
import { useFeatures } from '../hooks/useFeatures';
import { useApplyNodeRole, useNodeRoles, type RoleSyncStrategy, type StorageEngine } from '../hooks/useNodeOrchestration';
import { STORAGE_ENGINE_OPTIONS, SYNC_STRATEGY_OPTIONS, roleSupportsNode, summarizeRole } from '../utils/orchestration';

const N3_NODE_TYPES = [
  { value: 'neo-cli', label: 'Neo CLI (C#)', description: 'Official Neo implementation with plugin support', chain: 'n3' as const },
  { value: 'neo-go', label: 'Neo Go', description: 'High-performance Go implementation', chain: 'n3' as const },
];

const X_NODE_TYPES = [
  { value: 'neox-go', label: 'Neo X (geth)', description: 'EVM-compatible sidechain — bane-labs/go-ethereum', chain: 'x' as const },
];

const N3_NETWORKS = [
  { value: 'mainnet', label: 'Mainnet', description: 'Production Neo N3 network' },
  { value: 'testnet', label: 'Testnet', description: 'Test network for development' },
  { value: 'private', label: 'Private', description: 'Local private network' },
];

const X_NETWORKS = [
  { value: 'neox-mainnet', label: 'Neo X Mainnet', description: 'Production Neo X network (chain id 47763)' },
  { value: 'neox-testnet', label: 'Neo X Testnet', description: 'Neo X test network (chain id 12227332)' },
];

function networksForType(type: string) {
  return type === 'neox-go' ? X_NETWORKS : N3_NETWORKS;
}

const SYNC_MODES = [
  { value: 'full', label: 'Full Sync', description: 'Download and verify entire blockchain' },
  { value: 'light', label: 'Light Sync', description: 'Fast sync with partial verification' },
];

export function getDefaultCreateNodeFormValues() {
  return toNodeFormValues({
    name: '',
    type: 'neo-cli',
    network: 'mainnet',
    syncMode: 'full',
    settings: {
      relay: true,
      debugMode: false,
    },
  });
}

export default function CreateNode() {
  const navigate = useNavigate();
  const createNode = useCreateNode();
  const applyRole = useApplyNodeRole();
  const rolesQuery = useNodeRoles();
  const secureSigners = useSecureSigners();
  const features = useFeatures();
  const NODE_TYPES = features.neox ? [...N3_NODE_TYPES, ...X_NODE_TYPES] : N3_NODE_TYPES;
  
  const [error, setError] = useState('');
  const [suggestion, setSuggestion] = useState('');
  const [code, setCode] = useState('');

  const [formData, setFormData] = useState(getDefaultCreateNodeFormValues);
  const [initialRoleId, setInitialRoleId] = useState('');
  const compatibleRoles = (rolesQuery.data ?? []).filter((role) => roleSupportsNode(role, formData.type));
  const selectedInitialRole = compatibleRoles.find((role) => role.id === initialRoleId);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuggestion('');
    setCode('');

    if (!formData.name.trim()) {
      setError('Please enter a node name');
      return;
    }

    if (formData.maxConnections !== '') {
      const val = Number(formData.maxConnections);
      if (!Number.isInteger(val) || val < 1) {
        setError('Max Connections must be a positive number');
        return;
      }
    }

    if (formData.minPeers !== '' && formData.maxPeers !== '') {
      if (Number(formData.minPeers) > Number(formData.maxPeers)) {
        setError('Min Peers cannot exceed Max Peers');
        return;
      }
    }

    try {
      const node = await createNode.mutateAsync(normalizeNodeUpsertPayload(formData));
      if (selectedInitialRole) {
        try {
          await applyRole.mutateAsync({
            roleId: selectedInitialRole.id,
            nodeId: node.id,
            storageEngine: formData.storageEngine,
          });
        } catch (roleError) {
          setError(`Node was created, but ${selectedInitialRole.name} could not be applied.`);
          if (roleError instanceof ApiRequestError) {
            setSuggestion(roleError.suggestion ?? roleError.message);
            setCode(roleError.code ?? '');
          } else {
            setSuggestion(roleError instanceof Error ? roleError.message : 'Open the node detail page and retry the role application after the node is stopped.');
            setCode('');
          }
          return;
        }
      }
      navigate(`/nodes/${node.id}`);
    } catch (err) {
      if (err instanceof ApiRequestError) {
        setError(err.message);
        setSuggestion(err.suggestion ?? '');
        setCode(err.code ?? '');
      } else {
        setError(err instanceof Error ? err.message : 'Failed to create node');
        setSuggestion('');
        setCode('');
      }
    }
  };

  return (
    <div className="max-w-4xl mx-auto animate-fade-in">
      <div className="mb-6">
        <Link to="/nodes" className="text-slate-600 hover:text-slate-950 flex items-center gap-2 text-sm">
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
            <h1 className="text-xl font-bold text-slate-950">Create Node</h1>
            <p className="text-slate-600 text-sm">Configure your new Neo node</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Node Name */}
          <div className="animate-fade-in-up" style={{ animationDelay: '0.05s' }}>
            <label className="block text-sm font-medium text-slate-700 mb-2">
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
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Node Type
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {NODE_TYPES.map((type) => (
                <label
                  key={type.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
                    formData.type === type.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-200 hover:border-slate-300 hover:bg-slate-50'
                  }`}
                >
                  <input
                    type="radio"
                    name="type"
                    value={type.value}
                    checked={formData.type === type.value}
                    onChange={(e) => {
                      const nextType = e.target.value as 'neo-cli' | 'neo-go' | 'neox-go';
                      const isX = nextType === 'neox-go';
                      const stillValidNetwork = isX
                        ? formData.network.startsWith('neox-')
                        : !formData.network.startsWith('neox-');
                      setFormData({
                        ...formData,
                        type: nextType,
                        network: stillValidNetwork
                          ? formData.network
                          : (isX ? 'neox-mainnet' : 'mainnet'),
                        ...(nextType !== 'neo-cli'
                          ? {
                              keyProtectionMode: 'standard',
                              secureSignerProfileId: '',
                            }
                          : {}),
                      });
                      setInitialRoleId('');
                    }}
                    className="sr-only"
                  />
                  <p className="font-medium text-slate-950">{type.label}</p>
                  <p className="text-sm text-slate-600 mt-1">{type.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Network */}
          <div className="animate-fade-in-up" style={{ animationDelay: '0.15s' }}>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Network
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              {networksForType(formData.type).map((network) => (
                <label
                  key={network.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
                    formData.network === network.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-200 hover:border-slate-300 hover:bg-slate-50'
                  }`}
                >
                  <input
                    type="radio"
                    name="network"
                    value={network.value}
                    checked={formData.network === network.value}
                    onChange={(e) => setFormData({ ...formData, network: e.target.value as 'mainnet' | 'testnet' | 'private' | 'neox-mainnet' | 'neox-testnet' })}
                    className="sr-only"
                  />
                  <p className="font-medium text-slate-950">{network.label}</p>
                  <p className="text-xs text-slate-600 mt-1">{network.description}</p>
                </label>
              ))}
            </div>
          </div>

          {/* Role Preset */}
          <div className="animate-fade-in-up" style={{ animationDelay: '0.18s' }}>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Role Preset
            </label>
            <div className="rounded-lg border border-slate-200 bg-slate-50 p-4">
              <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_220px]">
                <div>
                  <select
                    className="input"
                    value={initialRoleId}
                    onChange={(e) => {
                      const roleId = e.target.value;
                      const role = compatibleRoles.find((candidate) => candidate.id === roleId);
                      setInitialRoleId(roleId);
                      setFormData((current) => ({
                        ...current,
                        ...(role?.profile.storageEngine ? { storageEngine: role.profile.storageEngine } : {}),
                        ...(role?.profile.sync?.strategy ? { syncStrategy: role.profile.sync.strategy } : {}),
                      }));
                    }}
                    disabled={rolesQuery.isLoading || compatibleRoles.length === 0}
                  >
                    <option value="">No role preset</option>
                    {compatibleRoles.map((role) => (
                      <option key={role.id} value={role.id}>
                        {role.name} · {role.kind}
                      </option>
                    ))}
                  </select>
                  <p className="mt-2 text-xs text-slate-600">
                    {selectedInitialRole
                      ? summarizeRole(selectedInitialRole)
                      : compatibleRoles.length === 0
                        ? 'No compatible role presets for this implementation. Neo CLI supports the built-in plugin roles; custom roles can be created from a node detail page.'
                        : 'Optionally apply plugins, config, storage, sync, and data context immediately after creation.'}
                  </p>
                </div>
                <div className="rounded-lg border border-slate-200 bg-white p-3 text-sm text-slate-700">
                  <p className="font-medium text-slate-950">Apply timing</p>
                  <p className="mt-1 text-xs leading-5 text-slate-600">The new node is created first, then the role is applied while it is stopped.</p>
                </div>
              </div>
            </div>
          </div>

          {/* Sync Mode */}
          <div className="animate-fade-in-up" style={{ animationDelay: '0.2s' }}>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Sync Mode
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {SYNC_MODES.map((mode) => (
                <label
                  key={mode.value}
                  className={`cursor-pointer p-4 rounded-lg border-2 transition-all duration-200 ${
                    formData.syncMode === mode.value
                      ? 'border-blue-500 bg-blue-500/10'
                      : 'border-slate-200 hover:border-slate-300 hover:bg-slate-50'
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
                  <p className="font-medium text-slate-950">{mode.label}</p>
                  <p className="text-sm text-slate-600 mt-1">{mode.description}</p>
                </label>
              ))}
            </div>
          </div>

          <div className="animate-fade-in-up rounded-lg border border-slate-200 bg-slate-50 p-4" style={{ animationDelay: '0.23s' }}>
            <div className="mb-3">
              <h2 className="text-sm font-medium text-slate-950">Storage and Initial Sync Strategy</h2>
              <p className="mt-1 text-sm text-slate-600">Choose the node database engine and the strategy NeoNexus records for role/data-context orchestration.</p>
            </div>
            <div className="grid gap-3 lg:grid-cols-2">
              <div>
                <label className="block text-xs font-medium text-slate-600 mb-2">Storage Engine</label>
                <div className="grid gap-3 sm:grid-cols-2">
                  {STORAGE_ENGINE_OPTIONS.map((engine) => (
                    <label
                      key={engine.value}
                      className={`cursor-pointer rounded-lg border p-4 transition-all ${
                        formData.storageEngine === engine.value
                          ? 'border-teal-500 bg-white'
                          : 'border-slate-200 bg-white/70 hover:border-slate-300'
                      }`}
                    >
                      <input
                        type="radio"
                        name="storageEngine"
                        value={engine.value}
                        checked={formData.storageEngine === engine.value}
                        onChange={(e) => setFormData({ ...formData, storageEngine: e.target.value as StorageEngine })}
                        className="sr-only"
                      />
                      <p className="font-medium text-slate-950">{engine.label}</p>
                      <p className="mt-1 text-xs leading-5 text-slate-600">{engine.description}</p>
                    </label>
                  ))}
                </div>
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-600 mb-2">Sync Strategy</label>
                <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
                  {SYNC_STRATEGY_OPTIONS.map((strategy) => (
                    <label
                      key={strategy.value}
                      className={`cursor-pointer rounded-lg border p-4 transition-all ${
                        formData.syncStrategy === strategy.value
                          ? 'border-teal-500 bg-white'
                          : 'border-slate-200 bg-white/70 hover:border-slate-300'
                      }`}
                    >
                      <input
                        type="radio"
                        name="syncStrategy"
                        value={strategy.value}
                        checked={formData.syncStrategy === strategy.value}
                        onChange={(e) => setFormData({ ...formData, syncStrategy: e.target.value as RoleSyncStrategy })}
                        className="sr-only"
                      />
                      <p className="font-medium text-slate-950">{strategy.label}</p>
                      <p className="mt-1 text-xs leading-5 text-slate-600">{strategy.description}</p>
                    </label>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Info Box */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <p className="text-sm text-blue-800">
              <strong>Note:</strong> The node will be created with automatically assigned ports. 
              Ports will be allocated starting from 10332 (RPC) and 10333 (P2P).
            </p>
          </div>

          <div className="space-y-4 rounded-lg border border-teal-200 bg-teal-50 p-4">
            <div>
              <h2 className="text-sm font-medium text-teal-950">Private Key Protection</h2>
              <p className="mt-1 text-sm text-teal-900">
                NeoNexus can bind `neo-cli` nodes to a secure signer profile so node configs reference a signing endpoint instead of raw private keys.
              </p>
            </div>

            {formData.type === 'neo-cli' ? (
              <>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <label
                    className={`cursor-pointer rounded-lg border-2 p-4 transition-all ${
                      formData.keyProtectionMode === 'standard'
                        ? 'border-teal-500 bg-white'
                        : 'border-slate-200 bg-white/70 hover:border-slate-300'
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
                    <p className="font-medium text-slate-950">Standard Local Wallet</p>
                    <p className="mt-1 text-sm text-slate-600">Use the node's regular wallet flow without NeoNexus-managed signer binding.</p>
                  </label>

                  <label
                    className={`cursor-pointer rounded-lg border-2 p-4 transition-all ${
                      formData.keyProtectionMode === 'secure-signer'
                        ? 'border-teal-500 bg-white'
                        : 'border-slate-200 bg-white/70 hover:border-slate-300'
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
                    <p className="font-medium text-slate-950">Secure Signer / TEE</p>
                    <p className="mt-1 text-sm text-slate-600">Attach a software, SGX, or Nitro signer profile and auto-wire Neo SignClient.</p>
                  </label>
                </div>

                {formData.keyProtectionMode === 'secure-signer' && (
                  <div className="space-y-3">
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-2">Secure Signer Profile</label>
                      <select
                        value={formData.secureSignerProfileId}
                        onChange={(e) => setFormData({ ...formData, secureSignerProfileId: e.target.value })}
                        className="input"
                        disabled={!secureSigners.data?.some((profile) => profile.enabled)}
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
                    <p className="text-sm text-amber-700">
                        No secure signer profiles are configured yet. Add one in <Link to="/settings" className="underline">Settings</Link> before enabling TEE-backed signing.
                      </p>
                    )}
                    {(secureSigners.data ?? []).length > 0 && !(secureSigners.data ?? []).some((profile) => profile.enabled) && (
                      <p className="text-sm text-amber-700">
                        All secure signer profiles are disabled. Enable a profile in <Link to="/settings" className="underline">Settings</Link> before attaching protected signing.
                      </p>
                    )}

                    <p className="text-xs text-slate-600">
                      NeoNexus stores only the signer profile reference and endpoint metadata here. It does not store raw WIF or plaintext unlock material.
                    </p>
                  </div>
                )}
              </>
            ) : (
              <p className="text-sm text-teal-900">
                Secure signer auto-wiring currently targets `neo-cli` nodes via the Neo `SignClient` plugin. `neo-go` remains standard-wallet only for now.
              </p>
            )}
          </div>

          {/* Advanced Settings */}
          <div className="space-y-4">
            <div>
              <h2 className="text-sm font-medium text-slate-700 mb-3">Advanced Settings</h2>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div>
                  <label className="block text-xs font-medium text-slate-600 mb-2">Max Connections</label>
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
                  <label className="block text-xs font-medium text-slate-600 mb-2">Min Peers</label>
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
                  <label className="block text-xs font-medium text-slate-600 mb-2">Max Peers</label>
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
              <label className="flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-4 py-3">
                <div>
                  <p className="text-sm font-medium text-slate-950">Relay Transactions</p>
                  <p className="text-xs text-slate-600">Allow the node to relay transactions</p>
                </div>
                <input
                  type="checkbox"
                  checked={formData.relay}
                  onChange={(e) => setFormData({ ...formData, relay: e.target.checked })}
                  className="h-4 w-4"
                />
              </label>

              <label className="flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-4 py-3">
                <div>
                  <p className="text-sm font-medium text-slate-950">Debug Mode</p>
                  <p className="text-xs text-slate-600">Enable verbose debugging where supported</p>
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
              <label className="block text-sm font-medium text-slate-700 mb-2">Custom Config JSON</label>
              <textarea
                value={formData.customConfig}
                onChange={(e) => setFormData({ ...formData, customConfig: e.target.value })}
                className="input min-h-36 font-mono text-xs"
                placeholder='Optional JSON merged into node settings, e.g. { "Trace": true }'
                spellCheck={false}
              />
            </div>
          </div>

          <FeedbackBanner error={error} suggestion={suggestion} code={code} />

          {/* Submit */}
          <div className="flex flex-col-reverse gap-3 sm:flex-row">
            <Link to="/nodes" className="btn btn-secondary justify-center sm:flex-1">
              Cancel
            </Link>
            <button
              type="submit"
              disabled={createNode.isPending || applyRole.isPending}
              className="btn btn-primary justify-center sm:flex-1"
            >
              {createNode.isPending || applyRole.isPending ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  {applyRole.isPending ? 'Applying role...' : 'Creating...'}
                </>
              ) : (
                selectedInitialRole ? 'Create and Apply Role' : 'Create Node'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
