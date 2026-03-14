'use client';

import { CheckCircle2, Eye, EyeOff, Globe, Key, Lock, Plus, RefreshCw, Shield, Trash2 } from 'lucide-react';
import { useState } from 'react';

export default function Security() {
  const [showKey, setShowKey] = useState(false);

  return (
    <div className="min-h-screen pb-12 space-y-8">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Security & Access</h1>
          <p className="text-gray-400 text-lg">Manage API keys, firewalls, and endpoint protection rules.</p>
        </div>
        <button className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-lg text-sm font-bold transition-colors flex items-center gap-2">
          <Plus className="w-4 h-4" /> Create API Key
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Left Column: API Keys */}
        <div className="lg:col-span-2 space-y-8">
          
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-2xl overflow-hidden shadow-sm">
            <div className="px-6 py-5 border-b border-[#333333] bg-[#111111]/50 flex items-center gap-3">
              <Key className="w-5 h-5 text-[#00E599]" />
              <h2 className="text-lg font-bold text-white">Active API Keys</h2>
            </div>
            
            <div className="p-6">
              <div className="border border-[#333333] rounded-xl p-4 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 hover:border-gray-500 transition-colors">
                <div>
                  <h3 className="text-base font-bold text-white flex items-center gap-2">
                    Production DApp Key
                    <span className="bg-[#00E599]/10 text-[#00E599] text-[10px] px-2 py-0.5 rounded font-bold uppercase">Master</span>
                  </h3>
                  <p className="text-xs text-gray-500 mt-1">Created on Oct 12, 2023 • Never used</p>
                </div>
                
                <div className="flex items-center gap-3 w-full sm:w-auto">
                  <div className="bg-[#111111] border border-[#333333] rounded-lg px-3 py-2 flex items-center gap-3 flex-1">
                    <code className="text-sm font-mono text-gray-300 w-48">
                      {showKey ? 'nk_live_a8f9c2e4b6d7e8f9a0b1c2d3' : 'nk_live_••••••••••••••••••••••••'}
                    </code>
                    <button onClick={() => setShowKey(!showKey)} className="text-gray-500 hover:text-white transition-colors">
                      {showKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
                  <button className="p-2 text-gray-500 hover:text-white bg-[#111111] border border-[#333333] rounded-lg transition-colors">
                    <RefreshCw className="w-4 h-4" />
                  </button>
                  <button className="p-2 text-red-500 hover:text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg transition-colors">
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div className="bg-[#1A1A1A] border border-[#333333] rounded-2xl overflow-hidden shadow-sm">
            <div className="px-6 py-5 border-b border-[#333333] bg-[#111111]/50 flex items-center gap-3">
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
                  className="flex-1 bg-[#111111] border border-[#333333] rounded-xl px-4 py-2.5 text-sm text-white focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
                />
                <button className="bg-[#333333] hover:bg-[#444444] text-white px-6 py-2.5 rounded-xl text-sm font-bold transition-colors">
                  Add Rule
                </button>
              </div>

              <div className="border border-[#333333] rounded-xl overflow-hidden">
                <table className="w-full text-left text-sm">
                  <thead className="bg-[#111111] text-gray-400">
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
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-2xl shadow-sm overflow-hidden">
            <div className="px-6 py-5 border-b border-[#333333] bg-[#111111]/50 flex items-center gap-3">
              <Lock className="w-5 h-5 text-yellow-500" />
              <div>
                <h2 className="text-lg font-bold text-white">Method Firewall</h2>
              </div>
            </div>
            
            <div className="p-6">
              <p className="text-sm text-gray-400 mb-6">Select which Neo N3 RPC methods are allowed on your public endpoints. Prevents abuse of heavy state queries.</p>
              
              <div className="space-y-5">
                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" defaultChecked className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[#111111] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Write Methods</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">sendrawtransaction, submitblock</span>
                  </div>
                </label>
                
                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[#111111] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Debug Methods</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">getstorage, getcontractstate</span>
                  </div>
                </label>

                <label className="flex items-start gap-3 cursor-pointer group">
                  <div className="relative flex items-center justify-center mt-0.5">
                    <input type="checkbox" defaultChecked className="peer appearance-none w-5 h-5 rounded border border-[#555] bg-[#111111] checked:bg-[#00E599] checked:border-[#00E599] transition-all cursor-pointer" />
                    <CheckCircle2 className="w-3.5 h-3.5 text-black absolute opacity-0 peer-checked:opacity-100 pointer-events-none" />
                  </div>
                  <div>
                    <span className="text-sm font-bold text-white block">Allow Execution Tests</span>
                    <span className="text-xs text-gray-500 block mt-0.5 font-mono">invokefunction, invokescript</span>
                  </div>
                </label>
              </div>

              <div className="mt-8 pt-6 border-t border-[#333333]">
                <button className="w-full bg-[#333333] hover:bg-[#444444] text-white py-3 rounded-xl text-sm font-bold transition-colors">
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
