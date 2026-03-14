'use client';

import { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, Activity, Box, Copy, Globe, MoreVertical, Play, Power, RotateCcw, Server, Terminal, Lock, Plug } from 'lucide-react';
import { Endpoint } from '../EndpointsList';

export default function EndpointDetailsClient({ endpoint }: { endpoint: Endpoint | null }) {
  const [activeTab, setActiveTab] = useState('overview');

  const tabs = [
    { id: 'overview', name: 'Overview' },
    { id: 'metrics', name: 'Health Metrics' },
    { id: 'logs', name: 'Node Logs' },
    { id: 'plugins', name: 'Plugins' },
    { id: 'settings', name: 'Settings' },
  ];

  const wssUrl = endpoint?.url ? endpoint.url.replace('https://', 'wss://').replace('/v1', '/ws') : 'wss://node-tokyo-01.neonexus.io/ws/xyz';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <Link href="/endpoints" className="inline-flex items-center text-sm text-gray-400 hover:text-white mb-4 transition-colors">
          <ArrowLeft className="w-4 h-4 mr-1" /> Back to Endpoints
        </Link>
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
              {endpoint?.name || 'GameFi Indexer Node'}
              <span className={`text-xs font-medium px-2 py-1 rounded-full border ${
                endpoint?.status.toLowerCase() === 'active' 
                  ? 'border-[#00E599]/30 text-[#00E599] bg-[#00E599]/10' 
                  : 'border-yellow-500/30 text-yellow-500 bg-yellow-500/10'
              }`}>
                {endpoint?.status || 'Active'}
              </span>
            </h1>
            <p className="text-sm text-gray-400 mt-1 flex items-center gap-2">
              <Server className="w-4 h-4" /> {endpoint?.type || 'Dedicated'} {endpoint?.network || 'N3 Mainnet'} • AWS Tokyo
            </p>
          </div>
          
          <div className="flex items-center gap-3">
            <button className="bg-[#1A1A1A] border border-[#333333] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2">
              <RotateCcw className="w-4 h-4" /> Restart
            </button>
            <button className="bg-[#1A1A1A] border border-[#333333] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2">
              <Power className="w-4 h-4" /> Stop
            </button>
            <button className="bg-[#1A1A1A] border border-[#333333] hover:bg-[#252525] text-gray-400 p-2 rounded-md transition-colors">
              <MoreVertical className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-[#333333]">
        <nav className="flex space-x-8">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`py-4 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-[#00E599] text-[#00E599]'
                  : 'border-transparent text-gray-400 hover:text-white hover:border-gray-600'
              }`}
            >
              {tab.name}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      <div className="pt-2">
        {activeTab === 'overview' && (
          <div className="space-y-6 animate-in fade-in">
            {/* Connection URLs */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Globe className="w-5 h-5 text-blue-400" />
                  <h2 className="text-lg font-medium text-white">HTTPS Endpoint</h2>
                </div>
                <div className="bg-[#111111] border border-[#333333] p-3 rounded-lg flex items-center justify-between group">
                  <code className="text-sm text-[#00E599] truncate">{endpoint?.url || 'https://node-tokyo-01.neonexus.io/v1/xyz'}</code>
                  <button 
                    onClick={() => navigator.clipboard.writeText(endpoint?.url || '')}
                    className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[#1A1A1A] opacity-0 group-hover:opacity-100"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Activity className="w-5 h-5 text-purple-400" />
                  <h2 className="text-lg font-medium text-white">WSS Endpoint (WebSocket)</h2>
                </div>
                <div className="bg-[#111111] border border-[#333333] p-3 rounded-lg flex items-center justify-between group">
                  <code className="text-sm text-purple-400 truncate">{wssUrl}</code>
                  <button 
                    onClick={() => navigator.clipboard.writeText(wssUrl)}
                    className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[#1A1A1A] opacity-0 group-hover:opacity-100"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>

            {/* Quick Stats */}
            <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-6">Node Status</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Box className="w-3 h-3"/> Block Height</div>
                  <div className="text-2xl font-bold text-white">5,342,109</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Globe className="w-3 h-3"/> Peers Connected</div>
                  <div className="text-2xl font-bold text-white">42</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Terminal className="w-3 h-3"/> Client Version</div>
                  <div className="text-2xl font-bold text-white">neo-go v0.106.0</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Activity className="w-3 h-3"/> Uptime</div>
                  <div className="text-2xl font-bold text-white">99.99%</div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'metrics' && (
          <div className="space-y-6 animate-in fade-in">
            <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6 flex items-center justify-center h-64">
              <div className="text-center text-gray-500">
                <Activity className="w-8 h-8 mx-auto mb-3 opacity-50" />
                <p>System metrics (CPU, Memory, Disk I/O) are initializing...</p>
                <p className="text-xs mt-1">Requires at least 1 hour of data collection.</p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'logs' && (
          <div className="animate-in fade-in">
            <div className="bg-[#111111] border border-[#333333] rounded-xl overflow-hidden font-mono text-sm">
              <div className="bg-[#1A1A1A] border-b border-[#333333] px-4 py-2 flex justify-between items-center">
                <div className="flex space-x-2">
                  <div className="w-3 h-3 rounded-full bg-red-500"></div>
                  <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                  <div className="w-3 h-3 rounded-full bg-green-500"></div>
                </div>
                <span className="text-gray-400 text-xs flex items-center gap-1"><Activity className="w-3 h-3 text-[#00E599]" /> Live Stream</span>
              </div>
              <div className="p-4 h-96 overflow-y-auto text-gray-300 space-y-1">
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:01.123</span><span className="text-blue-400">INFO</span><span>node started, client version: neo-go v0.106.0</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:02.045</span><span className="text-blue-400">INFO</span><span>initializing block store at /data/chain</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:05.882</span><span className="text-[#00E599]">SYNC</span><span>syncing blocks: height 5342100, peers 12</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:08.111</span><span className="text-[#00E599]">SYNC</span><span>syncing blocks: height 5342105, peers 24</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:10.932</span><span className="text-yellow-400">WARN</span><span>slow peer response, dropping connection IP: 45.33.12.X</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:15.001</span><span className="text-[#00E599]">SYNC</span><span>node synchronized. current height: 5342109</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:02:18.553</span><span className="text-blue-400">RPC </span><span>serving RPC at :30333</span></div>
                <div className="flex gap-4"><span className="text-gray-500 shrink-0">14:03:01.000</span><span className="text-purple-400">WS  </span><span>new websocket connection established</span></div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'plugins' && (
          <div className="animate-in fade-in space-y-4">
            <div className="bg-[#1A1A1A] border border-[#00E599]/30 rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center">
              <div className="flex gap-4">
                <div className="p-3 bg-[#00E599]/10 rounded-lg shrink-0">
                  <Lock className="w-6 h-6 text-[#00E599]" />
                </div>
                <div>
                  <h3 className="text-lg font-medium text-white flex items-center gap-2">
                    TEE Privacy Oracle 
                    <span className="text-xs bg-[#00E599]/20 text-[#00E599] px-2 py-0.5 rounded border border-[#00E599]/30">Running</span>
                  </h3>
                  <p className="text-sm text-gray-400 mt-1">Phala CVM instance securely bound to this node.</p>
                </div>
              </div>
              <button className="bg-[#111111] border border-[#333333] hover:bg-[#252525] px-4 py-2 rounded text-sm text-white transition-colors">
                Configure
              </button>
            </div>

            <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center opacity-70">
              <div className="flex gap-4">
                <div className="p-3 bg-gray-800 rounded-lg shrink-0">
                  <Plug className="w-6 h-6 text-gray-400" />
                </div>
                <div>
                  <h3 className="text-lg font-medium text-white">Account Abstraction Bundler</h3>
                  <p className="text-sm text-gray-400 mt-1">Native AA relay services. Currently disabled.</p>
                </div>
              </div>
              <Link href="/marketplace" className="text-[#00E599] hover:underline text-sm font-medium">
                Install from Marketplace
              </Link>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="animate-in fade-in space-y-6">
            <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-4">Node Configuration (neo-go)</h2>
              <p className="text-sm text-gray-400 mb-6">Modify your core Neo node parameters. Node will automatically restart after saving.</p>
              
              <div className="space-y-5">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">Max Connected Peers</label>
                  <input type="number" defaultValue={100} className="w-full md:w-1/3 bg-[#111111] border border-[#333333] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">RPC.MaxGasInvoke</label>
                  <input type="number" defaultValue={10} className="w-full md:w-1/3 bg-[#111111] border border-[#333333] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                  <p className="text-xs text-gray-500 mt-1">Maximum GAS allowed for test invocations (invokefunction).</p>
                </div>
                
                <div className="pt-4 space-y-4">
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[#333333] bg-[#111111] text-[#00E599] focus:ring-[#00E599]" />
                    <span className="text-sm text-gray-300">Enable P2P Notary Request Payload</span>
                  </label>
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[#333333] bg-[#111111] text-[#00E599] focus:ring-[#00E599]" />
                    <span className="text-sm text-gray-300">Enable Extensible Payload (State Root)</span>
                  </label>
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input type="checkbox" className="w-4 h-4 rounded border-[#333333] bg-[#111111] text-[#00E599] focus:ring-[#00E599]" />
                    <span className="text-sm text-gray-300">Enable Oracle Service Node</span>
                  </label>
                </div>

                <div className="pt-6 mt-6 border-t border-[#333333]">
                  <button className="bg-[#00E599] hover:bg-[#00cc88] text-black px-6 py-2 rounded-md font-bold transition-colors">
                    Save Configuration
                  </button>
                </div>
              </div>
            </div>

            <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6">
              <h2 className="text-lg font-medium text-red-400 mb-2">Danger Zone</h2>
              <p className="text-sm text-gray-400 mb-4">Deleting this node is permanent and cannot be undone. You will lose the dedicated IP, URL, and all local node data.</p>
              <button className="bg-red-500 hover:bg-red-600 text-white px-6 py-2 rounded-md font-bold transition-colors">
                Delete Node
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
