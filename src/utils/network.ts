export function getNetworkMagic(network: 'mainnet' | 'testnet' | 'private'): number {
  switch (network) {
    case 'mainnet':
      return 860833102;
    case 'testnet':
      return 894710606;
    case 'private':
      return 56753;
    default:
      return 860833102;
  }
}

export function getSeedList(network: 'mainnet' | 'testnet'): string[] {
  if (network === 'mainnet') {
    return [
      'seed1.neo.org:10333',
      'seed2.neo.org:10333',
      'seed3.neo.org:10333',
      'seed4.neo.org:10333',
      'seed5.neo.org:10333',
    ];
  } else {
    return [
      'seed1t.neo.org:20333',
      'seed2t.neo.org:20333',
      'seed3t.neo.org:20333',
      'seed4t.neo.org:20333',
      'seed5t.neo.org:20333',
    ];
  }
}
