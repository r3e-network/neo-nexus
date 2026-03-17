'use client';

import { useState } from 'react';
import Link from 'next/link';
import { Search, ChevronRight } from 'lucide-react';
import { docsNavigation, filterDocsNavigation } from '@/services/docs/DocsIndex';

export default function Docs() {
  const [query, setQuery] = useState('');
  const filteredNavigation = filterDocsNavigation(docsNavigation, query);

  return (
    <div className="min-h-screen flex bg-[var(--color-dark-bg)]">
      {/* Sidebar */}
      <aside className="w-64 border-r border-[var(--color-dark-border)] hidden md:flex flex-col h-[calc(100vh-80px)] sticky top-20 overflow-y-auto">
        <div className="p-6">
          <div className="relative mb-6">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input 
              type="text" 
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search docs..." 
              className="w-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg pl-9 pr-3 py-2 text-sm text-white focus:outline-none focus:border-[#00E599]"
            />
          </div>
          
          <nav className="space-y-6">
            {filteredNavigation.map((section) => (
              <div key={section.title}>
                <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">{section.title}</h4>
                <ul className="space-y-2">
                  {section.items.map((item, index) => (
                    <li key={item.anchor}>
                      <a
                        href={item.anchor}
                        className={`text-sm transition-colors ${query.trim() === '' && section.title === 'Getting Started' && index === 0 ? 'text-[#00E599] font-medium' : 'text-gray-400 hover:text-white'}`}
                      >
                        {item.label}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
            {filteredNavigation.length === 0 && (
              <div className="text-sm text-gray-500">No matching docs sections.</div>
            )}
          </nav>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 md:p-16 max-w-4xl">
        <div className="text-sm text-[#00E599] font-medium mb-4 flex items-center gap-2">
          Docs <ChevronRight className="w-4 h-4 text-gray-600" /> Getting Started <ChevronRight className="w-4 h-4 text-gray-600" /> Introduction
        </div>
        
        <h1 id="introduction" className="text-4xl font-bold text-white mb-6 scroll-mt-24">Introduction to NeoNexus</h1>
        
        <div className="prose prose-invert prose-neo max-w-none">
          <p className="text-lg text-gray-300 leading-relaxed mb-8">
            NeoNexus is the industrial-grade Web3 infrastructure platform specifically built for the Neo ecosystem. We provide highly available RPC endpoints, dedicated node hosting, and advanced indexing tools for both <strong>Neo N3</strong> and <strong>Neo X</strong> (EVM compatible).
          </p>
          
          <h2 className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2">Why NeoNexus?</h2>
          <p className="text-gray-400 mb-4">
            Building on Neo requires a stable connection to the blockchain. Running your own node (neo-go, neo-cli, or neo-x-geth) requires significant devops overhead, server costs, and constant maintenance to ensure synchronization. NeoNexus solves this by providing:
          </p>
          <ul className="list-disc list-inside text-gray-400 space-y-2 mb-8 ml-4">
            <li><strong className="text-white">Instant Setup:</strong> Get an API key in seconds and start building.</li>
            <li><strong className="text-white">Fast Snapshot Sync:</strong> Launch dedicated nodes that sync in minutes using official snapshots.</li>
            <li><strong className="text-white">Multi-Protocol Support:</strong> Native support for both the powerful Neo N3 engine and the EVM-compatible Neo X network.</li>
            <li><strong className="text-white">Dedicated Performance:</strong> For production DApps, provision isolated managed nodes with Hetzner as the primary provider and DigitalOcean as fallback.</li>
            <li><strong className="text-white">Enterprise Security:</strong> IP allowlists, routed traffic controls, and JSON-RPC method firewall presets out of the box.</li>
          </ul>

          <h2 id="create-account" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">Create an Account</h2>
          <p className="text-gray-400 mb-6">
            Start from the sign-up flow, create an organization, and choose whether you want a free shared endpoint or a managed dedicated node footprint.
          </p>
          <div className="mb-8">
            <Link href="/login?signup=true" className="inline-flex items-center gap-2 bg-[#00E599] hover:bg-[#00cc88] text-black px-5 py-3 rounded-lg font-bold transition-colors not-prose">
              Create Free Account <ChevronRight className="w-4 h-4" />
            </Link>
          </div>

          <h2 id="quick-start" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">First API Request</h2>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg p-4 mb-6">
            <pre className="text-sm text-gray-300 font-mono">
              <code>{`curl -X POST https://mainnet.neonexus.cloud/v1/your-api-key \\
  -H "Content-Type: application/json" \\
  -d '{ "jsonrpc": "2.0", "id": 1, "method": "getblockcount", "params": [] }'`}</code>
            </pre>
          </div>

          <h2 id="shared-endpoints" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">Shared Endpoints</h2>
          <p className="text-gray-400 mb-8">
            Shared endpoints are routed through APISIX to managed upstream node pools. They are ideal for prototyping, smaller workloads, and teams that need fast onboarding without isolated capacity.
          </p>

          <h2 id="dedicated-nodes" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">Dedicated Nodes</h2>
          <p className="text-gray-400 mb-8">
            Dedicated nodes are provisioned as managed VMs. You can choose the node engine, sync mode, region, and supported plugins while the control plane handles provisioning, routing, and lifecycle operations.
          </p>

          <h2 id="websockets" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">WebSockets (WSS)</h2>
          <p className="text-gray-400 mb-8">
            Neo X endpoints expose optional WebSocket routing, and WSS URLs are shown in the endpoint details view once the route is configured. Neo N3 shared and dedicated endpoints remain primarily RPC-oriented unless the current runtime supports a WebSocket surface.
          </p>

          <h2 id="tee-oracle" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">TEE Oracle</h2>
          <p className="text-gray-400 mb-8">
            The TEE Oracle plugin is managed per dedicated endpoint. Install it from the endpoint&apos;s Plugins tab, provide the required signing key, and the platform will sync the runtime configuration to the managed node.
          </p>

          <h2 id="aa-bundler" className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2 scroll-mt-24">AA Bundler</h2>
          <p className="text-gray-400 mb-8">
            The Account Abstraction Bundler plugin follows the same dedicated-endpoint flow: browse the plugin catalog, choose a node, and configure it from the endpoint plugin management surface.
          </p>

          <div className="flex justify-between items-center mt-16 pt-8 border-t border-[var(--color-dark-border)]">
            <div></div>
            <Link href="/login?signup=true" className="flex flex-col items-end group">
              <span className="text-xs text-gray-500 mb-1">Next</span>
              <span className="text-[#00E599] font-medium flex items-center gap-1 group-hover:gap-2 transition-all">Create an Account <ChevronRight className="w-4 h-4" /></span>
            </Link>
          </div>
        </div>
      </main>
    </div>
  );
}
