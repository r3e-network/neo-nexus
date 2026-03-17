import { describe, expect, it, vi, beforeEach } from 'vitest';
import { resolveInfrastructureSelection } from '../services/infrastructure/ProviderCatalog';
import { getSharedBackendTarget } from '../services/endpoints/SharedEndpointConfig';
import { mergeNodeSettings } from '../services/settings/NodeSettings';
import { renderNodeRuntimeArtifacts } from '../services/settings/RenderedNodeRuntime';

describe('Edge Cases and Resilience', () => {

  describe('Infrastructure Provider Resilience', () => {
    it('gracefully falls back to default provider if unknown provider is requested', () => {
      const selection = resolveInfrastructureSelection('invalid-provider', 'unknown-region');
      expect(selection.provider).toBe('hetzner'); // DEFAULT_PROVIDER
      expect(selection.region).toBe('fsn1'); // getDefaultRegion('hetzner')
    });

    it('gracefully falls back to default region if unknown region is requested for valid provider', () => {
      const selection = resolveInfrastructureSelection('digitalocean', 'moon-base-1');
      expect(selection.provider).toBe('digitalocean');
      expect(selection.region).toBe('fra1'); // getDefaultRegion('digitalocean')
    });
  });

  describe('Shared Endpoint Routing', () => {
    beforeEach(() => {
      vi.stubEnv('SHARED_NEO_N3_MAINNET_UPSTREAM', '10.0.1.100:10332');
      vi.stubEnv('SHARED_NEO_N3_TESTNET_UPSTREAM', '10.0.1.101:10332');
      vi.stubEnv('SHARED_NEO_X_MAINNET_UPSTREAM', '10.0.1.200:8545');
    });

    it('resolves correct upstream targets for different networks', () => {
      const mainnetN3 = getSharedBackendTarget('neo-n3', 'mainnet');
      expect(mainnetN3.host).toBe('10.0.1.100');
      
      const testnetN3 = getSharedBackendTarget('neo-n3', 'testnet');
      expect(testnetN3.host).toBe('10.0.1.101');

      const mainnetX = getSharedBackendTarget('neo-x', 'mainnet');
      expect(mainnetX.host).toBe('10.0.1.200');
    });

    it('throws error for unsupported private shared network', () => {
      expect(() => getSharedBackendTarget('neo-n3', 'private')).toThrow(/Private networks are not supported/);
    });
  });

  describe('Node Settings Edge Cases', () => {
    it('merges completely undefined or null user settings cleanly', () => {
      const merged = mergeNodeSettings('neo-go', null);
      expect(merged.maxPeers).toBe(100);
      expect(merged.rpcEnabled).toBe(true);

      const mergedUndefined = mergeNodeSettings('neo-go', undefined);
      expect(mergedUndefined.maxPeers).toBe(100);
    });

    it('ignores invalid types in partial user settings', () => {
      const merged = mergeNodeSettings('neo-go', { maxPeers: 'not-a-number', rpcEnabled: 'yes' });
      // It should ignore the bad types and use defaults
      expect(merged.maxPeers).toBe(100);
      expect(merged.rpcEnabled).toBe(true);
    });
  });

  describe('Node Runtime Generation', () => {
    it('escapes environment variables to prevent basic injection', () => {
      const settings = mergeNodeSettings('neo-cli', {
        envVars: {
          'MALICIOUS': 'value" && rm -rf / && echo "hacked'
        }
      });

      const runtime = renderNodeRuntimeArtifacts({
        clientEngine: 'neo-cli',
        network: 'mainnet',
        settings
      });

      // The quotes should be escaped so they don't break out of the string
      expect(runtime.runCommand).toContain('-e MALICIOUS="value\\" && rm -rf / && echo \\"hacked"');
    });

    it('safely handles empty custom docker flags', () => {
      const settings = mergeNodeSettings('neo-cli', {
        customDockerFlags: ''
      });

      const runtime = renderNodeRuntimeArtifacts({
        clientEngine: 'neo-cli',
        network: 'mainnet',
        settings
      });

      // Shouldn't have dangling spaces or empty flags
      expect(runtime.runCommand).not.toContain('  ');
    });
  });

});
