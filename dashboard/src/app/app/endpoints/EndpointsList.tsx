'use client';

import { Copy, Plus, Server, Settings, SignalHigh } from 'lucide-react';
import Link from 'next/link';
import { deriveEndpointConnectionDisplay } from '@/services/endpoints/EndpointConnectionDisplay';

export type Endpoint = {
  id: string | number;
  name: string;
  protocol?: string;
  networkKey?: string;
  network: string;
  type: string;
  url: string;
  wssUrl?: string | null;
  customDomain?: string | null;
  providerPublicIp?: string | null;
  status: string;
  requests: string | number;
  clientEngine?: string;
  syncMode?: string | null;
  cloudProvider?: string | null;
  region?: string | null;
};

export default function EndpointsList({ endpoints }: { endpoints: Endpoint[] }) {
  return (
    <div className="space-y-8">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Endpoints</h1>
          <p className="text-gray-400 mt-1">Manage your Neo N3 and Neo X shared and dedicated RPC nodes.</p>
        </div>
        <Link href="/app/endpoints/new" className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-md text-sm font-bold transition-colors flex items-center gap-2">
          <Plus className="w-4 h-4" />
          Create Dedicated Node
        </Link>
      </div>

      <div className="space-y-4">
        {endpoints.map((ep) => {
          const connectionDisplay = deriveEndpointConnectionDisplay({
            status: ep.status,
            url: ep.url,
            wssUrl: ep.wssUrl,
          });

          return (
            <div key={ep.id} className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between">
              <div className="flex items-start gap-4">
                <div className={`p-3 rounded-lg ${ep.type.toLowerCase() === 'dedicated' ? 'bg-blue-500/10 text-blue-400' : 'bg-[#00E599]/10 text-[#00E599]'}`}>
                  <Server className="w-6 h-6" />
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1">
                    <h3 className="text-lg font-medium text-white">{ep.name}</h3>
                    <span className={`text-xs px-2 py-1 rounded-full border ${ep.status.toLowerCase() === 'active' ? 'border-[#00E599]/30 text-[#00E599] bg-[#00E599]/10' : 'border-yellow-500/30 text-yellow-500 bg-yellow-500/10'}`}>
                      {ep.status}
                    </span>
                  </div>
                  <div className="text-sm text-gray-400 flex items-center gap-3">
                    <span>{ep.network}</span>
                    <span>•</span>
                    <span className="capitalize">{ep.type}</span>
                    <span>•</span>
                    <span className="flex items-center gap-1"><SignalHigh className="w-3 h-3" /> {ep.requests} reqs/mo</span>
                  </div>
                </div>
              </div>

              <div className="flex flex-col sm:flex-row items-center gap-3 w-full md:w-auto">
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] px-3 py-2 rounded-md flex-1 md:w-64 flex items-center justify-between gap-3 group">
                  <div className="min-w-0">
                    <div className={`text-[10px] uppercase tracking-wide font-bold mb-1 ${
                      connectionDisplay.readiness === 'live'
                        ? 'text-[#00E599]'
                        : connectionDisplay.readiness === 'planned'
                          ? 'text-yellow-300'
                          : connectionDisplay.readiness === 'offline'
                            ? 'text-red-300'
                            : 'text-gray-500'
                    }`}>
                      {connectionDisplay.label}
                    </div>
                    <code className="text-sm text-gray-300 truncate block w-40">{connectionDisplay.httpsUrl ?? 'Waiting for route assignment'}</code>
                  </div>
                  {connectionDisplay.httpsUrl && (
                    <button 
                      onClick={() => navigator.clipboard.writeText(connectionDisplay.httpsUrl ?? '')}
                      className="text-gray-500 hover:text-white transition-colors shrink-0"
                      title="Copy URL"
                    >
                      <Copy className="w-4 h-4" />
                    </button>
                  )}
                </div>
                <Link href={`/app/endpoints/${ep.id}`} className="bg-[#333333] hover:bg-[#444444] text-white p-2 rounded-md transition-colors">
                  <Settings className="w-4 h-4" />
                </Link>
              </div>
            </div>
          );
        })}

        {endpoints.length === 0 && (
          <div className="text-center py-12 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl">
            <Server className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No endpoints found</h3>
            <p className="text-gray-400 mb-6">You haven&apos;t deployed any nodes yet.</p>
            <Link href="/app/endpoints/new" className="inline-flex bg-[#00E599] hover:bg-[#00cc88] text-black px-6 py-2 rounded-md text-sm font-bold transition-colors items-center gap-2">
              <Plus className="w-4 h-4" /> Deploy your first node
            </Link>
          </div>
        )}
      </div>
    </div>
  );
}
