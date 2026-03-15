import { Search, ChevronRight, FileText } from 'lucide-react';

export default function Docs() {
  return (
    <div className="min-h-screen flex bg-[var(--color-dark-bg)]">
      {/* Sidebar */}
      <aside className="w-64 border-r border-[var(--color-dark-border)] hidden md:flex flex-col h-[calc(100vh-80px)] sticky top-20 overflow-y-auto">
        <div className="p-6">
          <div className="relative mb-6">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input 
              type="text" 
              placeholder="Search docs..." 
              className="w-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg pl-9 pr-3 py-2 text-sm text-white focus:outline-none focus:border-[#00E599]"
            />
          </div>
          
          <nav className="space-y-6">
            <div>
              <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">Getting Started</h4>
              <ul className="space-y-2">
                <li><a href="#" className="text-sm text-[#00E599] font-medium">Introduction</a></li>
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">Create an Account</a></li>
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">First API Request</a></li>
              </ul>
            </div>
            
            <div>
              <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">Nodes</h4>
              <ul className="space-y-2">
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">Shared Endpoints</a></li>
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">Dedicated Nodes</a></li>
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">WebSockets (WSS)</a></li>
              </ul>
            </div>

            <div>
              <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">Marketplace</h4>
              <ul className="space-y-2">
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">TEE Oracle</a></li>
                <li><a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">AA Bundler</a></li>
              </ul>
            </div>
          </nav>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 md:p-16 max-w-4xl">
        <div className="text-sm text-[#00E599] font-medium mb-4 flex items-center gap-2">
          Docs <ChevronRight className="w-4 h-4 text-gray-600" /> Getting Started <ChevronRight className="w-4 h-4 text-gray-600" /> Introduction
        </div>
        
        <h1 className="text-4xl font-bold text-white mb-6">Introduction to NeoNexus</h1>
        
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
            <li><strong className="text-white">Dedicated Performance:</strong> For production DApps, deploy a dedicated node in Tokyo, Frankfurt, or Virginia.</li>
            <li><strong className="text-white">Enterprise Security:</strong> IP allowlists and RPC method firewalls out of the box.</li>
          </ul>

          <h2 className="text-2xl font-bold text-white mt-10 mb-4 border-b border-[var(--color-dark-border)] pb-2">Quick Start</h2>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg p-4 mb-6">
            <pre className="text-sm text-gray-300 font-mono">
              <code>{`curl -X POST https://mainnet.neonexus.cloud/v1/your-api-key \\
  -H "Content-Type: application/json" \\
  -d '{ "jsonrpc": "2.0", "id": 1, "method": "getblockcount", "params": [] }'`}</code>
            </pre>
          </div>

          <div className="flex justify-between items-center mt-16 pt-8 border-t border-[var(--color-dark-border)]">
            <div></div>
            <a href="#" className="flex flex-col items-end group">
              <span className="text-xs text-gray-500 mb-1">Next</span>
              <span className="text-[#00E599] font-medium flex items-center gap-1 group-hover:gap-2 transition-all">Create an Account <ChevronRight className="w-4 h-4" /></span>
            </a>
          </div>
        </div>
      </main>
    </div>
  );
}
