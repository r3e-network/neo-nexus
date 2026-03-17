import { describe, expect, it } from 'vitest';
import { renderPluginConfig } from './PluginConfigRenderer';

describe('PluginConfigRenderer', () => {
  it('renders tee-oracle config with secret references', () => {
    const rendered = renderPluginConfig({
      pluginId: 'tee-oracle',
      endpointId: 12,
      secretRefs: ['tee-oracle_private_key'],
      secretPayloads: {
        tee_oracle_private_key: 'L1abc',
      },
      runtimeImage: 'ghcr.io/neonexus/tee-oracle:latest',
      configData: {
        rpcTarget: 'https://node.example.com/v1',
      },
    });

    expect(rendered).toContain('"pluginId": "tee-oracle"');
    expect(rendered).toContain('tee-oracle_private_key');
    expect(rendered).toContain('rpcTarget');
  });

  it('renders aa-bundler config with generic payload data', () => {
    const rendered = renderPluginConfig({
      pluginId: 'aa-bundler',
      endpointId: 13,
      secretRefs: ['aa-bundler_private_key'],
      secretPayloads: {
        aa_bundler_private_key: 'L1xyz',
      },
      runtimeImage: 'ghcr.io/neonexus/aa-bundler:latest',
      configData: {
        bundlerUrl: 'https://bundler.example.com',
      },
    });

    expect(rendered).toContain('"pluginId": "aa-bundler"');
    expect(rendered).toContain('bundlerUrl');
  });

  it('renders official Neo plugin config (RpcServer) mapping configuration accurately', () => {
    const rendered = renderPluginConfig({
      pluginId: 'RpcServer',
      endpointId: 14,
      secretRefs: [],
      secretPayloads: {},
      runtimeImage: 'neo-cli-plugin',
      configData: {
        DisabledMethods: ['getversion'],
        Port: 10332
      },
    });

    const parsed = JSON.parse(rendered);
    expect(parsed.pluginId).toBe('RpcServer');
    expect(parsed.config.DisabledMethods).toEqual(['getversion']);
    expect(parsed.config.Port).toBe(10332);
  });
});
