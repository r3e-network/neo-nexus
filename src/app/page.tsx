'use client';

import { Activity, ArrowRight, CheckCircle2, ChevronRight, Copy, CreditCard, ExternalLink, Plus, Server, ShieldCheck, Zap } from 'lucide-react';
import { AreaChart, Area, ResponsiveContainer, XAxis, Tooltip } from 'recharts';
import Link from 'next/link';

const mockChartData = [
  { time: 'Mon', requests: 120000 },
  { time: 'Tue', requests: 250000 },
  { time: 'Wed', requests: 180000 },
  { time: 'Thu', requests: 380000 },
  { time: 'Fri', requests: 290000 },
  { time: 'Sat', requests: 420000 },
  { time: 'Sun', requests: 310000 },
];

export default function Overview() {
  return (
    <div className="min-h-screen pb-12 space-y-8">
      {/* Welcome Header */}
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Welcome back, Neo Dev.</h1>
          <p className="text-gray-400 text-lg">Here's what's happening with your infrastructure today.</p>
        </div>
        <div className="flex items-center gap-3">
          <div className="bg-[#1A1A1A] border border-[#333333] px-4 py-2 rounded-lg flex items-center gap-3">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-[#00E599] animate-pulse"></div>
              <span className="text-sm text-gray-300 font-medium">N3 Mainnet</span>
            </div>
            <span className="text-gray-600">|</span>
            <span className="text-sm text-[#00E599] font-medium">Operational</span>
          </div>
          <Link href="/endpoints/new" className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-lg text-sm font-bold transition-colors flex items-center gap-2">
            <Plus className="w-4 h-4" /> Deploy Node
          </Link>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column (Wider) */}
        <div className="lg:col-span-2 space-y-6">
          
          {/* Quick Stats & Usage */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <div className="bg-[#1A1A1A] border border-[#333333] p-6 rounded-2xl flex flex-col justify-between">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h3 className="text-gray-400 font-medium mb-1">Total RPC Requests</h3>
                  <div className="text-3xl font-bold text-white">1,950,000</div>
                </div>
                <div className="p-3 bg-blue-500/10 rounded-xl text-blue-400">
                  <Activity className="w-6 h-6" />
                </div>
              </div>
              <div className="h-16 w-full -ml-2">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={mockChartData}>
                    <defs>
                      <linearGradient id="colorReq" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                        <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                      </linearGradient>
                    </defs>
                    <Tooltip 
                      contentStyle={{ backgroundColor: '#111', border: 'none', borderRadius: '8px', fontSize: '12px' }}
                      itemStyle={{ color: '#3b82f6' }}
                      cursor={{ stroke: '#333' }}
                    />
                    <Area type="monotone" dataKey="requests" stroke="#3b82f6" strokeWidth={2} fillOpacity={1} fill="url(#colorReq)" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>

            <div className="bg-[#1A1A1A] border border-[#333333] p-6 rounded-2xl flex flex-col justify-between">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h3 className="text-gray-400 font-medium mb-1">Shared Plan Usage</h3>
                  <div className="text-3xl font-bold text-white">39%</div>
                </div>
                <div className="p-3 bg-[#00E599]/10 rounded-xl text-[#00E599]">
                  <Zap className="w-6 h-6" />
                </div>
              </div>
              <div>
                <div className="flex justify-between text-sm text-gray-400 mb-2">
                  <span>1.95M requests</span>
                  <span>5M limit</span>
                </div>
                <div className="w-full bg-[#111111] rounded-full h-2.5 overflow-hidden border border-[#333333]">
                  <div className="bg-[#00E599] h-2.5 rounded-full" style={{ width: '39%' }}></div>
                </div>
                <p className="text-xs text-gray-500 mt-3 flex items-center gap-1">
                  Resets in 12 days <CreditCard className="w-3 h-3" />
                </p>
              </div>
            </div>
          </div>

          {/* Active Endpoints Quick View */}
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-2xl overflow-hidden">
            <div className="px-6 py-5 border-b border-[#333333] flex justify-between items-center bg-[#111111]/50">
              <h2 className="text-lg font-bold text-white">Your Endpoints</h2>
              <Link href="/endpoints" className="text-sm text-[#00E599] hover:text-[#00cc88] font-medium flex items-center">
                View All <ChevronRight className="w-4 h-4" />
              </Link>
            </div>
            <div className="divide-y divide-[#333333]">
              <div className="p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 hover:bg-[#111111]/30 transition-colors">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-lg bg-[#00E599]/10 border border-[#00E599]/20 flex items-center justify-center text-[#00E599]">
                    <Server className="w-5 h-5" />
                  </div>
                  <div>
                    <h3 className="font-bold text-white flex items-center gap-2">
                      GameFi Indexer Node
                      <span className="px-2 py-0.5 text-[10px] font-bold bg-[#00E599]/20 text-[#00E599] rounded">DEDICATED</span>
                    </h3>
                    <p className="text-xs text-gray-400 mt-1">AWS Tokyo • Full Node • N3 Mainnet</p>
                  </div>
                </div>
                <div className="flex items-center gap-3 w-full sm:w-auto">
                  <div className="bg-[#111111] border border-[#333333] px-3 py-1.5 rounded flex items-center gap-2 max-w-[200px] overflow-hidden">
                    <div className="w-2 h-2 rounded-full bg-[#00E599]"></div>
                    <code className="text-xs text-gray-300 truncate">https://node-tokyo-01.neonexus.io/v1</code>
                  </div>
                  <button className="p-1.5 text-gray-500 hover:text-white bg-[#111111] border border-[#333333] rounded transition-colors">
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div className="p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 hover:bg-[#111111]/30 transition-colors">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-lg bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
                    <Zap className="w-5 h-5" />
                  </div>
                  <div>
                    <h3 className="font-bold text-white flex items-center gap-2">
                      Default Shared Endpoint
                      <span className="px-2 py-0.5 text-[10px] font-bold bg-blue-500/20 text-blue-400 rounded">SHARED</span>
                    </h3>
                    <p className="text-xs text-gray-400 mt-1">Global Routing • N3 Mainnet</p>
                  </div>
                </div>
                <div className="flex items-center gap-3 w-full sm:w-auto">
                  <div className="bg-[#111111] border border-[#333333] px-3 py-1.5 rounded flex items-center gap-2 max-w-[200px] overflow-hidden">
                    <div className="w-2 h-2 rounded-full bg-[#00E599]"></div>
                    <code className="text-xs text-gray-300 truncate">https://mainnet.neonexus.io/v1/a8f...</code>
                  </div>
                  <button className="p-1.5 text-gray-500 hover:text-white bg-[#111111] border border-[#333333] rounded transition-colors">
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Right Column (Sidebar) */}
        <div className="space-y-6">
          {/* Security & Firewall Box */}
          <div className="bg-gradient-to-br from-[#1A1A1A] to-[#111111] border border-[#333333] rounded-2xl p-6 relative overflow-hidden">
            <ShieldCheck className="absolute -bottom-4 -right-4 w-32 h-32 text-white/[0.02]" />
            <h2 className="text-lg font-bold text-white mb-2 flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-[#00E599]" /> Security Firewall
            </h2>
            <p className="text-sm text-gray-400 mb-6">Your endpoints are currently protected by IP Allowlist.</p>
            
            <div className="space-y-3 mb-6">
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-400">Allowed IPs</span>
                <span className="font-medium text-white">2 rules active</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-400">Method Blocklist</span>
                <span className="font-medium text-yellow-500">Not configured</span>
              </div>
            </div>

            <Link href="/security" className="block w-full text-center bg-[#333333] hover:bg-[#444444] text-white py-2 rounded-lg text-sm font-medium transition-colors">
              Configure Security
            </Link>
          </div>

          {/* Changelog & Updates */}
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-2xl p-6">
            <h2 className="text-lg font-bold text-white mb-6">Latest Updates</h2>
            <div className="space-y-6">
              <div className="relative pl-4 border-l-2 border-[#333333]">
                <div className="absolute w-2.5 h-2.5 bg-[#00E599] rounded-full -left-[6px] top-1"></div>
                <h4 className="text-sm font-bold text-white">Neo-Go v0.106.0 Deployed</h4>
                <p className="text-xs text-gray-400 mt-1">All dedicated nodes have been patched with the latest RPC optimizations.</p>
                <span className="text-[10px] text-gray-500 mt-2 block">2 days ago</span>
              </div>
              <div className="relative pl-4 border-l-2 border-[#333333]">
                <div className="absolute w-2.5 h-2.5 bg-blue-500 rounded-full -left-[6px] top-1"></div>
                <h4 className="text-sm font-bold text-white">New TEE Oracle Plugin</h4>
                <p className="text-xs text-gray-400 mt-1">You can now bind Phala CVM to your dedicated nodes natively.</p>
                <span className="text-[10px] text-gray-500 mt-2 block">1 week ago</span>
              </div>
              <div className="relative pl-4 border-l-2 border-transparent">
                <div className="absolute w-2.5 h-2.5 bg-gray-500 rounded-full -left-[6px] top-1"></div>
                <h4 className="text-sm font-bold text-white">Frankfurt Region Added</h4>
                <p className="text-xs text-gray-400 mt-1">EU-Central-1 is now available for AWS dedicated deployments.</p>
                <span className="text-[10px] text-gray-500 mt-2 block">2 weeks ago</span>
              </div>
            </div>
            <button className="mt-6 text-sm text-[#00E599] flex items-center gap-1 hover:underline font-medium">
              Read full changelog <ExternalLink className="w-3 h-3" />
            </button>
          </div>
        </div>

      </div>
    </div>
  );
}
