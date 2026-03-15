'use client';

import { Activity, ArrowUpRight, ArrowDownRight, Clock, ServerCrash, ShieldAlert } from 'lucide-react';
import { 
  AreaChart, Area, XAxis, YAxis, Tooltip as LineTooltip, ResponsiveContainer as RCLine, CartesianGrid,
  BarChart, Bar, ResponsiveContainer as RCBar, Cell
} from 'recharts';
import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export default function Analytics() {
  // In a real production app, this would fetch from a Next.js API route that queries 
  // Prometheus/Grafana or an APISIX logging endpoint. We'll use SWR for automatic polling.
  const { data, error, isLoading } = useSWR('/api/metrics', fetcher, { 
    refreshInterval: 10000,
    fallbackData: {
      latencyData: [
        { time: '00:00', ms: 45, requests: 2100 },
        { time: '04:00', ms: 52, requests: 4300 },
        { time: '08:00', ms: 48, requests: 8500 },
        { time: '12:00', ms: 120, requests: 12400 },
        { time: '16:00', ms: 65, requests: 9400 },
        { time: '20:00', ms: 43, requests: 6200 },
        { time: '24:00', ms: 40, requests: 3100 },
      ],
      methodData: [
        { name: 'invokefunction', value: 125000 },
        { name: 'getapplicationlog', value: 85000 },
        { name: 'getnep17balances', value: 45000 },
        { name: 'getblock', value: 25000 },
        { name: 'getversion', value: 5000 },
      ],
      stats: {
        totalRequests: '1.24M',
        successRate: '99.8%',
        avgLatency: 48,
        bandwidth: '42.5'
      },
      errorLog: [
        { time: '10 mins ago', method: 'invokefunction', error: '-500: Invalid parameters', ip: '45.22.11.X' },
        { time: '1 hour ago', method: 'getapplicationlog', error: '-100: Unknown transaction', ip: '192.168.1.X' },
        { time: '3 hours ago', method: 'sendrawtransaction', error: '-501: Insufficient GAS', ip: '8.8.4.X' },
      ]
    }
  });

  return (
    <div className="min-h-screen pb-12 space-y-8">
      {/* Header & Filters */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white mb-2 flex items-center gap-3">
            Metrics & Analytics
            {isLoading && <span className="w-2 h-2 rounded-full bg-[#00E599] animate-pulse"></span>}
          </h1>
          <p className="text-gray-400 text-lg">Deep insight into your Neo infrastructure health.</p>
        </div>
        <div className="bg-[#23252E] border border-[#2E313D] p-1 rounded-lg flex text-sm">
          <button className="px-4 py-1.5 rounded-md text-gray-400 hover:text-white transition-colors">24H</button>
          <button className="px-4 py-1.5 rounded-md bg-[#333333] text-white shadow font-medium">7D</button>
          <button className="px-4 py-1.5 rounded-md text-gray-400 hover:text-white transition-colors">30D</button>
        </div>
      </div>

      {/* Top Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-[#23252E] border border-[#2E313D] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Total Requests</span>
            <div className="p-2 bg-blue-500/10 rounded-lg"><Activity className="w-4 h-4 text-blue-400" /></div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.totalRequests}</div>
          <div className="flex items-center text-sm text-[#00E599]">
            <ArrowUpRight className="w-4 h-4 mr-1" /> 12.5% vs last week
          </div>
        </div>

        <div className="bg-[#23252E] border border-[#2E313D] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Success Rate</span>
            <div className="p-2 bg-[#00E599]/10 rounded-lg"><ShieldAlert className="w-4 h-4 text-[#00E599]" /></div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.successRate}</div>
          <div className="flex items-center text-sm text-gray-500">
            0.2% error rate (mostly 4xx)
          </div>
        </div>

        <div className="bg-[#23252E] border border-[#2E313D] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Avg. Latency</span>
            <div className="p-2 bg-yellow-500/10 rounded-lg"><Clock className="w-4 h-4 text-yellow-500" /></div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.avgLatency}<span className="text-lg text-gray-400 ml-1">ms</span></div>
          <div className="flex items-center text-sm text-red-400">
            <ArrowDownRight className="w-4 h-4 mr-1" /> Spike at 12:00
          </div>
        </div>

        <div className="bg-[#23252E] border border-[#2E313D] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Bandwidth</span>
            <div className="p-2 bg-purple-500/10 rounded-lg"><ServerCrash className="w-4 h-4 text-purple-400" /></div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.bandwidth}<span className="text-lg text-gray-400 ml-1">GB</span></div>
          <div className="flex items-center text-sm text-[#00E599]">
            <ArrowUpRight className="w-4 h-4 mr-1" /> 5.2% vs last week
          </div>
        </div>
      </div>

      {/* Main Charts Area */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        
        {/* Large Area Chart: Volume & Latency */}
        <div className="lg:col-span-2 bg-[#23252E] border border-[#2E313D] rounded-2xl p-6">
          <h2 className="text-lg font-bold text-white mb-6">Request Volume & Latency</h2>
          <div className="h-80 w-full">
            <RCLine width="100%" height="100%">
              <AreaChart data={data.latencyData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#00E599" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#00E599" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                <XAxis dataKey="time" stroke="#666" fontSize={12} tickLine={false} axisLine={false} dy={10} />
                <YAxis yAxisId="left" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                <YAxis yAxisId="right" orientation="right" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                <LineTooltip 
                  contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '12px', boxShadow: '0 10px 25px -5px rgba(0, 0, 0, 0.5)' }}
                  itemStyle={{ color: '#fff' }}
                  labelStyle={{ color: '#888', marginBottom: '4px' }}
                />
                <Area 
                  yAxisId="left"
                  type="monotone" 
                  dataKey="requests" 
                  stroke="#00E599" 
                  strokeWidth={3}
                  fillOpacity={1} 
                  fill="url(#colorRequests)" 
                  animationDuration={500}
                />
                <Area 
                  yAxisId="right"
                  type="monotone" 
                  dataKey="ms" 
                  stroke="#f59e0b" 
                  strokeWidth={2}
                  fill="none" 
                  strokeDasharray="5 5"
                  animationDuration={500}
                />
              </AreaChart>
            </RCLine>
          </div>
        </div>

        {/* Bar Chart: RPC Methods */}
        <div className="bg-[#23252E] border border-[#2E313D] rounded-2xl p-6 flex flex-col">
          <h2 className="text-lg font-bold text-white mb-6">Top RPC Methods</h2>
          <div className="flex-1 w-full min-h-[250px]">
            <RCBar width="100%" height="100%">
              <BarChart data={data.methodData} layout="vertical" margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
                <XAxis type="number" hide />
                <YAxis dataKey="name" type="category" width={120} stroke="#888" fontSize={11} tickLine={false} axisLine={false} />
                <LineTooltip 
                  cursor={{fill: '#222'}}
                  contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '8px' }}
                />
                <Bar dataKey="value" radius={[0, 4, 4, 0]} barSize={20} animationDuration={500}>
                  {data.methodData.map((entry: any, index: number) => (
                    <Cell key={`cell-${index}`} fill={index === 0 ? '#00E599' : '#334155'} />
                  ))}
                </Bar>
              </BarChart>
            </RCBar>
          </div>
        </div>
      </div>

      {/* Error Logs Table */}
      <div className="bg-[#23252E] border border-[#2E313D] rounded-2xl overflow-hidden">
        <div className="px-6 py-5 border-b border-[#2E313D] flex justify-between items-center">
          <h2 className="text-lg font-bold text-white">Recent Errors</h2>
          <button className="text-sm text-[#00E599] hover:text-[#00cc88] font-medium">View All Logs</button>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="bg-[#1A1C23] text-gray-400">
              <tr>
                <th className="px-6 py-3 font-medium">Time</th>
                <th className="px-6 py-3 font-medium">RPC Method</th>
                <th className="px-6 py-3 font-medium">Error Code / Message</th>
                <th className="px-6 py-3 font-medium">Source IP</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#333333] text-gray-300">
              {data.errorLog.map((log: any, i: number) => (
                <tr key={i} className="hover:bg-[#222222] transition-colors">
                  <td className="px-6 py-4 text-gray-500">{log.time}</td>
                  <td className="px-6 py-4 font-mono text-xs">{log.method}</td>
                  <td className="px-6 py-4 text-red-400">{log.error}</td>
                  <td className="px-6 py-4 font-mono text-xs text-gray-500">{log.ip}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

    </div>
  );
}
