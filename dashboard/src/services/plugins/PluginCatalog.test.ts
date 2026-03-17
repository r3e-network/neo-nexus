import { describe, expect, it } from 'vitest';
import {
  getPluginDefinition,
  isSupportedPlugin,
  listSupportedPlugins,
} from './PluginCatalog';

describe('PluginCatalog', () => {
  it('lists supported plugins with deterministic ids', () => {
    expect(listSupportedPlugins().map((plugin) => plugin.id)).toEqual([
      'tee-oracle',
      'aa-bundler',
      'tee-mempool',
    ]);
  });

  it('marks signing-key requirements correctly', () => {
    expect(getPluginDefinition('tee-oracle')?.requiresPrivateKey).toBe(true);
    expect(getPluginDefinition('aa-bundler')?.requiresPrivateKey).toBe(true);
    expect(getPluginDefinition('tee-mempool')?.defaultImage).toContain('tee-mempool');
  });

  it('rejects unsupported plugin ids', () => {
    expect(isSupportedPlugin('unknown-plugin')).toBe(false);
    expect(getPluginDefinition('unknown-plugin')).toBeNull();
  });
});
