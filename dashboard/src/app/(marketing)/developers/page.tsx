import { BookOpen, Code2, Terminal, Wrench } from 'lucide-react';

export default function Developers() {
  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <div className="bg-[var(--color-dark-panel)] border-b border-[var(--color-dark-border)]">
        <div className="max-w-7xl mx-auto px-6 pt-32 pb-16">
          <h1 className="text-4xl md:text-6xl font-bold tracking-tight text-white mb-6">Developer Hub</h1>
          <p className="text-xl text-gray-400 max-w-2xl">
            Everything you need to build, deploy, and scale your Neo applications.
          </p>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-6 py-16">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          
          <a href="/docs" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#00E599]/50 p-8 rounded-3xl transition-all group">
            <div className="w-12 h-12 rounded-xl bg-blue-500/10 text-blue-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
              <BookOpen className="w-6 h-6" />
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">Documentation</h3>
            <p className="text-gray-400 mb-6">Read our comprehensive guides on how to connect to nodes, configure firewalls, and install marketplace plugins.</p>
            <span className="text-[#00E599] font-medium">Read Docs →</span>
          </a>

          <a href="#" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#00E599]/50 p-8 rounded-3xl transition-all group">
            <div className="w-12 h-12 rounded-xl bg-purple-500/10 text-purple-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
              <Code2 className="w-6 h-6" />
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">API Reference</h3>
            <p className="text-gray-400 mb-6">Explore the full Neo N3 and Neo X RPC specifications. Test requests directly from the browser.</p>
            <span className="text-[#00E599] font-medium">View API →</span>
          </a>

          <a href="#" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#00E599]/50 p-8 rounded-3xl transition-all group">
            <div className="w-12 h-12 rounded-xl bg-yellow-500/10 text-yellow-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
              <Wrench className="w-6 h-6" />
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">SDKs & Tools</h3>
            <p className="text-gray-400 mb-6">Find guides for integrating with neon-js, boa (Python), and neo-mamba.</p>
            <span className="text-[#00E599] font-medium">Explore Tools →</span>
          </a>

          <a href="#" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#00E599]/50 p-8 rounded-3xl transition-all group">
            <div className="w-12 h-12 rounded-xl bg-green-500/10 text-green-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
              <Terminal className="w-6 h-6" />
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">Tutorials</h3>
            <p className="text-gray-400 mb-6">Step-by-step guides on deploying smart contracts, setting up WebSockets, and reading historical state.</p>
            <span className="text-[#00E599] font-medium">Start Learning →</span>
          </a>

        </div>

        <div className="mt-16 bg-gradient-to-r from-[#111111] to-[#1A1A1A] border border-[var(--color-dark-border)] rounded-3xl p-10 flex flex-col md:flex-row items-center justify-between gap-8">
          <div>
            <h3 className="text-2xl font-bold text-white mb-2">Need help?</h3>
            <p className="text-gray-400">Join our Discord community or open a ticket. We are here to help you build.</p>
          </div>
          <button className="bg-white text-black px-6 py-3 rounded-lg font-bold hover:bg-gray-200 transition-colors whitespace-nowrap">
            Join Discord
          </button>
        </div>
      </div>
    </div>
  );
}
