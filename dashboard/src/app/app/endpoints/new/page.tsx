'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { ArrowLeft, CheckCircle2, ChevronRight, Cloud, HardDrive, Info, Layers, Server, Zap } from 'lucide-react';
import Link from 'next/link';
import { createEndpointAction } from '../actions';
import toast from 'react-hot-toast';

const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback;

export default function CreateEndpoint() {
  const router = useRouter();
  const [isDeploying, setIsDeploying] = useState(false);

  // Form State
  const [name, setName] = useState('My Node');
  const [protocol, setProtocol] = useState<'neo-n3' | 'neo-x'>('neo-n3');
  const [network, setNetwork] = useState('mainnet');
  const [clientEngine, setClientEngine] = useState('neo-go');
  const [nodeType, setNodeType] = useState('dedicated');
  const [provider, setProvider] = useState('digitalocean');
  const [region, setRegion] = useState('nyc3');
  const [syncMode, setSyncMode] = useState('full');

  // Ensure valid client engine when protocol changes
  const handleProtocolChange = (newProtocol: 'neo-n3' | 'neo-x') => {
    setProtocol(newProtocol);
    if (newProtocol === 'neo-x') {
      setClientEngine('neo-x-geth');
    } else {
      setClientEngine('neo-go');
    }
  };

  const pricing = {
    shared: { full: 0, archive: 49 },
    dedicated: { full: 99, archive: 149 }
  };

  const calculatePrice = () => {
    return pricing[nodeType as keyof typeof pricing][syncMode as keyof typeof pricing.shared];
  };

  const handleDeploy = async () => {
    setIsDeploying(true);
    
    try {
      const result = await createEndpointAction({
        name,
        protocol,
        network,
        type: nodeType,
        clientEngine,
        provider,
        region,
        syncMode
      });

      if (result.success) {
        toast.success('Node deployed successfully!');
        router.push(`/app/endpoints/${result.id}`);
      } else {
        toast.error(result.error || 'Failed to deploy node');
        setIsDeploying(false);
      }
    } catch (error: unknown) {
      console.error(error);
      toast.error(getErrorMessage(error, 'An unexpected error occurred'));
      setIsDeploying(false);
    }
  };

  return (
    <div className="min-h-screen pb-12">
      {/* Header */}
      <div className="mb-8">
        <Link href="/app/endpoints" className="inline-flex items-center text-sm text-gray-400 hover:text-white mb-6 transition-colors">
          <ArrowLeft className="w-4 h-4 mr-2" /> Back to Endpoints
        </Link>
        <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Deploy Node</h1>
        <p className="text-gray-400 text-lg">Configure and launch your enterprise-grade infrastructure.</p>
      </div>

      <div className="flex flex-col xl:flex-row gap-8 items-start">
        
        {/* Main Configuration Form - Left Column */}
        <div className="flex-1 space-y-8 w-full">
          
          {/* Section 0: Node Name */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <h2 className="text-xl font-bold text-white mb-4">Node Name</h2>
            <input 
              type="text" 
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Production Mainnet Node"
              className="w-full bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#00E599] focus:ring-1 focus:ring-[#00E599] transition-all"
            />
          </section>

          {/* Section 0.5: Protocol */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">1</div>
              <h2 className="text-xl font-bold text-white">Select Protocol</h2>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
              <div 
                onClick={() => handleProtocolChange('neo-n3')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  protocol === 'neo-n3' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {protocol === 'neo-n3' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <div className="flex items-center gap-4 mb-3">
                  <div className="w-12 h-12 rounded-lg bg-[#00E599] flex items-center justify-center text-black font-bold text-xl shadow-[0_0_15px_rgba(0,229,153,0.3)]">N3</div>
                  <h3 className="text-lg font-bold text-white">Neo N3</h3>
                </div>
                <p className="text-sm text-gray-400 leading-relaxed">The powerful, feature-rich core blockchain. Native smart contracts in multiple languages.</p>
              </div>

              <div 
                onClick={() => handleProtocolChange('neo-x')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  protocol === 'neo-x' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {protocol === 'neo-x' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <div className="flex items-center gap-4 mb-3">
                  <div className="w-12 h-12 rounded-lg bg-purple-500 flex items-center justify-center text-white font-bold text-xl shadow-[0_0_15px_rgba(168,85,247,0.3)]">X</div>
                  <h3 className="text-lg font-bold text-white">Neo X</h3>
                </div>
                <p className="text-sm text-gray-400 leading-relaxed">EVM-compatible sidechain. Deploy Solidity contracts and access the Ethereum ecosystem.</p>
              </div>
            </div>
          </section>

          {/* Section 1: Network */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">2</div>
              <h2 className="text-xl font-bold text-white">Select Network</h2>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
              <div 
                onClick={() => setNetwork('mainnet')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  network === 'mainnet' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {network === 'mainnet' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <div className="flex items-center gap-4 mb-3">
                  <div className="w-10 h-10 rounded-lg bg-[#00E599] flex items-center justify-center text-black font-bold text-xl shadow-[0_0_15px_rgba(0,229,153,0.3)]">M</div>
                  <h3 className="text-lg font-bold text-white">Mainnet</h3>
                </div>
                <p className="text-sm text-gray-400 leading-relaxed">Production environment. Real assets and state.</p>
              </div>

              <div 
                onClick={() => setNetwork('testnet')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  network === 'testnet' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {network === 'testnet' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <div className="flex items-center gap-4 mb-3">
                  <div className="w-10 h-10 rounded-lg bg-[#333333] border border-gray-600 flex items-center justify-center text-white font-bold text-xl">T</div>
                  <h3 className="text-lg font-bold text-white">Testnet</h3>
                </div>
                <p className="text-sm text-gray-400 leading-relaxed">Development. Free GAS for testing smart contracts.</p>
              </div>

              <div 
                onClick={() => setNetwork('private')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  network === 'private' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {network === 'private' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <div className="flex items-center gap-4 mb-3">
                  <div className="w-10 h-10 rounded-lg bg-yellow-500/20 border border-yellow-500/30 flex items-center justify-center text-yellow-500 font-bold text-xl">P</div>
                  <h3 className="text-lg font-bold text-white">Private Net</h3>
                </div>
                <p className="text-sm text-gray-400 leading-relaxed">Your own isolated Neo Genesis block. Perfect for CI/CD.</p>
              </div>
            </div>
          </section>

          {/* Section 2: Client Engine */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">3</div>
              <h2 className="text-xl font-bold text-white">Client Engine</h2>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
              {protocol === 'neo-n3' ? (
                <>
                  <div 
                    onClick={() => setClientEngine('neo-go')}
                    className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                      clientEngine === 'neo-go' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                    }`}
                  >
                    {clientEngine === 'neo-go' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                    <div className="flex items-center gap-3 mb-3">
                      <span className="text-2xl">🐹</span>
                      <h3 className="text-lg font-bold text-white">neo-go</h3>
                      <span className="bg-[#00E599]/20 text-[#00E599] text-[10px] px-2 py-0.5 rounded font-bold">RECOMMENDED</span>
                    </div>
                    <p className="text-sm text-gray-400 leading-relaxed">
                      Written in Go. Highest performance, lowest memory footprint, and blazing fast RPC responses. Default for heavy infrastructure.
                    </p>
                  </div>

                  <div 
                    onClick={() => setClientEngine('neo-cli')}
                    className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                      clientEngine === 'neo-cli' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                    }`}
                  >
                    {clientEngine === 'neo-cli' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                    <div className="flex items-center gap-3 mb-3">
                      <span className="text-2xl">💎</span>
                      <h3 className="text-lg font-bold text-white">neo-cli (C#)</h3>
                      <span className="bg-blue-500/20 text-blue-400 text-[10px] px-2 py-0.5 rounded font-bold">OFFICIAL</span>
                    </div>
                    <p className="text-sm text-gray-400 leading-relaxed">
                      The official reference implementation developed by Neo Foundation. 100% consensus compatibility and native plugin support.
                    </p>
                  </div>
                </>
              ) : (
                <div 
                  onClick={() => setClientEngine('neo-x-geth')}
                  className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 border-[#00E599] bg-[#00E599]/5`}
                >
                  <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />
                  <div className="flex items-center gap-3 mb-3">
                    <span className="text-2xl">⚡</span>
                    <h3 className="text-lg font-bold text-white">neo-x-geth</h3>
                    <span className="bg-[#00E599]/20 text-[#00E599] text-[10px] px-2 py-0.5 rounded font-bold">EVM</span>
                  </div>
                  <p className="text-sm text-gray-400 leading-relaxed">
                    Official execution client for Neo X. Fully compatible with Ethereum tooling like Hardhat, Truffle, and Metamask.
                  </p>
                </div>
              )}
            </div>
          </section>

          {/* Section 3: Node Type */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">3</div>
              <h2 className="text-xl font-bold text-white">Node Type</h2>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
              <div 
                onClick={() => setNodeType('shared')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  nodeType === 'shared' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {nodeType === 'shared' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <Layers className={`w-8 h-8 mb-4 ${nodeType === 'shared' ? 'text-[#00E599]' : 'text-gray-400 group-hover:text-gray-300'}`} />
                <h3 className="text-lg font-bold text-white mb-2">Elastic (Shared)</h3>
                <p className="text-sm text-gray-400 mb-4 h-10">Auto-scaling global endpoints. Ideal for lightweight DApps and basic querying.</p>
                <div className="pt-4 border-t border-gray-800 flex items-center justify-between">
                  <span className="text-sm font-medium text-gray-400">Rate limited</span>
                  <span className="text-lg font-bold text-white">Free</span>
                </div>
              </div>

              <div 
                onClick={() => setNodeType('dedicated')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 overflow-hidden ${
                  nodeType === 'dedicated' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                <div className="absolute top-0 right-0 bg-gradient-to-r from-[#00E599] to-blue-500 text-black text-xs font-bold px-4 py-1.5 rounded-bl-xl shadow-md">RECOMMENDED</div>
                {nodeType === 'dedicated' && <CheckCircle2 className="absolute top-10 right-4 w-6 h-6 text-[#00E599]" />}
                <Server className={`w-8 h-8 mb-4 ${nodeType === 'dedicated' ? 'text-[#00E599]' : 'text-gray-400 group-hover:text-gray-300'}`} />
                <h3 className="text-lg font-bold text-white mb-2">Dedicated</h3>
                <p className="text-sm text-gray-400 mb-4 h-10">Private K8s deployment. Unlimited requests, custom plugins, max performance.</p>
                <div className="pt-4 border-t border-gray-800 flex items-center justify-between">
                  <span className="text-sm font-medium text-gray-400">Unlimited QPS</span>
                  <span className="text-lg font-bold text-white">From $99<span className="text-xs text-gray-500 font-normal">/mo</span></span>
                </div>
              </div>
            </div>
          </section>

          {/* Section 4: Cloud Provider & Region (Only if Dedicated) */}
          <div className={`transition-all duration-500 overflow-hidden ${nodeType === 'dedicated' ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}>
            <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">4</div>
                <h2 className="text-xl font-bold text-white">Hosting Configuration</h2>
              </div>

              <div className="space-y-8">
                {/* Providers */}
                <div>
                  <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider mb-4">Cloud Provider</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div 
                      onClick={() => setProvider('aws')}
                      className={`cursor-pointer rounded-xl p-4 border-2 flex items-center gap-4 transition-all ${
                        provider === 'aws' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                      }`}
                    >
                      <div className="w-12 h-12 rounded bg-gray-800 flex items-center justify-center">
                        <Cloud className={`w-6 h-6 ${provider === 'aws' ? 'text-[#00E599]' : 'text-gray-300'}`} />
                      </div>
                      <div className="flex-1">
                        <h4 className="font-bold text-white">Amazon Web Services</h4>
                        <p className="text-xs text-gray-400">Enterprise tier</p>
                      </div>
                      {provider === 'aws' && <CheckCircle2 className="w-5 h-5 text-[#00E599]" />}
                    </div>
                    
                    <div 
                      onClick={() => setProvider('gcp')}
                      className={`cursor-pointer rounded-xl p-4 border-2 flex items-center gap-4 transition-all ${
                        provider === 'gcp' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                      }`}
                    >
                      <div className="w-12 h-12 rounded bg-gray-800 flex items-center justify-center">
                        <Server className={`w-6 h-6 ${provider === 'gcp' ? 'text-[#00E599]' : 'text-gray-300'}`} />
                      </div>
                      <div className="flex-1">
                        <h4 className="font-bold text-white">Google Cloud</h4>
                        <p className="text-xs text-gray-400">Premium network tier</p>
                      </div>
                      {provider === 'gcp' && <CheckCircle2 className="w-5 h-5 text-[#00E599]" />}
                    </div>
                  </div>
                </div>

                {/* Regions */}
                <div>
                  <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider mb-4 flex items-center gap-2">
                    Region <Info className="w-4 h-4 text-gray-500 cursor-help" />
                  </h3>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                    {[
                      { id: 'ap-northeast-1', name: 'Tokyo', flag: '🇯🇵' },
                      { id: 'eu-central-1', name: 'Frankfurt', flag: '🇩🇪' },
                      { id: 'us-east-1', name: 'N. Virginia', flag: '🇺🇸' },
                      { id: 'ap-southeast-1', name: 'Singapore', flag: '🇸🇬' },
                    ].map((r) => (
                      <div 
                        key={r.id}
                        onClick={() => setRegion(r.id)}
                        className={`cursor-pointer rounded-lg p-3 border text-center transition-all ${
                          region === r.id ? 'border-[#00E599] bg-[#00E599]/5 text-white' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)] text-gray-400'
                        }`}
                      >
                        <div className="text-xl mb-1">{r.flag}</div>
                        <div className="text-sm font-medium">{r.name}</div>
                        <div className="text-[10px] opacity-70 mt-1">{r.id}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </section>
          </div>

          {/* Section 5: Node Configuration */}
          <section className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 rounded-full bg-[#00E599]/10 text-[#00E599] flex items-center justify-center font-bold">
                {nodeType === 'dedicated' ? '5' : '4'}
              </div>
              <h2 className="text-xl font-bold text-white">Node Mode</h2>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
              <div 
                onClick={() => setSyncMode('full')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  syncMode === 'full' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {syncMode === 'full' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <Zap className={`w-6 h-6 mb-3 ${syncMode === 'full' ? 'text-[#00E599]' : 'text-gray-400'}`} />
                <h3 className="text-base font-bold text-white mb-2">Full Node (RPC)</h3>
                <p className="text-sm text-gray-400 mb-2">Maintains recent state. Fastest sync time. Best for standard DApps, sending TXs, and querying recent blocks.</p>
                <div className="text-xs px-2 py-1 bg-gray-800 text-gray-300 rounded inline-block">Storage: ~120 GB</div>
              </div>

              <div 
                onClick={() => setSyncMode('archive')}
                className={`group relative cursor-pointer rounded-xl p-6 border-2 transition-all duration-200 ${
                  syncMode === 'archive' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500 bg-[var(--color-dark-panel)]'
                }`}
              >
                {syncMode === 'archive' && <CheckCircle2 className="absolute top-4 right-4 w-6 h-6 text-[#00E599]" />}
                <HardDrive className={`w-6 h-6 mb-3 ${syncMode === 'archive' ? 'text-[#00E599]' : 'text-gray-400'}`} />
                <h3 className="text-base font-bold text-white mb-2">Archive Node</h3>
                <p className="text-sm text-gray-400 mb-2">Retains all historical states since genesis. Required for deep historical indexing and complex analytics.</p>
                <div className="flex items-center justify-between mt-auto">
                  <div className="text-xs px-2 py-1 bg-yellow-500/10 text-yellow-500 border border-yellow-500/20 rounded inline-block">Storage: 2+ TB</div>
                  <div className="text-sm font-bold text-white">+${nodeType === 'shared' ? '49' : '50'}/mo</div>
                </div>
              </div>
            </div>
          </section>

        </div>

        {/* Right Column - Order Summary Fixed Panel */}
        <div className="w-full xl:w-96 xl:sticky xl:top-8 shrink-0">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl shadow-xl overflow-hidden flex flex-col">
            
            <div className="p-6 bg-[var(--color-dark-panel)] border-b border-[var(--color-dark-border)]">
              <h2 className="text-lg font-bold text-white flex items-center gap-2">
                Deployment Summary
              </h2>
            </div>
            
            <div className="p-6 space-y-6 flex-1">
              {/* Protocol info */}
              <div className="flex items-center gap-4">
                <div className="w-10 h-10 rounded-lg bg-[#00E599]/20 flex items-center justify-center">
                  <div className={`w-6 h-6 rounded flex items-center justify-center text-sm font-bold ${protocol === 'neo-n3' ? 'bg-[#00E599] text-black' : 'bg-purple-500 text-white'}`}>
                    {protocol === 'neo-n3' ? 'N3' : 'X'}
                  </div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 font-medium">Protocol</div>
                  <div className="text-base font-bold text-white">Neo {protocol === 'neo-n3' ? 'N3' : 'X'} {network === 'mainnet' ? 'Mainnet' : 'Testnet'}</div>
                </div>
              </div>

              <div className="w-full h-px bg-[#333333]"></div>

              {/* Details List */}
              <ul className="space-y-4">
                <li className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Node Type</span>
                  <span className="text-sm font-medium text-white capitalize">{nodeType}</span>
                </li>
                {nodeType === 'dedicated' && (
                  <li className="flex justify-between items-center">
                    <span className="text-sm text-gray-400">Infrastructure</span>
                    <span className="text-sm font-medium text-white uppercase flex items-center gap-2">
                      {provider === 'aws' ? <Cloud className="w-3 h-3 text-[#00E599]"/> : <Server className="w-3 h-3 text-[#00E599]"/>}
                      {provider} • {region}
                    </span>
                  </li>
                )}
                <li className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Client Engine</span>
                  <span className="text-sm font-medium text-white bg-gray-800 px-2 py-0.5 rounded">
                    {clientEngine === 'neo-go' ? 'neo-go v0.106.0' : (clientEngine === 'neo-cli' ? 'neo-cli v3.7.4' : 'neo-x-geth latest')}
                  </span>
                </li>
                <li className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Sync Mode</span>
                  <span className="text-sm font-medium text-white capitalize">{syncMode} Node</span>
                </li>
                <li className="flex justify-between items-center">
                  <span className="text-sm text-gray-400">Setup Time</span>
                  <span className="text-sm font-medium text-[#00E599] flex items-center gap-1">
                    <Zap className="w-3 h-3" />
                    {nodeType === 'shared' ? 'Instant' : '~2 Minutes'}
                  </span>
                </li>
              </ul>
            </div>

            <div className="p-6 bg-[var(--color-dark-panel)] border-t border-[var(--color-dark-border)]">
              <div className="flex justify-between items-end mb-6">
                <div>
                  <div className="text-sm text-gray-400 mb-1">Estimated Cost</div>
                  <div className="text-3xl font-bold text-white flex items-baseline gap-1">
                    ${calculatePrice()}
                    <span className="text-sm font-normal text-gray-500">/ month</span>
                  </div>
                </div>
              </div>

              <button 
                onClick={handleDeploy}
                disabled={isDeploying}
                className="w-full bg-[#00E599] hover:bg-[#00cc88] text-black font-bold py-4 rounded-xl shadow-[0_0_20px_rgba(0,229,153,0.3)] transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed group"
              >
                {isDeploying ? (
                  <>
                    <svg className="animate-spin -ml-1 mr-2 h-5 w-5 text-black" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Provisioning Node...
                  </>
                ) : (
                  <>
                    Deploy Node
                    <ChevronRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                  </>
                )}
              </button>
              
              <p className="text-xs text-center text-gray-500 mt-4">
                By deploying, you agree to our Terms of Service and Privacy Policy.
              </p>
            </div>
          </div>
        </div>

      </div>
    </div>
  );
}
