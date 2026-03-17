'use client';

import { useMemo, useState } from 'react';
import Link from 'next/link';
import { Box, Database, Search, ShieldCheck, Sparkles, Zap } from 'lucide-react';
import type { MarketplacePluginCard } from '@/services/plugins/MarketplaceCatalog';

type MarketplaceClientProps = {
  cards: MarketplacePluginCard[];
  dedicatedEndpointCount: number;
};

function getCardVisual(category: string) {
  if (category === 'Security') {
    return {
      icon: ShieldCheck,
      color: 'from-[#00E599]/20 to-[#00cc88]/20',
      iconColor: 'text-[#00E599]',
    };
  }

  return {
    icon: Zap,
    color: 'from-yellow-400/20 to-yellow-600/20',
    iconColor: 'text-yellow-400',
  };
}

export default function MarketplaceClient({ cards, dedicatedEndpointCount }: MarketplaceClientProps) {
  const [activeCategory, setActiveCategory] = useState('All');
  const [searchQuery, setSearchQuery] = useState('');

  const categories = useMemo(
    () => ['All', ...Array.from(new Set(cards.map((card) => card.category)))],
    [cards],
  );

  const filteredCards = cards.filter((card) => {
    const matchesCategory = activeCategory === 'All' || card.category === activeCategory;
    const matchesSearch = card.name.toLowerCase().includes(searchQuery.toLowerCase())
      || card.description.toLowerCase().includes(searchQuery.toLowerCase())
      || card.badge.toLowerCase().includes(searchQuery.toLowerCase());

    return matchesCategory && matchesSearch;
  });

  return (
    <div className="min-h-screen pb-12 space-y-8">
      <div className="relative bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 md:p-12 overflow-hidden">
        <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-[#00E599]/10 rounded-full blur-[100px] -translate-y-1/2 translate-x-1/3"></div>
        <div className="relative z-10 max-w-3xl">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#00E599]/10 border border-[#00E599]/20 text-[#00E599] text-sm font-medium mb-6">
            <Sparkles className="w-4 h-4" /> Managed Plugin Catalog
          </div>
          <h1 className="text-4xl font-bold tracking-tight text-white mb-4">Supported plugins for your dedicated nodes.</h1>
          <p className="text-lg text-gray-400 mb-4">
            This catalog reflects the plugins currently supported by the managed node runtime. Installation happens from a dedicated endpoint, not from this page.
          </p>
          <p className="text-sm text-gray-500 mb-8">
            {dedicatedEndpointCount > 0
              ? `${dedicatedEndpointCount} dedicated ${dedicatedEndpointCount === 1 ? 'node is' : 'nodes are'} available for plugin configuration.`
              : 'You need at least one dedicated node before any plugin can be configured.'}
          </p>
          <div className="relative w-full max-w-md">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
            <input
              type="text"
              placeholder="Search supported plugins..."
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              className="w-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl pl-12 pr-4 py-3 text-white focus:outline-none focus:border-[#00E599] focus:ring-1 focus:ring-[#00E599] transition-all"
            />
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2 overflow-x-auto pb-2 scrollbar-hide">
        {categories.map((category) => (
          <button
            key={category}
            onClick={() => setActiveCategory(category)}
            className={`whitespace-nowrap px-5 py-2.5 rounded-full text-sm font-medium transition-all ${
              activeCategory === category
                ? 'bg-[#00E599] text-black shadow-[0_0_15px_rgba(0,229,153,0.3)]'
                : 'bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] text-gray-400 hover:text-white hover:border-gray-500'
            }`}
          >
            {category}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredCards.map((card) => {
          const visual = getCardVisual(card.category);
          const Icon = visual.icon;

          return (
            <div key={card.id} className="group flex flex-col bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:border-[#555555] rounded-2xl overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
              <div className={`h-2 w-full bg-gradient-to-r ${visual.color}`}></div>
              <div className="p-6 flex-1 flex flex-col">
                <div className="flex justify-between items-start mb-4">
                  <div className={`p-3 rounded-xl bg-gradient-to-br ${visual.color} border border-white/5`}>
                    <Icon className={`w-8 h-8 ${visual.iconColor}`} />
                  </div>
                  <span className="text-xs font-bold px-2 py-1 rounded bg-[#00E599]/10 text-[#00E599] border border-[#00E599]/20 uppercase tracking-wide">
                    {card.badge}
                  </span>
                </div>

                <h3 className="text-xl font-bold text-white mb-1">{card.name}</h3>
                <p className="text-xs text-gray-500 font-medium mb-4">
                  {card.category} • {card.requiresPrivateKey ? 'Private key required' : 'No signing key required'}
                </p>

                <p className="text-gray-400 text-sm mb-4 flex-1 leading-relaxed">
                  {card.description}
                </p>

                <div className="space-y-2 mb-6">
                  <div className="text-sm font-medium text-white">{card.statusLabel}</div>
                  <div className="text-xs text-gray-500 font-mono break-all">{card.defaultImage}</div>
                </div>

                <div className="pt-4 border-t border-[var(--color-dark-border)] flex items-center justify-between mt-auto gap-4">
                  <div className="text-xs text-gray-500">
                    {card.installedOnCount > 0 ? 'Installed through endpoint plugin management.' : 'Install from a dedicated endpoint.'}
                  </div>
                  <Link
                    href={card.actionHref}
                    className="px-4 py-2 rounded-lg font-bold text-sm transition-all flex items-center gap-2 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] text-white hover:bg-[#00E599] hover:text-black hover:border-[#00E599]"
                  >
                    {card.actionLabel}
                  </Link>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {filteredCards.length === 0 && (
        <div className="py-20 text-center">
          <Box className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white">No plugins found</h3>
          <p className="text-gray-400 mt-1">Try adjusting your search query or category filter.</p>
        </div>
      )}

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-6">
        <div className="flex items-start gap-4">
          <div className="p-3 rounded-xl bg-blue-500/10 border border-blue-500/20">
            <Database className="w-6 h-6 text-blue-300" />
          </div>
          <div className="space-y-2">
            <h2 className="text-lg font-bold text-white">How installation works</h2>
            <p className="text-sm text-gray-400">
              Plugin configuration, key injection, remote sync, runtime status, and logs are managed from each dedicated endpoint&apos;s Plugins tab. This page is an inventory and entry point, not a one-click installer.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
