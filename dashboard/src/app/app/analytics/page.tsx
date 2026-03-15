'use client';

import { Activity, KeyRound, Server, TimerReset } from 'lucide-react';
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import useSWR from 'swr';

type AnalyticsResponse = {
  stats: {
    totalRequests: string;
    activeEndpoints: number;
    syncingEndpoints: number;
    apiKeys: number;
  };
  endpointUsageData: Array<{
    name: string;
    requests: number;
    status: string;
  }>;
  networkData: Array<{
    name: string;
    requests: number;
    endpoints: number;
  }>;
  recentEvents: Array<{
    time: string;
    category: 'Endpoint' | 'API Key' | 'Plugin';
    detail: string;
    status: string;
  }>;
};

const EMPTY_ANALYTICS: AnalyticsResponse = {
  stats: {
    totalRequests: '0',
    activeEndpoints: 0,
    syncingEndpoints: 0,
    apiKeys: 0,
  },
  endpointUsageData: [],
  networkData: [],
  recentEvents: [],
};

const fetcher = (url: string) => fetch(url).then((res) => res.json());

function formatEventTime(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(date);
}

export default function Analytics() {
  const { data = EMPTY_ANALYTICS, isLoading } = useSWR<AnalyticsResponse>('/api/metrics', fetcher, {
    refreshInterval: 10000,
    fallbackData: EMPTY_ANALYTICS,
  });

  return (
    <div className="min-h-screen pb-12 space-y-8">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white mb-2 flex items-center gap-3">
            Metrics & Analytics
            {isLoading && <span className="w-2 h-2 rounded-full bg-[#00E599] animate-pulse"></span>}
          </h1>
          <p className="text-gray-400 text-lg">
            Organization analytics derived from persisted endpoint, plugin, and access-key data.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-6">
        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Total Requests</span>
            <div className="p-2 bg-blue-500/10 rounded-lg">
              <Activity className="w-4 h-4 text-blue-400" />
            </div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.totalRequests}</div>
          <div className="text-sm text-gray-500">Summed from persisted endpoint usage counters.</div>
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Active Endpoints</span>
            <div className="p-2 bg-[#00E599]/10 rounded-lg">
              <Server className="w-4 h-4 text-[#00E599]" />
            </div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.activeEndpoints}</div>
          <div className="text-sm text-gray-500">Endpoints currently marked as Active.</div>
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">Syncing Endpoints</span>
            <div className="p-2 bg-yellow-500/10 rounded-lg">
              <TimerReset className="w-4 h-4 text-yellow-500" />
            </div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.syncingEndpoints}</div>
          <div className="text-sm text-gray-500">Infrastructure still catching up or provisioning.</div>
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl">
          <div className="flex justify-between items-start mb-4">
            <span className="text-sm font-medium text-gray-400">API Keys</span>
            <div className="p-2 bg-purple-500/10 rounded-lg">
              <KeyRound className="w-4 h-4 text-purple-400" />
            </div>
          </div>
          <div className="text-3xl font-bold text-white mb-2">{data.stats.apiKeys}</div>
          <div className="text-sm text-gray-500">Active access credentials registered for this organization.</div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-6">
          <h2 className="text-lg font-bold text-white mb-6">Requests by Network</h2>
          <div className="h-80 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={data.networkData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                <defs>
                  <linearGradient id="networkRequests" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#00E599" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#00E599" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                <XAxis dataKey="name" stroke="#666" fontSize={12} tickLine={false} axisLine={false} dy={10} />
                <YAxis yAxisId="left" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                <YAxis yAxisId="right" orientation="right" stroke="#666" fontSize={12} tickLine={false} axisLine={false} />
                <Tooltip
                  contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '12px' }}
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
                  fill="url(#networkRequests)"
                />
                <Area
                  yAxisId="right"
                  type="monotone"
                  dataKey="endpoints"
                  stroke="#60a5fa"
                  strokeWidth={2}
                  fill="none"
                  strokeDasharray="5 5"
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-6 flex flex-col">
          <h2 className="text-lg font-bold text-white mb-6">Top Endpoints by Requests</h2>
          <div className="flex-1 w-full min-h-[250px]">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={data.endpointUsageData} layout="vertical" margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
                <XAxis type="number" hide />
                <YAxis dataKey="name" type="category" width={120} stroke="#888" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip
                  cursor={{ fill: '#222' }}
                  contentStyle={{ backgroundColor: '#111', border: '1px solid #333', borderRadius: '8px' }}
                />
                <Bar dataKey="requests" radius={[0, 4, 4, 0]} barSize={20}>
                  {data.endpointUsageData.map((entry, index) => (
                    <Cell
                      key={`${entry.name}-${index}`}
                      fill={entry.status === 'Active' ? '#00E599' : '#334155'}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl overflow-hidden">
        <div className="px-6 py-5 border-b border-[var(--color-dark-border)] flex justify-between items-center">
          <h2 className="text-lg font-bold text-white">Recent Activity</h2>
          <span className="text-sm text-gray-500">Latest persisted infrastructure events</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="bg-[var(--color-dark-panel)] text-gray-400">
              <tr>
                <th className="px-6 py-3 font-medium">Time</th>
                <th className="px-6 py-3 font-medium">Category</th>
                <th className="px-6 py-3 font-medium">Detail</th>
                <th className="px-6 py-3 font-medium">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#333333] text-gray-300">
              {data.recentEvents.length === 0 ? (
                <tr>
                  <td className="px-6 py-6 text-gray-500" colSpan={4}>
                    No organization activity has been persisted yet.
                  </td>
                </tr>
              ) : (
                data.recentEvents.map((event) => (
                  <tr key={`${event.category}-${event.time}-${event.detail}`} className="hover:bg-[#222222] transition-colors">
                    <td className="px-6 py-4 text-gray-500">{formatEventTime(event.time)}</td>
                    <td className="px-6 py-4 font-medium">{event.category}</td>
                    <td className="px-6 py-4">{event.detail}</td>
                    <td className="px-6 py-4">{event.status}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
