'use client';

import { CheckCircle2, Eye, EyeOff, Globe, Key, Lock, Plus, RefreshCw, Shield, Trash2, Copy } from 'lucide-react';
import { useState } from 'react';
import { createApiKeyAction, deleteApiKeyAction } from './actions';

export type ApiKeyType = {
  id: string;
  name: string;
  keyHash: string;
  createdAt: Date;
  isActive: boolean;
};

export default function SecurityClient({ initialKeys, billingPlan }: { initialKeys: ApiKeyType[], billingPlan: string }) {
  const [keys, setKeys] = useState<ApiKeyType[]>(initialKeys);
  const [visibleKeys, setVisibleKeys] = useState<Record<string, boolean>>({});
  const [isCreating, setIsCreating] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);

  const toggleVisibility = (id: string) => {
    setVisibleKeys(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const handleCreateKey = async () => {
    setIsCreating(true);
    const result = await createApiKeyAction(`Key ${keys.length + 1}`);
    if (result.success && result.key) {
      setNewKey(result.key); // Show raw key once
      // Optimistic update (requires a real reload to get the DB id, but Next.js Server Actions usually revalidate)
      window.location.reload(); 
    } else {
      alert('Failed to create key or DB not connected.');
    }
    setIsCreating(false);
  };

  const handleDeleteKey = async (id: string) => {
    const result = await deleteApiKeyAction(id);
    if (result.success) {
      setKeys(keys.filter(k => k.id !== id));
    }
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
                      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg px-3 py-2 flex items-center gap-3 flex-1">
                        <code className="text-sm font-mono text-gray-300 w-48 truncate">
                          {visibleKeys[key.id] ? `nk_live_...${key.keyHash.substring(0, 8)}` : 'nk_live_••••••••••••••••••••••••'}
                        </code>
                        <button onClick={() => toggleVisibility(key.id)} className="text-gray-500 hover:text-white transition-colors">
                          {visibleKeys[key.id] ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                        </button>
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
                <p className="text-xs text-gray-400 font-normal">Restrict requests to trusted sources only.</p>
              </div>
            </div>
            
            <div className="p-6 space-y-6">
              <div className="flex gap-3">
                <input 
                  type="text" 
                  placeholder="e.g. 192.168.1.1 or https://mydapp.com" 
                  className="flex-1 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
                />
                <button className="bg-[#333333] hover:bg-[#444444] text-white px-6 py-2.5 rounded-xl text-sm font-bold transition-colors">
                  Add Rule
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
                    <tr className="hover:bg-[#222222] transition-colors">
                      <td className="px-4 py-3 font-mono">203.0.113.45</td>
                      <td className="px-4 py-3"><span className="bg-gray-800 text-gray-300 px-2 py-0.5 rounded text-xs">IP Address</span></td>
                      <td className="px-4 py-3 text-right"><button className="text-red-400 hover:text-red-300">Remove</button></td>
                    </tr>
                    <tr className="hover:bg-[#222222] transition-colors">
                      <td className="px-4 py-3 font-mono">https://app.neodapp.io</td>
                      <td className="px-4 py-3"><span className="bg-gray-800 text-gray-300 px-2 py-0.5 rounded text-xs">Origin (CORS)</span></td>
                      <td className="px-4 py-3 text-right"><button className="text-red-400 hover:text-red-300">Remove</button></td>
                    </tr>
                  </tbody>
                </table>
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
              <p className="text-sm text-gray-400 mb-6">Select which RPC methods are allowed on your public endpoints. Prevents abuse of heavy state queries.</p>
              
              {billingPlan === 'developer' && (
                <div className="bg-yellow-500/10 border border-yellow-500/20 text-yellow-500 text-xs p-3 rounded-lg mb-6">
                  Method Firewall requires the Growth or Dedicated plan. <a href="/app/billing" className="underline font-bold">Upgrade now</a>.
                </div>
              )}

              <div className={`space-y-5 ${billingPlan === 'developer' ? 'opacity-50 pointer-events-none' : ''}`}>
                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" defaultChecked className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[var(--color-dark-panel)] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Write Methods</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">sendrawtransaction, submitblock</span>
                  </div>
                </label>
                
                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[var(--color-dark-panel)] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Debug Methods</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">getstorage, getcontractstate</span>
                  </div>
                </label>

                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" defaultChecked className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[var(--color-dark-panel)] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Execution Tests</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">invokefunction, invokescript</span>
                  </div>
                </label>
              </div>

              <div className="mt-8 pt-6 border-t border-[var(--color-dark-border)]">
                <button 
                  disabled={billingPlan === 'developer'}
                  className="w-full bg-[#333333] hover:bg-[#444444] disabled:opacity-50 disabled:cursor-not-allowed text-white py-3 rounded-xl text-sm font-bold transition-colors"
                >
                  Save Firewall Rules
                </button>
              </div>
            </div>
          </div>

          <div className="bg-[#00E599]/10 border border-[#00E599]/20 rounded-2xl p-6 text-[#00E599]">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="w-5 h-5" />
              <h3 className="font-bold">Enterprise Security</h3>
            </div>
            <p className="text-sm opacity-80 leading-relaxed mb-4">
              Need VPC peering, private AWS PrivateLink, or dedicated proxy nodes? Upgrade to Enterprise for custom network topologies.
            </p>
            <button className="px-4 py-2 bg-[#00E599] text-black text-xs font-bold rounded-lg hover:bg-[#00cc88] transition-colors">
              Contact Sales
            </button>
          </div>
        </div>

      </div>
    </div>
  );
}
