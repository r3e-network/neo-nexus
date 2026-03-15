import Link from 'next/link';
import { ArrowRight, Blocks, Cloud, Code, Globe, Lock, Server, ShieldCheck, Zap, Database } from 'lucide-react';

export default function Home() {
  return (
    <div className="flex flex-col min-h-screen">
      
      {/* Hero Section */}
      <section className="relative pt-32 pb-20 md:pt-48 md:pb-32 overflow-hidden">
        {/* Background Gradients */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] bg-[#00E599]/20 rounded-full blur-[120px] -z-10"></div>
        <div className="absolute top-40 right-0 w-[500px] h-[500px] bg-blue-500/10 rounded-full blur-[120px] -z-10"></div>

        <div className="max-w-7xl mx-auto px-6 text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] text-sm text-gray-300 mb-8 hover:border-[#00E599] transition-colors cursor-pointer">
            <span className="w-2 h-2 rounded-full bg-[#00E599] animate-pulse"></span>
            Neo N3 and Neo X are fully supported
            <ArrowRight className="w-3 h-3" />
          </div>
          
          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight text-white mb-8 max-w-4xl mx-auto leading-tight">
            The Ultimate Web3 Cloud for <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#00E599] to-blue-400">Neo N3</span> & <span className="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-400">Neo X</span>
          </h1>
          
          <p className="text-lg md:text-xl text-gray-400 mb-10 max-w-2xl mx-auto leading-relaxed">
            Fast, scalable, and reliable RPC endpoints, dedicated nodes, and enterprise-grade APIs. Build your next DApp on Neo with zero devops.
          </p>
          
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link href="/login?signup=true" className="w-full sm:w-auto bg-[#00E599] hover:bg-[#00cc88] text-black px-8 py-4 rounded-xl font-bold transition-all hover:shadow-[0_0_20px_rgba(0,229,153,0.4)] text-lg">
              Deploy a Node for Free
            </Link>
            <Link href="/docs" className="w-full sm:w-auto bg-[var(--color-dark-panel)] hover:bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] text-white px-8 py-4 rounded-xl font-bold transition-colors text-lg flex items-center justify-center gap-2">
              Read Docs
            </Link>
          </div>
        </div>
      </section>

      {/* Trusted By / Stats Section */}
      <section className="py-10 border-y border-white/5 bg-[#050505]">
        <div className="max-w-7xl mx-auto px-6">
          <p className="text-center text-sm font-bold text-gray-500 uppercase tracking-widest mb-8">Trusted by the Neo Ecosystem</p>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center divide-x divide-white/5">
            <div>
              <div className="text-4xl font-bold text-white mb-2">99.99%</div>
              <div className="text-sm text-gray-400">Uptime SLA</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-white mb-2">50ms</div>
              <div className="text-sm text-gray-400">Global Latency</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-white mb-2">1B+</div>
              <div className="text-sm text-gray-400">Requests Served</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-white mb-2">3</div>
              <div className="text-sm text-gray-400">Global Regions</div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="products" className="py-24 md:py-32">
        <div className="max-w-7xl mx-auto px-6">
          <div className="text-center max-w-3xl mx-auto mb-20">
            <h2 className="text-3xl md:text-5xl font-bold text-white mb-6">Everything you need to build on Neo</h2>
            <p className="text-gray-400 text-lg">From free shared endpoints for rapid prototyping to custom K8s-deployed dedicated nodes for enterprise scale.</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {/* Feature 1 */}
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#00E599]/50 rounded-2xl p-8 transition-all group">
              <div className="w-14 h-14 rounded-xl bg-[#00E599]/10 text-[#00E599] flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <Globe className="w-7 h-7" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">Global Shared APIs</h3>
              <p className="text-gray-400 leading-relaxed mb-6">
                Connect instantly to our globally load-balanced pool of Neo N3 and Neo X nodes. Perfect for quick queries and small apps.
              </p>
              <Link href="/pricing" className="text-[#00E599] font-medium flex items-center gap-1 hover:gap-2 transition-all">
                Get free API key <ArrowRight className="w-4 h-4" />
              </Link>
            </div>

            {/* Feature 2 */}
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-blue-500/50 rounded-2xl p-8 transition-all group relative overflow-hidden">
              <div className="absolute top-0 right-0 bg-blue-500 text-black text-xs font-bold px-3 py-1 rounded-bl-lg">POPULAR</div>
              <div className="w-14 h-14 rounded-xl bg-blue-500/10 text-blue-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <Server className="w-7 h-7" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">Dedicated Nodes</h3>
              <p className="text-gray-400 leading-relaxed mb-6">
                Your private Neo instance on AWS or GCP. Zero rate limits, max performance, and complete control over client engine (neo-go/neo-cli).
              </p>
              <Link href="/pricing" className="text-blue-400 font-medium flex items-center gap-1 hover:gap-2 transition-all">
                View dedicated plans <ArrowRight className="w-4 h-4" />
              </Link>
            </div>

            {/* Feature 3 */}
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-purple-500/50 rounded-2xl p-8 transition-all group">
              <div className="w-14 h-14 rounded-xl bg-purple-500/10 text-purple-400 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <Blocks className="w-7 h-7" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">Marketplace Add-ons</h3>
              <p className="text-gray-400 leading-relaxed mb-6">
                One-click install advanced capabilities: Phala TEE Oracles, Native Account Abstraction Bundlers, and high-speed Token Indexers.
              </p>
              <Link href="/developers" className="text-purple-400 font-medium flex items-center gap-1 hover:gap-2 transition-all">
                Explore add-ons <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Developer Experience Section */}
      <section className="py-24 md:py-32 bg-[#050505] border-t border-white/5 relative overflow-hidden">
        <div className="absolute top-1/2 right-0 w-[600px] h-[600px] bg-[#00E599]/5 rounded-full blur-[100px] -translate-y-1/2"></div>
        
        <div className="max-w-7xl mx-auto px-6 grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
          <div>
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] text-sm text-[#00E599] font-medium mb-6">
              <Code className="w-4 h-4" /> Developer First
            </div>
            <h2 className="text-3xl md:text-4xl font-bold text-white mb-6">Powerful dashboard. Deep insights.</h2>
            <p className="text-gray-400 text-lg leading-relaxed mb-8">
              Monitor your DApp's health in real-time. Our console provides granular metrics on RPC calls, latency, bandwidth, and errors. Configure advanced firewalls to protect your endpoints from abuse.
            </p>
            
            <ul className="space-y-4 mb-8">
              {[
                'Real-time metrics and historical analytics',
                'Advanced IP and Method firewall',
                'Direct WSS (WebSocket) support',
                'Seamless integration with Neo dAPI'
              ].map((item, i) => (
                <li key={i} className="flex items-center gap-3 text-gray-300">
                  <ShieldCheck className="w-5 h-5 text-[#00E599]" />
                  {item}
                </li>
              ))}
            </ul>
            
            <Link href="/login?signup=true" className="text-white font-bold underline decoration-[#00E599] decoration-2 underline-offset-4 hover:text-[#00E599] transition-colors">
              Go to Console →
            </Link>
          </div>
          
          <div className="relative">
            <div className="absolute inset-0 bg-gradient-to-tr from-[#00E599] to-blue-500 rounded-2xl blur-2xl opacity-20"></div>
            <div className="relative bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-6 shadow-2xl">
              <div className="flex gap-2 mb-4">
                <div className="w-3 h-3 rounded-full bg-red-500"></div>
                <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                <div className="w-3 h-3 rounded-full bg-green-500"></div>
              </div>
              <pre className="text-sm text-gray-300 font-mono overflow-x-auto">
                <code className="language-javascript">
{`// Initialize connection to NeoNexus
import { rpc } from '@cityofzion/neon-js';

const node = new rpc.RPCClient(
  'https://mainnet.neonexus.cloud/v1/your-api-key'
);

// Query Neo N3 Blockchain
async function getBlockCount() {
  const height = await node.getBlockCount();
  console.log(\`Current block: \${height}\`);
  
  // High-speed indexer API
  const balance = await node.query({
    method: 'getnep17balances',
    params: ['NfgH5...]
  });
}`}
                </code>
              </pre>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-32 relative">
        <div className="max-w-4xl mx-auto px-6 text-center">
          <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">Ready to scale on Neo?</h2>
          <p className="text-xl text-gray-400 mb-10">Join the best teams building the future of the Neo ecosystem. Deploy your infrastructure in seconds.</p>
          <Link href="/login?signup=true" className="bg-white text-black hover:bg-gray-200 px-10 py-5 rounded-full font-bold text-lg transition-colors shadow-xl">
            Create Free Account
          </Link>
        </div>
      </section>
      
    </div>
  );
}
