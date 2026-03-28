import { useState, useEffect } from 'react';
import { Pencil, Save, X } from 'lucide-react';
import { useUpdateNode, type Node } from '../../hooks/useNodes';
import { useSecureSigners } from '../../hooks/useSecureSigners';
import { normalizeNodeUpsertPayload, toNodeFormValues } from '../../utils/nodePayloads';
import { FeedbackBanner } from '../../components/FeedbackBanner';

interface NodeConfigEditorProps {
  node: Node;
}

export function NodeConfigEditor({ node }: NodeConfigEditorProps) {
  const updateNode = useUpdateNode();
  const secureSigners = useSecureSigners();
  const [isEditing, setIsEditing] = useState(false);
  const [configError, setConfigError] = useState('');
  const [configSuccess, setConfigSuccess] = useState('');
  const [formData, setFormData] = useState(() => toNodeFormValues(node));

  useEffect(() => {
    if (!isEditing) {
      setFormData(toNodeFormValues(node));
    }
  }, [node, isEditing]);

  const handleSave = async () => {
    setConfigError('');
    setConfigSuccess('');

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
      setIsEditing(false);
      setConfigSuccess('Configuration updated successfully.');
    } catch (error) {
      setConfigError(error instanceof Error ? error.message : 'Failed to update configuration.');
    }
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between gap-4 mb-4">
        <h3 className="text-lg font-semibold text-white">Configuration</h3>
        {isEditing ? (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => {
                setIsEditing(false);
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
              type="button"
              onClick={handleSave}
              disabled={updateNode.isPending}
              className="btn btn-primary"
            >
              <Save className="w-4 h-4" />
              {updateNode.isPending ? 'Saving...' : 'Save'}
            </button>
          </div>
        ) : (
          <button
            type="button"
            onClick={() => {
              setConfigError('');
              setConfigSuccess('');
              setFormData(toNodeFormValues(node));
              setIsEditing(true);
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

      {isEditing ? (
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
  );
}
