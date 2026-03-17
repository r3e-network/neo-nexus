import { describe, expect, it } from 'vitest';
import {
  getSupportedNodeSettingKeys,
  renderNodeRuntimeArtifacts,
} from './RenderedNodeRuntime';

describe('RenderedNodeRuntime', () => {
  it('renders neo-go settings into config and run script artifacts', () => {
    const rendered = renderNodeRuntimeArtifacts({
      clientEngine: 'neo-go',
      network: 'mainnet',
      settings: {
        maxPeers: 42,
        rpcEnabled: false,
        websocketEnabled: false,
        graphqlEnabled: false,
        cacheMb: null,
      },
    });

    expect(rendered.neoGoConfig).toContain('MaxPeers: 42');
    expect(rendered.neoGoConfig).toContain('Enabled: false');
    expect(rendered.runScript).toContain('neonexus-node');
    expect(rendered.runCommand).toContain('nspccdev/neo-go:0.106.0');
  });

  it('renders neo-x geth flags from the persisted settings', () => {
    const rendered = renderNodeRuntimeArtifacts({
      clientEngine: 'neo-x-geth',
      network: 'mainnet',
      settings: {
        maxPeers: 60,
        rpcEnabled: true,
        websocketEnabled: true,
        graphqlEnabled: true,
        cacheMb: 2048,
      },
    });

    expect(rendered.runCommand).toContain('--http');
    expect(rendered.runCommand).toContain('--ws');
    expect(rendered.runCommand).toContain('--graphql');
    expect(rendered.runCommand).toContain('--cache 2048');
    expect(rendered.runCommand).toContain('--maxpeers 60');
  });

  it('reports only the supported setting keys for each engine', () => {
    expect(getSupportedNodeSettingKeys('neo-go')).toEqual(['maxPeers', 'rpcEnabled']);
    expect(getSupportedNodeSettingKeys('neo-cli')).toEqual(['rpcEnabled']);
    expect(getSupportedNodeSettingKeys('neo-x-geth')).toEqual([
      'maxPeers',
      'rpcEnabled',
      'websocketEnabled',
      'graphqlEnabled',
      'cacheMb',
    ]);
  });
});
