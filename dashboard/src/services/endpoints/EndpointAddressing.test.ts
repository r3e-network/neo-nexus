import { describe, expect, it } from 'vitest';
import { buildPlannedEndpointAddress } from './EndpointAddressing';

describe('EndpointAddressing', () => {
  it('builds dedicated planned addresses from region and route key', () => {
    const address = buildPlannedEndpointAddress({
      type: 'dedicated',
      protocol: 'neo-x',
      networkKey: 'mainnet',
      region: 'fsn1',
      routeKey: '42',
      rootDomain: 'neonexus.cloud',
    });

    expect(address.httpsUrl).toBe('https://node-fsn1-42.neonexus.cloud/v1');
    expect(address.wssUrl).toBe('wss://node-fsn1-42.neonexus.cloud/ws');
  });

  it('builds shared planned addresses from network and route key', () => {
    const address = buildPlannedEndpointAddress({
      type: 'shared',
      protocol: 'neo-n3',
      networkKey: 'mainnet',
      routeKey: '84',
      rootDomain: 'neonexus.cloud',
    });

    expect(address.httpsUrl).toBe('https://mainnet.neonexus.cloud/v1/84');
    expect(address.wssUrl).toBeNull();
  });

  it('falls back to the default root domain when one is not provided', () => {
    const address = buildPlannedEndpointAddress({
      type: 'dedicated',
      protocol: 'neo-n3',
      networkKey: 'mainnet',
      region: 'ash',
      routeKey: '55',
    });

    expect(address.httpsUrl).toBe('https://node-ash-55.neonexus.cloud/v1');
  });
});
