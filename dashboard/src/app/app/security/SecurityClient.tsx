'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { CheckCircle2, Globe, Key, Lock, Plus, Shield, Trash2, Copy } from 'lucide-react';
import { useState } from 'react';
import toast from 'react-hot-toast';
import {
  createApiKeyAction,
  createFirewallRuleAction,
  deleteApiKeyAction,
  deleteFirewallRuleAction,
  setMethodFirewallPresetAction,
} from './actions';
import { mergeCreatedApiKey } from './SecurityClientState';
import {
  listMethodFirewallPresets,
  isMethodFirewallPresetActive,
} from '@/services/security/MethodFirewallPresets';

export type ApiKeyType = {
  id: string;
  name: string;
  keyHash: string;
  createdAt: Date;
  isActive: boolean;
};

type FirewallRuleView = {
  id: number;
  type: 'ip_allow' | 'origin_allow' | 'method_block';
  value: string;
  createdAt: string;
};

export default function SecurityClient({
  initialKeys,
  initialFirewallRules,
  billingPlan,
}: {
  initialKeys: ApiKeyType[];
  initialFirewallRules: FirewallRuleView[];
  billingPlan: string;
}) {
  const router = useRouter();
  const [keys, setKeys] = useState<ApiKeyType[]>(initialKeys);
  const [firewallRules, setFirewallRules] = useState<FirewallRuleView[]>(initialFirewallRules);
  const [isCreating, setIsCreating] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);
  const [firewallType, setFirewallType] = useState<'ip_allow' | 'origin_allow'>('ip_allow');
  const [firewallValue, setFirewallValue] = useState('');
  const [isSavingFirewallRule, setIsSavingFirewallRule] = useState(false);
  const [deletingFirewallRuleId, setDeletingFirewallRuleId] = useState<number | null>(null);
  const [updatingMethodPreset, setUpdatingMethodPreset] = useState<string | null>(null);
  const allowlistRules = firewallRules.filter((rule) => rule.type !== 'method_block');
  const methodBlockRules = firewallRules.filter((rule) => rule.type === 'method_block');

  const handleCreateKey = async () => {
    setIsCreating(true);
    const result = await createApiKeyAction(`Key ${keys.length + 1}`);
    if (result.success && result.key && result.apiKey) {
      setNewKey(result.key); // Show raw key once
      setKeys((current) => mergeCreatedApiKey(current, {
        ...result.apiKey,
        createdAt: new Date(result.apiKey.createdAt),
      }));
      toast.success('API key created. Copy it now because it will not be shown again.');
    } else {
      toast.error(result.error || 'Failed to create API key.');
    }
    setIsCreating(false);
  };

  const handleDeleteKey = async (id: string) => {
    const result = await deleteApiKeyAction(id);
    if (result.success) {
      setKeys(keys.filter(k => k.id !== id));
      toast.success('API key removed.');
    } else {
      toast.error(result.error || 'Failed to delete API key.');
    }
  };

  const handleCreateFirewallRule = async () => {
    if (!firewallValue.trim()) {
      toast.error('Enter an IP/CIDR or origin URL.');
      return;
    }

    setIsSavingFirewallRule(true);
    const result = await createFirewallRuleAction(firewallType, firewallValue.trim());
    if (result.success && result.rule) {
      setFirewallRules((current) => [...current, result.rule]);
      setFirewallValue('');
      toast.success('Firewall rule saved and synced to routed endpoints.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to save firewall rule.');
    }
    setIsSavingFirewallRule(false);
  };

  const handleDeleteFirewallRule = async (id: number) => {
    setDeletingFirewallRuleId(id);
    const result = await deleteFirewallRuleAction(id);
    if (result.success) {
      setFirewallRules((current) => current.filter((rule) => rule.id !== id));
      toast.success('Firewall rule removed and routes resynced.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to remove firewall rule.');
    }
    setDeletingFirewallRuleId(null);
  };

  const handleToggleMethodPreset = async (presetId: 'write_methods' | 'debug_methods' | 'execution_tests', enabled: boolean) => {
    setUpdatingMethodPreset(presetId);
    const result = await setMethodFirewallPresetAction(presetId, enabled);
    if (result.success && 'rules' in result && result.rules) {
      const rules = result.rules as FirewallRuleView[];
      setFirewallRules(rules.map((rule) => ({
        id: rule.id,
        type: rule.type as FirewallRuleView['type'],
        value: rule.value,
        createdAt: rule.createdAt,
      })));
      toast.success(enabled ? 'Method firewall preset enabled.' : 'Method firewall preset removed.');
    } else {
      toast.error(result.error || 'Failed to update method firewall preset.');
    }
    setUpdatingMethodPreset(null);
  };

  return (
    <div className="min-h-screen pb-12 space-y-8">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Security & Access</h1>
          <p className="text-gray-400 text-lg">Manage API keys, firewalls, and endpoint protection rules.</p>
        </div>
        <button 
          onClick={handleCreateKey}
          disabled={isCreating}
          className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-lg text-sm font-bold transition-colors flex items-center gap-2"
        >
          <Plus className="w-4 h-4" /> {isCreating ? 'Creating...' : 'Create API Key'}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Left Column: API Keys */}
        <div className="lg:col-span-2 space-y-8">
          
          {newKey && (
             <div className="bg-[#00E599]/10 border border-[#00E599]/30 rounded-xl p-6 relative overflow-hidden">
                <h3 className="text-lg font-bold text-[#00E599] mb-2">New API Key Created!</h3>
                <p className="text-sm text-gray-300 mb-4">Please copy this key now. You will not be able to see it again.</p>
                <div className="bg-[var(--color-dark-panel)] p-3 rounded-lg flex items-center justify-between border border-[var(--color-dark-border)]">
                  <code className="text-[#00E599] font-mono break-all">{newKey}</code>
                  <button onClick={() => navigator.clipboard.writeText(newKey)} className="text-gray-400 hover:text-white ml-4">
                    <Copy className="w-5 h-5" />
                  </button>
                </div>
                <button onClick={() => setNewKey(null)} className="mt-4 text-xs text-gray-400 hover:text-white underline">I have copied my key</button>
             </div>
          )}

          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl overflow-hidden shadow-sm">
            <div className="px-6 py-5 border-b border-[var(--color-dark-border)] bg-[var(--color-dark-panel)]/50 flex items-center gap-3">
              <Key className="w-5 h-5 text-[#00E599]" />
              <h2 className="text-lg font-bold text-white">Active API Keys</h2>
            </div>
            
            <div className="p-6 space-y-4">
              {keys.length === 0 ? (
                 <div className="text-center py-8 text-gray-500 text-sm">No API keys found. Create one to authenticate your requests.</div>
              ) : (
                keys.map((key, index) => (
                  <div key={key.id} className="border border-[var(--color-dark-border)] rounded-xl p-4 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 hover:border-gray-500 transition-colors">
                    <div>
                      <h3 className="text-base font-bold text-white flex items-center gap-2">
                        {key.name}
                        {index === 0 && <span className="bg-[#00E599]/10 text-[#00E599] text-[10px] px-2 py-0.5 rounded font-bold uppercase">Master</span>}
                      </h3>
                      <p className="text-xs text-gray-500 mt-1">Created on {new Date(key.createdAt).toLocaleDateString()}</p>
                    </div>
                    
                    <div className="flex items-center gap-3 w-full sm:w-auto">
                      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg px-3 py-2 flex flex-col gap-1 flex-1 min-w-[220px]">
                        <div className="text-[10px] uppercase tracking-wide text-gray-500 font-bold">Stored fingerprint</div>
                        <code className="text-sm font-mono text-gray-300 truncate">sha256:{key.keyHash.substring(0, 16)}</code>
                        <div className="text-xs text-gray-500">Raw API keys are only shown once at creation time.</div>
                      </div>
                      <button onClick={() => handleDeleteKey(key.id)} className="p-2 text-red-500 hover:text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg transition-colors">
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl overflow-hidden shadow-sm">
            <div className="px-6 py-5 border-b border-[var(--color-dark-border)] bg-[var(--color-dark-panel)]/50 flex items-center gap-3">
              <Globe className="w-5 h-5 text-blue-400" />
              <div>
                <h2 className="text-lg font-bold text-white">IP & Origin Allowlist</h2>
                <p className="text-xs text-gray-400 font-normal">Org-level trusted source restrictions applied to routed endpoint traffic.</p>
              </div>
            </div>
            
            <div className="p-6 space-y-6">
              <div className="flex gap-3">
                <select
                  value={firewallType}
                  onChange={(event) => setFirewallType(event.target.value as 'ip_allow' | 'origin_allow')}
                  className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl px-4 py-2.5 text-sm text-white"
                >
                  <option value="ip_allow">IP / CIDR</option>
                  <option value="origin_allow">Origin URL</option>
                </select>
                <input
                  type="text"
                  value={firewallValue}
                  onChange={(event) => setFirewallValue(event.target.value)}
                  placeholder={firewallType === 'ip_allow' ? 'e.g. 203.0.113.10 or 203.0.113.0/24' : 'https://app.example.com'}
                  className="flex-1 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
                />
                <button
                  onClick={handleCreateFirewallRule}
                  disabled={isSavingFirewallRule}
                  className="bg-[#333333] hover:bg-[#444444] disabled:opacity-50 text-white px-6 py-2.5 rounded-xl text-sm font-bold transition-colors"
                >
                  {isSavingFirewallRule ? 'Saving...' : 'Add Rule'}
                </button>
              </div>

              <div className="border border-[var(--color-dark-border)] rounded-xl overflow-hidden">
                <table className="w-full text-left text-sm">
                  <thead className="bg-[var(--color-dark-panel)] text-gray-400">
                    <tr>
                      <th className="px-4 py-3 font-medium">Value</th>
                      <th className="px-4 py-3 font-medium">Type</th>
                      <th className="px-4 py-3 font-medium text-right">Action</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[#333333] text-gray-300">
                    {allowlistRules.length === 0 ? (
                      <tr>
                        <td colSpan={3} className="px-4 py-6 text-center text-gray-500">
                          No org-level allowlist rules yet. Add one to restrict routed endpoint traffic.
                        </td>
                      </tr>
                    ) : allowlistRules.map((rule) => (
                      <tr key={rule.id} className="hover:bg-[#222222] transition-colors">
                        <td className="px-4 py-3 font-mono">{rule.value}</td>
                        <td className="px-4 py-3">
                          <span className="bg-gray-800 text-gray-300 px-2 py-0.5 rounded text-xs">
                            {rule.type === 'ip_allow' ? 'IP / CIDR' : 'Origin'}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-right">
                          <button
                            onClick={() => handleDeleteFirewallRule(rule.id)}
                            disabled={deletingFirewallRuleId !== null}
                            className="text-red-400 hover:text-red-300 disabled:opacity-50"
                          >
                            {deletingFirewallRuleId === rule.id ? 'Removing...' : 'Remove'}
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-4">
                <h3 className="text-sm font-bold text-white mb-2">Current enforcement</h3>
                <ul className="space-y-2 text-sm text-gray-300">
                  <li>API keys are enforced through per-key APISIX consumers.</li>
                  <li>IP/CIDR allowlist rules are applied through APISIX route `ip-restriction` plugins.</li>
                  <li>Origin allowlist rules are applied through APISIX route `cors` plugins.</li>
                </ul>
              </div>
            </div>
          </div>

        </div>

        {/* Right Column: Advanced Firewall */}
        <div className="space-y-6">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl shadow-sm overflow-hidden">
            <div className="px-6 py-5 border-b border-[var(--color-dark-border)] bg-[var(--color-dark-panel)]/50 flex items-center gap-3">
              <Lock className="w-5 h-5 text-yellow-500" />
              <div>
                <h2 className="text-lg font-bold text-white">Method Firewall</h2>
              </div>
            </div>
            
            <div className="p-6">
              <p className="text-sm text-gray-400 mb-6">Block selected JSON-RPC method categories on routed public traffic. These presets are enforced through APISIX route plugins.</p>
              
              {billingPlan === 'developer' && (
                <div className="bg-yellow-500/10 border border-yellow-500/20 text-yellow-500 text-xs p-3 rounded-lg mb-6">
                  Method firewall presets require the Growth or Dedicated plan. <Link href="/app/billing" className="underline font-bold">Upgrade now</Link>.
                </div>
              )}

              <div className="space-y-5">
                {listMethodFirewallPresets().map((preset) => {
                  const isActive = isMethodFirewallPresetActive(
                    preset.id,
                    methodBlockRules.map((rule) => rule.value),
                  );

                  return (
                    <label key={preset.id} className={`flex items-start gap-3 ${billingPlan === 'developer' ? 'opacity-50' : ''}`}>
                      <div className="relative flex items-center justify-center mt-0.5">
                        <input
                          type="checkbox"
                          checked={isActive}
                          disabled={billingPlan === 'developer' || updatingMethodPreset !== null}
                          onChange={(event) => handleToggleMethodPreset(preset.id, event.target.checked)}
                          className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[var(--color-dark-panel)] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer disabled:cursor-not-allowed"
                        />
                        <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                      </div>
                      <div>
                        <span className="text-sm font-bold text-white block">{preset.name}</span>
                        <span className="text-xs text-gray-500 block mt-0.5">{preset.description}</span>
                        <span className="text-xs text-gray-500 block mt-1 font-mono">{preset.methods.join(', ')}</span>
                      </div>
                    </label>
                  );
                })}
              </div>

              <p className="mt-8 pt-6 border-t border-[var(--color-dark-border)] text-sm text-gray-400">
                IP/origin allowlists use native APISIX plugins. JSON-RPC method blocking is enforced through an APISIX serverless access-phase function that inspects request bodies before they reach the upstream node.
              </p>
            </div>
          </div>

          <div className="bg-[#00E599]/10 border border-[#00E599]/20 rounded-2xl p-6 text-[#00E599]">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="w-5 h-5" />
              <h3 className="font-bold">Enterprise Security</h3>
            </div>
            <p className="text-sm opacity-80 leading-relaxed mb-4">
              Need private network peering, dedicated proxy nodes, or custom traffic-isolation requirements? Upgrade to Enterprise for tailored network topologies.
            </p>
            <Link href="/pricing" className="inline-flex px-4 py-2 bg-[#00E599] text-black text-xs font-bold rounded-lg hover:bg-[#00cc88] transition-colors">
              Contact Sales
            </Link>
          </div>
        </div>

      </div>
    </div>
  );
}
