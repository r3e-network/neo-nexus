'use client';

import { useState } from 'react';
import { Blocks, Box, Code, Database, Eye, Search, ShieldCheck, Sparkles, Zap, ExternalLink } from 'lucide-react';

const categories = ['All', 'Developer Tools', 'Indexing & Data', 'Security & Oracles'];

const plugins = [
  {
    id: 'aa-bundler',
    name: 'Neo AA Bundler',
    provider: 'NeoNexus Official',
    category: 'Developer Tools',
    description: 'Enable native Account Abstraction capabilities. Exposes specialized RPC methods for fee delegation and batched user operations.',
    icon: Zap,
    color: 'from-yellow-400/20 to-yellow-600/20',
    iconColor: 'text-yellow-400',
    price: '$49/mo',
    type: 'Subscription',
    status: 'Available',
    featured: true,
  },
  {
    id: 'tee-oracle',
    name: 'Phala TEE Oracle',
    provider: 'Phala Network',
    category: 'Security & Oracles',
    description: 'Bind a Phala CVM oracle to your node. Fetch off-chain HTTPS data securely and push it on-chain without exposing API keys.',
    icon: ShieldCheck,
    color: 'from-[#00E599]/20 to-[#00cc88]/20',
    iconColor: 'text-[#00E599]',
    price: '$99/mo',
    type: 'Subscription',
    status: 'Installed',
    featured: true,
  },
  {
    id: 'token-tracker',
    name: 'NEP Token Indexer',
    provider: 'Neo Data Labs',
    category: 'Indexing & Data',
    description: 'A dedicated high-speed GraphQL indexer for NEP-11 (NFTs) and NEP-17 tokens. Instantly query holders and historical transfers.',
    icon: Blocks,
    color: 'from-blue-400/20 to-blue-600/20',
    iconColor: 'text-blue-400',
    price: 'Free',
    type: 'Open Source',
    status: 'Available',
    featured: false,
  },
  {
    id: 'mempool-watcher',
    name: 'Mempool Streamer',
    provider: 'NeoNexus Official',
    category: 'Developer Tools',
    description: 'Get real-time WebSocket streams of unconfirmed transactions before they are packed into a block. Perfect for MEV bots.',
    icon: Eye,
    color: 'from-purple-400/20 to-purple-600/20',
    iconColor: 'text-purple-400',
    price: '$19/mo',
    type: 'Subscription',
    status: 'Coming Soon',
    featured: false,
  },
  {
    id: 'sql-sync',
    name: 'PostgreSQL Sync',
    provider: 'ChainDB',
    category: 'Indexing & Data',
    description: 'Continuously syncs blockchain state directly into a managed PostgreSQL database for complex custom queries.',
    icon: Database,
    color: 'from-orange-400/20 to-orange-600/20',
    iconColor: 'text-orange-400',
    price: 'Usage Based',
    type: 'Pay-as-you-go',
    status: 'Available',
    featured: false,
  },
  {
    id: 'smart-contract-verifier',
    name: 'Contract Verifier',
    provider: 'Neo Security Team',
    category: 'Security & Oracles',
    description: 'Automatically verifies deployed NEF contracts against published source code on GitHub. Adds verification badges to RPC responses.',
    icon: Code,
    color: 'from-pink-400/20 to-pink-600/20',
    iconColor: 'text-pink-400',
    price: 'Free',
    type: 'Community',
    status: 'Available',
    featured: false,
  }
];

export default function Marketplace() {
  const [activeCategory, setActiveCategory] = useState('All');
  const [searchQuery, setSearchQuery] = useState('');

  const filteredPlugins = plugins.filter(p => {
    const matchesCategory = activeCategory === 'All' || p.category === activeCategory;
    const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
                          p.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  return (
    <div className="min-h-screen pb-12 space-y-8">
      {/* Hero Section */}
      <div className="relative bg-[#1A1A1A] border border-[#333333] rounded-2xl p-8 md:p-12 overflow-hidden">
        <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-[#00E599]/10 rounded-full blur-[100px] -translate-y-1/2 translate-x-1/3"></div>
        <div className="relative z-10 max-w-2xl">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#00E599]/10 border border-[#00E599]/20 text-[#00E599] text-sm font-medium mb-6">
            <Sparkles className="w-4 h-4" /> Nexus Add-ons
          </div>
          <h1 className="text-4xl font-bold tracking-tight text-white mb-4">Supercharge your N3 Infrastructure.</h1>
          <p className="text-lg text-gray-400 mb-8">
            Extend your dedicated nodes with enterprise-grade indexing, Account Abstraction relayers, and privacy oracles with a single click. No devops required.
          </p>
          <div className="relative w-full max-w-md">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
            <input 
              type="text" 
              placeholder="Search plugins, indexers, oracles..." 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-[#111111] border border-[#333333] rounded-xl pl-12 pr-4 py-3 text-white focus:outline-none focus:border-[#00E599] focus:ring-1 focus:ring-[#00E599] transition-all"
            />
          </div>
        </div>
      </div>

      {/* Category Filters */}
      <div className="flex items-center gap-2 overflow-x-auto pb-2 scrollbar-hide">
        {categories.map(category => (
          <button
            key={category}
            onClick={() => setActiveCategory(category)}
            className={`whitespace-nowrap px-5 py-2.5 rounded-full text-sm font-medium transition-all ${
              activeCategory === category 
                ? 'bg-[#00E599] text-black shadow-[0_0_15px_rgba(0,229,153,0.3)]' 
                : 'bg-[#1A1A1A] border border-[#333333] text-gray-400 hover:text-white hover:border-gray-500'
            }`}
          >
            {category}
          </button>
        ))}
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredPlugins.map((plugin) => (
          <div key={plugin.id} className="group flex flex-col bg-[#1A1A1A] border border-[#333333] hover:border-[#555555] rounded-2xl overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
            {/* Card Header */}
            <div className={`h-2 w-full bg-gradient-to-r ${plugin.color}`}></div>
            <div className="p-6 flex-1 flex flex-col">
              <div className="flex justify-between items-start mb-4">
                <div className={`p-3 rounded-xl bg-gradient-to-br ${plugin.color} border border-white/5`}>
                  <plugin.icon className={`w-8 h-8 ${plugin.iconColor}`} />
                </div>
                {plugin.featured && (
                  <span className="text-xs font-bold px-2 py-1 rounded bg-[#00E599]/10 text-[#00E599] border border-[#00E599]/20 uppercase tracking-wide">
                    Featured
                  </span>
                )}
              </div>
              
              <h3 className="text-xl font-bold text-white mb-1">{plugin.name}</h3>
              <p className="text-xs text-gray-500 font-medium mb-4 flex items-center gap-1">
                By <span className="text-gray-300">{plugin.provider}</span> • {plugin.category}
              </p>
              
              <p className="text-gray-400 text-sm mb-6 flex-1 leading-relaxed">
                {plugin.description}
              </p>
              
              {/* Card Footer */}
              <div className="pt-4 border-t border-[#333333] flex items-center justify-between mt-auto">
                <div>
                  <div className="text-lg font-bold text-white">{plugin.price}</div>
                  <div className="text-xs text-gray-500">{plugin.type}</div>
                </div>
                
                <button 
                  className={`px-4 py-2 rounded-lg font-bold text-sm transition-all flex items-center gap-2 ${
                    plugin.status === 'Installed' 
                      ? 'bg-[#333333] text-[#00E599] cursor-default'
                      : plugin.status === 'Coming Soon'
                      ? 'bg-transparent border border-[#333333] text-gray-500 cursor-not-allowed'
                      : 'bg-[#111111] border border-[#333333] text-white hover:bg-[#00E599] hover:text-black hover:border-[#00E599]'
                  }`}
                  disabled={plugin.status !== 'Available'}
                >
                  {plugin.status === 'Installed' ? 'Installed' : plugin.status === 'Coming Soon' ? 'Coming Soon' : 'Add to Node'}
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
      
      {filteredPlugins.length === 0 && (
        <div className="py-20 text-center">
          <Box className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white">No plugins found</h3>
          <p className="text-gray-400 mt-1">Try adjusting your search query or category filter.</p>
        </div>
      )}
    </div>
  );
}
