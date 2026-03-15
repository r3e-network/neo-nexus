'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { ArrowLeft, Activity, Box, Copy, Globe, MoreVertical, Play, Power, RotateCcw, Server, Terminal, Lock, Plug, Bell, Mail, Webhook, Cpu, HardDrive, RefreshCw, Download } from 'lucide-react';
import { Endpoint } from '../EndpointsList';
import { NeoNodeService } from '@/services/neo/NeoNodeService';
import { addNodePluginAction } from '../pluginActions';
import toast from 'react-hot-toast';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts';

export default function EndpointDetailsClient({ endpoint }: { endpoint: Endpoint | null }) {
  const [activeTab, setActiveTab] = useState('overview');
  const [blockHeight, setBlockHeight] = useState<number | string>('Syncing...');
  const [peerCount, setPeerCount] = useState<number | string>('Syncing...');
  const [installPluginModal, setInstallPluginModal] = useState<string | null>(null);
  const [privateKey, setPrivateKey] = useState('');
  const [isInstalling, setIsInstalling] = useState(false);

  useEffect(() => {
    let interval: NodeJS.Timeout;

    const fetchNodeStats = async () => {
      if (endpoint?.url && endpoint.status.toLowerCase() === 'active') {
        const height = await NeoNodeService.getBlockCount(endpoint.url);
        if (height !== null) setBlockHeight(height.toLocaleString());

        const peers = await NeoNodeService.getPeersCount(endpoint.url);
        if (peers !== null) setPeerCount(peers);
      } else if (endpoint?.status.toLowerCase() !== 'active') {
        setBlockHeight(endpoint?.status || 'Unknown');
        setPeerCount('-');
      }
    };

    fetchNodeStats();
    
    // Poll every 15 seconds if active
    if (endpoint?.status.toLowerCase() === 'active') {
      interval = setInterval(fetchNodeStats, 15000);
    }

    return () => clearInterval(interval);
  }, [endpoint]);

  const handleInstallPlugin = async () => {
    if (!installPluginModal) return;
    if (!privateKey) {
        toast.error('A private key is required to configure this plugin.');
        return;
    }

    setIsInstalling(true);
    toast.loading(`Deploying ${installPluginModal} sidecar...`, { id: 'plugin' });

    const result = await addNodePluginAction(endpoint?.id as number, installPluginModal, {}, privateKey);

    if (result.success) {
        toast.success(`${installPluginModal} installed and configured securely! Node is restarting.`, { id: 'plugin' });
        setInstallPluginModal(null);
        setPrivateKey('');
    } else {
        toast.error(result.error || 'Failed to install plugin.', { id: 'plugin' });
    }
    setIsInstalling(false);
  };

  const tabs = [
    { id: 'overview', name: 'Overview' },
    { id: 'metrics', name: 'Health Metrics' },
    { id: 'logs', name: 'Node Logs' },
    { id: 'plugins', name: 'Plugins' },
    { id: 'alerts', name: 'Alerts' },
    { id: 'settings', name: 'Settings' },
  ];

  const wssUrl = endpoint?.url ? endpoint.url.replace('https://', 'wss://').replace('/v1', '/ws') : 'wss://node-tokyo-01.neonexus.cloud/ws/xyz';


  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <Link href="/app/endpoints" className="inline-flex items-center text-sm text-gray-400 hover:text-white mb-4 transition-colors">
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
            <button className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2">
              <RotateCcw className="w-4 h-4" /> Restart
            </button>
            <button className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2">
              <Power className="w-4 h-4" /> Stop
            </button>
            <button className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-gray-400 p-2 rounded-md transition-colors">
              <MoreVertical className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-[var(--color-dark-border)]">
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
              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Globe className="w-5 h-5 text-blue-400" />
                  <h2 className="text-lg font-medium text-white">HTTPS Endpoint</h2>
                </div>
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-3 rounded-lg flex items-center justify-between group">
                  <code className="text-sm text-[#00E599] truncate">{endpoint?.url || 'https://node-tokyo-01.neonexus.cloud/v1/xyz'}</code>
                  <button 
                    onClick={() => navigator.clipboard.writeText(endpoint?.url || '')}
                    className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[var(--color-dark-panel)] opacity-0 group-hover:opacity-100"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Activity className="w-5 h-5 text-purple-400" />
                  <h2 className="text-lg font-medium text-white">WSS Endpoint (WebSocket)</h2>
                </div>
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-3 rounded-lg flex items-center justify-between group">
                  <code className="text-sm text-purple-400 truncate">{wssUrl}</code>
                  <button 
                    onClick={() => navigator.clipboard.writeText(wssUrl)}
                    className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[var(--color-dark-panel)] opacity-0 group-hover:opacity-100"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>

            {/* Quick Stats */}
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-6">Node Status</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Box className="w-3 h-3"/> Block Height</div>
                  <div className="text-2xl font-bold text-white">{blockHeight}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Globe className="w-3 h-3"/> Peers Connected</div>
                  <div className="text-2xl font-bold text-white">{peerCount}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Terminal className="w-3 h-3"/> Client Engine</div>
                  <div className="text-2xl font-bold text-white">{endpoint?.clientEngine || 'neo-go'}</div>
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
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex justify-between items-center mb-6">
                  <h2 className="text-lg font-medium text-white flex items-center gap-2"><Cpu className="w-5 h-5 text-blue-400" /> CPU Usage</h2>
                  <span className="text-sm font-bold text-white">45%</span>
                </div>
                <div className="h-64 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={[
                      { time: '10:00', value: 30 }, { time: '10:05', value: 45 }, { time: '10:10', value: 38 }, 
                      { time: '10:15', value: 60 }, { time: '10:20', value: 42 }, { time: '10:25', value: 45 }
                    ]}>
                      <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                      <XAxis dataKey="time" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                      <YAxis stroke="#666" fontSize={12} tickLine={false} axisLine={false} domain={[0, 100]} />
                      <Tooltip contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '8px' }} />
                      <Line type="monotone" dataKey="value" stroke="#3b82f6" strokeWidth={2} dot={false} activeDot={{ r: 4 }} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>

              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex justify-between items-center mb-6">
                  <h2 className="text-lg font-medium text-white flex items-center gap-2"><Activity className="w-5 h-5 text-[#00E599]" /> Memory Usage</h2>
                  <span className="text-sm font-bold text-white">4.2 GB / 8 GB</span>
                </div>
                <div className="h-64 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={[
                      { time: '10:00', value: 3.8 }, { time: '10:05', value: 3.9 }, { time: '10:10', value: 4.1 }, 
                      { time: '10:15', value: 4.5 }, { time: '10:20', value: 4.3 }, { time: '10:25', value: 4.2 }
                    ]}>
                      <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                      <XAxis dataKey="time" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                      <YAxis stroke="#666" fontSize={12} tickLine={false} axisLine={false} domain={[0, 8]} />
                      <Tooltip contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '8px' }} />
                      <Line type="monotone" dataKey="value" stroke="#00E599" strokeWidth={2} dot={false} activeDot={{ r: 4 }} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>

              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 lg:col-span-2">
                <div className="flex justify-between items-center mb-6">
                  <h2 className="text-lg font-medium text-white flex items-center gap-2"><HardDrive className="w-5 h-5 text-purple-400" /> Disk I/O (Read/Write)</h2>
                  <span className="text-sm font-bold text-white">2.5 MB/s</span>
                </div>
                <div className="h-64 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={[
                      { time: '10:00', read: 1.2, write: 0.5 }, { time: '10:05', read: 2.5, write: 1.0 }, { time: '10:10', read: 1.8, write: 0.8 }, 
                      { time: '10:15', read: 3.5, write: 2.1 }, { time: '10:20', read: 2.1, write: 0.9 }, { time: '10:25', read: 2.5, write: 1.2 }
                    ]}>
                      <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                      <XAxis dataKey="time" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                      <YAxis stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                      <Tooltip contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '8px' }} />
                      <Line type="monotone" dataKey="read" stroke="#a855f7" strokeWidth={2} dot={false} />
                      <Line type="monotone" dataKey="write" stroke="#ec4899" strokeWidth={2} dot={false} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'logs' && (
          <div className="animate-in fade-in">
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden font-mono text-sm">
              <div className="bg-[var(--color-dark-panel)] border-b border-[var(--color-dark-border)] px-4 py-2 flex justify-between items-center">
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
            <div className="bg-[var(--color-dark-panel)] border border-[#00E599]/30 rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center">
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
              <button 
                onClick={() => setInstallPluginModal('tee-oracle')}
                className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] px-4 py-2 rounded text-sm text-white transition-colors"
              >
                Configure
              </button>
            </div>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex gap-4">
                <div className="p-3 bg-gray-800 rounded-lg shrink-0">
                  <Plug className="w-6 h-6 text-gray-400" />
                </div>
                <div>
                  <h3 className="text-lg font-medium text-white">Account Abstraction Bundler</h3>
                  <p className="text-sm text-gray-400 mt-1">Native AA relay services. Currently disabled.</p>
                </div>
              </div>
              <button 
                onClick={() => setInstallPluginModal('aa-bundler')}
                className="text-[#00E599] bg-[#00E599]/10 border border-[#00E599]/20 hover:bg-[#00E599]/20 px-4 py-2 rounded text-sm font-medium transition-colors"
              >
                Install Plugin
              </button>
            </div>
          </div>
        )}

        {activeTab === 'alerts' && (
          <div className="animate-in fade-in space-y-6">
            <div className="flex justify-between items-center">
              <div>
                <h2 className="text-lg font-medium text-white">Alert Rules</h2>
                <p className="text-sm text-gray-400 mt-1">Get notified when your node requires attention.</p>
              </div>
              <button 
                onClick={() => toast('Alert rule creation modal would open here.', { icon: '🔔' })}
                className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
              >
                Create Alert
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 relative overflow-hidden">
                <div className="flex justify-between items-start mb-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-red-500/10 rounded-lg">
                      <Activity className="w-5 h-5 text-red-500" />
                    </div>
                    <h3 className="font-bold text-white">Node Down</h3>
                  </div>
                  <span className="bg-[#00E599]/20 text-[#00E599] text-xs px-2 py-1 rounded font-bold">ACTIVE</span>
                </div>
                <p className="text-sm text-gray-400 mb-4">Triggers if the RPC endpoint becomes unresponsive for more than 1 minute.</p>
                <div className="flex items-center gap-2 text-sm text-gray-300 bg-[var(--color-dark-panel)] p-2 rounded-md border border-[var(--color-dark-border)]">
                  <Mail className="w-4 h-4 text-gray-500" /> dev@neonexus.cloud
                </div>
              </div>

              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 relative overflow-hidden">
                <div className="flex justify-between items-start mb-4">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-yellow-500/10 rounded-lg">
                      <Box className="w-5 h-5 text-yellow-500" />
                    </div>
                    <h3 className="font-bold text-white">Sync Lag</h3>
                  </div>
                  <span className="bg-[#00E599]/20 text-[#00E599] text-xs px-2 py-1 rounded font-bold">ACTIVE</span>
                </div>
                <p className="text-sm text-gray-400 mb-4">Triggers if the node falls behind the main network by &gt; 100 blocks.</p>
                <div className="flex items-center gap-2 text-sm text-gray-300 bg-[var(--color-dark-panel)] p-2 rounded-md border border-[var(--color-dark-border)]">
                  <Webhook className="w-4 h-4 text-gray-500" /> https://api.pagerduty.com/...
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="animate-in fade-in space-y-6">
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <div className="flex items-center gap-3 mb-4">
                 <h2 className="text-lg font-medium text-white">Node Configuration</h2>
                 <span className="bg-gray-800 text-gray-300 text-xs px-2 py-0.5 rounded font-mono">{endpoint?.clientEngine || 'neo-go'}</span>
              </div>
              <p className="text-sm text-gray-400 mb-6">Modify your core Neo node parameters. Changes are automatically injected into the Kubernetes ConfigMap and the node will restart.</p>
              
              {endpoint?.clientEngine === 'neo-x-geth' ? (
                 <div className="space-y-5">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">Max Connected Peers (--maxpeers)</label>
                    <input type="number" defaultValue={50} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">EVM Cache Size (--cache)</label>
                    <input type="number" defaultValue={4096} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                    <p className="text-xs text-gray-500 mt-1">Megabytes of memory allocated to internal caching (default: 4096).</p>
                  </div>
                  <div className="pt-4 space-y-4">
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">Enable WebSocket API (--ws)</span>
                    </label>
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">Enable GraphQL API (--graphql)</span>
                    </label>
                  </div>
                </div>
              ) : (
                <div className="space-y-5">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">Max Connected Peers</label>
                    <input type="number" defaultValue={100} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">RPC.MaxGasInvoke</label>
                    <input type="number" defaultValue={10} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                    <p className="text-xs text-gray-500 mt-1">Maximum GAS allowed for test invocations (invokefunction).</p>
                  </div>
                  
                  <div className="pt-4 space-y-4">
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">Enable P2P Notary Request Payload</span>
                    </label>
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">Enable Extensible Payload (State Root)</span>
                    </label>
                    {endpoint?.clientEngine === 'neo-cli' && (
                        <label className="flex items-center gap-3 cursor-pointer">
                        <input type="checkbox" className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                        <span className="text-sm text-gray-300">Enable DBFT Consensus Logging</span>
                        </label>
                    )}
                  </div>
                </div>
              )}

              <div className="pt-6 mt-6 border-t border-[var(--color-dark-border)]">
                <button 
                  onClick={() => {
                    toast.loading('Applying configuration to cluster ConfigMap...', { id: 'config' });
                    setTimeout(() => toast.success('Configuration saved. Node is restarting.', { id: 'config' }), 1500);
                  }}
                  className="bg-[#00E599] hover:bg-[#00cc88] text-black px-6 py-2 rounded-md font-bold transition-colors"
                >
                  Save Configuration
                </button>
              </div>
            </div>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-4">Maintenance Actions</h2>
              <p className="text-sm text-gray-400 mb-6">Perform advanced lifecycle operations on your node storage and state.</p>
              
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="border border-[var(--color-dark-border)] rounded-lg p-4 flex flex-col justify-between">
                   <div className="mb-4">
                     <h3 className="font-bold text-white flex items-center gap-2 mb-1"><RefreshCw className="w-4 h-4 text-blue-400" /> Fast Resync</h3>
                     <p className="text-xs text-gray-400">Purges local ledger data and automatically redownloads the latest official Neo Genesis snapshot.</p>
                   </div>
                   <button 
                     onClick={() => {
                        if (confirm('Are you sure you want to purge data and resync from snapshot? This will cause a few minutes of downtime.')) {
                            toast.loading('Purging PVC and triggering init-container...', { id: 'maint' });
                            setTimeout(() => toast.success('Node is resyncing from snapshot.', { id: 'maint' }), 2000);
                        }
                     }}
                     className="bg-[var(--color-dark-panel)] hover:bg-[#222222] border border-[var(--color-dark-border)] text-white px-4 py-2 rounded-md text-sm font-bold transition-colors w-full"
                   >
                     Trigger Resync
                   </button>
                </div>
                <div className="border border-[var(--color-dark-border)] rounded-lg p-4 flex flex-col justify-between">
                   <div className="mb-4">
                     <h3 className="font-bold text-white flex items-center gap-2 mb-1"><Download className="w-4 h-4 text-green-400" /> Export Snapshot</h3>
                     <p className="text-xs text-gray-400">Creates a point-in-time tarball of your Node's persistent volume and generates a signed S3 download link.</p>
                   </div>
                   <button 
                     onClick={() => {
                        toast.loading('Initiating volume snapshot... This may take up to 10 minutes depending on size.', { id: 'maint' });
                        setTimeout(() => toast.success('Snapshot queued! You will receive an email when ready.', { id: 'maint' }), 2000);
                     }}
                     className="bg-[var(--color-dark-panel)] hover:bg-[#222222] border border-[var(--color-dark-border)] text-white px-4 py-2 rounded-md text-sm font-bold transition-colors w-full"
                   >
                     Export Data
                   </button>
                </div>
              </div>
            </div>

            <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6">
              <h2 className="text-lg font-medium text-red-400 mb-2">Danger Zone</h2>
              <p className="text-sm text-gray-400 mb-4">Deleting this node is permanent and cannot be undone. You will lose the dedicated IP, URL, and all local node data (PVCs will be destroyed).</p>
              <button 
                onClick={() => {
                   if (confirm('Are you absolutely sure you want to destroy this node?')) {
                       toast.loading('Destroying Kubernetes resources...', { id: 'delete' });
                       setTimeout(() => {
                           toast.success('Node destroyed successfully.', { id: 'delete' });
                           window.location.href = '/app/endpoints';
                       }, 2000);
                   }
                }}
                className="bg-red-500 hover:bg-red-600 text-white px-6 py-2 rounded-md font-bold transition-colors"
              >
                Delete Node
              </button>
            </div>
          </div>
        )}
      </div>

      {installPluginModal && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
            <button onClick={() => { setInstallPluginModal(null); setPrivateKey(''); }} className="absolute top-4 right-4 text-gray-500 hover:text-white">✕</button>
            <h3 className="text-xl font-bold text-white mb-2">
              Configure {installPluginModal === 'tee-oracle' ? 'TEE Oracle' : (installPluginModal === 'tee-mempool' ? 'TEE Protected Mempool' : 'AA Bundler')}
            </h3>
            <p className="text-sm text-gray-400 mb-6">
              This plugin requires a wallet to sign transactions on-chain. Please provide the private key. It will be securely stored in our vault and mounted to your cluster.
            </p>
            
            <div className="space-y-4 mb-6">
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1">Neo N3 Private Key (WIF)</label>
                <input 
                  type="password" 
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
                  placeholder="L1K..." 
                  className="w-full bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#00E599] transition-colors"
                />
              </div>
            </div>

            <button 
              onClick={handleInstallPlugin}
              disabled={isInstalling || !privateKey}
              className="w-full bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)]"
            >
              {isInstalling ? 'Installing...' : 'Save securely & Deploy'}
            </button>
            <p className="text-xs text-center text-gray-500 mt-4 flex items-center justify-center gap-1">
              <Lock className="w-3 h-3" /> Encrypted and stored via AWS KMS
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
