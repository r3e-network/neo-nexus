'use client';

import { Copy, Plus, Server, Settings, SignalHigh } from 'lucide-react';
import Link from 'next/link';

export type Endpoint = {
  id: string | number;
  name: string;
  network: string;
  type: string;
  url: string;
  status: string;
  requests: string | number;
  clientEngine?: string;
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
        {endpoints.map((ep) => (
          <div key={ep.id} className="bg-[#23252E] border border-[#2E313D] rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between">
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
              <div className="bg-[#1A1C23] border border-[#2E313D] px-3 py-2 rounded-md flex-1 md:w-64 flex items-center justify-between group">
                <code className="text-sm text-gray-300 truncate w-40">{ep.url}</code>
                <button 
                  onClick={() => navigator.clipboard.writeText(ep.url)}
                  className="text-gray-500 hover:text-white transition-colors"
                  title="Copy URL"
                >
                  <Copy className="w-4 h-4" />
                </button>
              </div>
              <Link href={`/endpoints/${ep.id}`} className="bg-[#333333] hover:bg-[#444444] text-white p-2 rounded-md transition-colors">
                <Settings className="w-4 h-4" />
              </Link>
            </div>
          </div>
        ))}

        {endpoints.length === 0 && (
          <div className="text-center py-12 bg-[#23252E] border border-[#2E313D] rounded-xl">
            <Server className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No endpoints found</h3>
            <p className="text-gray-400 mb-6">You haven't deployed any nodes yet.</p>
            <Link href="/app/endpoints/new" className="inline-flex bg-[#00E599] hover:bg-[#00cc88] text-black px-6 py-2 rounded-md text-sm font-bold transition-colors items-center gap-2">
              <Plus className="w-4 h-4" /> Deploy your first node
            </Link>
          </div>
        )}
      </div>
    </div>
  );
}
