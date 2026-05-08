import type { N3NodeNetwork } from '../types/index';

export function getNetworkMagic(network: Exclude<N3NodeNetwork, 'neox-mainnet' | 'neox-testnet'>): number {
  switch (network) {
    case 'mainnet':
      return 860833102;
    case 'testnet':
      return 894710606;
    case 'private':
      return 56753;
    default:
      throw new Error(`Unsupported network: ${String(network)}`);
  }
}

export function getSeedList(network: Exclude<N3NodeNetwork, 'private'>): string[] {
  if (network === 'mainnet') {
    return [
      'seed1.neo.org:10333',
      'seed2.neo.org:10333',
      'seed3.neo.org:10333',
      'seed4.neo.org:10333',
      'seed5.neo.org:10333',
    ];
  } else {
    // T5 testnet (post-2024 reset). The legacy seedNt.neo.org hosts no
    // longer resolve / serve RPC, so neither the network-height tracker
    // nor neo-go config generation worked against them — testnet nodes
    // showed "Latest network height unavailable" on the dashboard.
    return [
      'seed1t5.neo.org:20333',
      'seed2t5.neo.org:20333',
      'seed3t5.neo.org:20333',
      'seed4t5.neo.org:20333',
      'seed5t5.neo.org:20333',
    ];
  }
}
