import Link from 'next/link';
import { Check, Server, Zap } from 'lucide-react';

export default function Pricing() {
  return (
    <div className="min-h-screen pb-24">
      {/* Header */}
      <div className="pt-32 pb-16 text-center px-6">
        <h1 className="text-4xl md:text-6xl font-bold tracking-tight text-white mb-6">Simple, transparent pricing</h1>
        <p className="text-xl text-gray-400 max-w-2xl mx-auto">
          Start for free, then pay only for what you use. Dedicated nodes available for enterprise scale.
        </p>
      </div>

      <div className="max-w-7xl mx-auto px-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-5xl mx-auto">
          
          {/* Developer Tier */}
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-3xl p-8 flex flex-col">
            <h3 className="text-xl font-bold text-white mb-2">Developer</h3>
            <p className="text-sm text-gray-400 mb-6">Perfect for learning, prototyping, and hackathons.</p>
            <div className="mb-6">
              <span className="text-4xl font-bold text-white">$0</span>
              <span className="text-gray-500"> / month</span>
            </div>
            <Link href="/login?signup=true" className="w-full bg-[#222222] hover:bg-[#333333] text-white text-center py-3 rounded-xl font-bold transition-colors mb-8">
              Start for Free
            </Link>
            <div className="space-y-4 flex-1">
              <div className="text-sm font-medium text-white mb-4 uppercase tracking-wider">Includes:</div>
              <Feature text="1 Shared API Endpoint" />
              <Feature text="5,000,000 requests / month" />
              <Feature text="N3 & Neo X Networks" />
              <Feature text="Community Discord Support" />
              <Feature text="Basic Analytics" />
            </div>
          </div>

          {/* Growth Tier */}
          <div className="bg-[var(--color-dark-panel)] border-2 border-[#00E599] rounded-3xl p-8 flex flex-col relative transform md:-translate-y-4 shadow-[0_0_40px_rgba(0,229,153,0.1)]">
            <div className="absolute top-0 left-1/2 -translate-x-1/2 bg-[#00E599] text-black text-xs font-bold px-4 py-1 rounded-b-lg">MOST POPULAR</div>
            <h3 className="text-xl font-bold text-white mb-2 mt-2">Growth</h3>
            <p className="text-sm text-gray-400 mb-6">For production DApps requiring higher limits and speed.</p>
            <div className="mb-6">
              <span className="text-4xl font-bold text-white">$49</span>
              <span className="text-gray-500"> / month</span>
            </div>
            <Link href="/login?signup=true" className="w-full bg-[#00E599] hover:bg-[#00cc88] text-black text-center py-3 rounded-xl font-bold transition-colors mb-8 shadow-lg">
              Get Started
            </Link>
            <div className="space-y-4 flex-1">
              <div className="text-sm font-medium text-white mb-4 uppercase tracking-wider">Everything in Developer, plus:</div>
              <Feature text="3 Shared API Endpoints" />
              <Feature text="20,000,000 requests / month" />
              <Feature text="Auto-scaling Global Network" />
              <Feature text="Email Support (24h SLA)" />
              <Feature text="Advanced Analytics, Allowlists & Method Firewall" />
              <Feature text="Access to Managed Plugin Catalog" />
            </div>
          </div>

          {/* Dedicated Tier */}
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-3xl p-8 flex flex-col">
            <h3 className="text-xl font-bold text-white mb-2">Dedicated</h3>
            <p className="text-sm text-gray-400 mb-6">Managed dedicated Neo N3 / Neo X VMs with isolated capacity and provider failover.</p>
            <div className="mb-6">
              <span className="text-4xl font-bold text-white">$99</span>
              <span className="text-gray-500"> / month</span>
              <span className="text-xs text-gray-500 block mt-1">Starting price per node</span>
            </div>
            <Link href="/login?signup=true" className="w-full bg-[#222222] hover:bg-[#333333] text-white text-center py-3 rounded-xl font-bold transition-colors mb-8">
              Deploy Dedicated Node
            </Link>
            <div className="space-y-4 flex-1">
              <div className="text-sm font-medium text-white mb-4 uppercase tracking-wider">Features:</div>
              <Feature text="Managed dedicated VM deployment" />
              <Feature text="Unlimited requests" />
              <Feature text="Client engines: neo-go, neo-cli, neo-x-geth" />
              <Feature text="Hetzner primary regions with DigitalOcean fallback" />
              <Feature text="WSS (WebSocket) Support" />
              <Feature text="Custom Node Configuration" />
              <Feature text="Priority 24/7 Support" />
            </div>
          </div>

        </div>

        {/* Add-ons Section */}
        <div className="mt-24 pt-16 border-t border-[var(--color-dark-border)] max-w-5xl mx-auto">
          <h2 className="text-2xl md:text-3xl font-bold text-white mb-8 text-center">Managed Plugin Catalog</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl flex justify-between items-center">
              <div>
                <h4 className="font-bold text-white flex items-center gap-2"><Zap className="w-4 h-4 text-yellow-400"/> Account Abstraction Bundler</h4>
                <p className="text-sm text-gray-400 mt-1">Managed per dedicated node through the plugin catalog.</p>
              </div>
              <div className="font-bold text-white">$49<span className="text-sm text-gray-500 font-normal">/mo</span></div>
            </div>
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-6 rounded-2xl flex justify-between items-center">
              <div>
                <h4 className="font-bold text-white flex items-center gap-2"><Server className="w-4 h-4 text-[#00E599]"/> Archive Node Sync</h4>
                <p className="text-sm text-gray-400 mt-1">Expanded storage and history retention for dedicated nodes.</p>
              </div>
              <div className="font-bold text-white">+$50<span className="text-sm text-gray-500 font-normal">/mo</span></div>
            </div>
          </div>
        </div>

      </div>
    </div>
  );
}

function Feature({ text }: { text: string }) {
  return (
    <div className="flex items-start gap-3">
      <Check className="w-5 h-5 text-[#00E599] shrink-0 mt-0.5" />
      <span className="text-gray-300 text-sm">{text}</span>
    </div>
  );
}
