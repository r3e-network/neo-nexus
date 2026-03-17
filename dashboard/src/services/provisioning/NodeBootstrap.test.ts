import { describe, expect, it } from 'vitest';
import { buildNodeBootstrapScript } from './NodeBootstrap';

describe('NodeBootstrap', () => {
  it('builds a neo-go bootstrap script with runtime sync support', () => {
    const script = buildNodeBootstrapScript({
      endpointName: 'rpc-mainnet-1',
      protocol: 'neo-n3',
      clientEngine: 'neo-go',
      network: 'mainnet',
      syncMode: 'full',
    });

    expect(script).toContain('#cloud-config');
    expect(script).toContain('nspccdev/neo-go:0.106.0');
    expect(script).toContain('10332:10332');
    expect(script).toContain('2112:2112');
    expect(script).toContain('ProtocolConfiguration:');
    expect(script).toContain('glob.glob');
    expect(script).toContain('neonexus-plugin-sync.service');
    expect(script).toContain('neonexus-node-sync');
    expect(script).toContain('/etc/neonexus/node-settings.json');
  });

  it('builds a neo-x bootstrap script with websocket support', () => {
    const script = buildNodeBootstrapScript({
      endpointName: 'neo-x-1',
      protocol: 'neo-x',
      clientEngine: 'neo-x-geth',
      network: 'mainnet',
      syncMode: 'archive',
    });

    expect(script).toContain('neofoundation/neo-x-geth:latest');
    expect(script).toContain('8545:8545');
    expect(script).toContain('8546:8546');
    expect(script).toContain('--http --http.addr 0.0.0.0');
    expect(script).toContain('--ws --ws.addr 0.0.0.0');
  });
});
