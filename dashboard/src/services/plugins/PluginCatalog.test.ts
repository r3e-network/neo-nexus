import { describe, expect, it } from 'vitest';
import {
  getPluginDefinition,
  isSupportedPlugin,
  listSupportedPlugins,
} from './PluginCatalog';

describe('PluginCatalog', () => {
  it('lists supported plugins including official Neo plugins', () => {
    const ids = listSupportedPlugins().map((plugin) => plugin.id);
    
    // Original plugins
    expect(ids).toContain('tee-oracle');
    expect(ids).toContain('aa-bundler');
    expect(ids).toContain('tee-mempool');
    
    // Official Neo Plugins
    expect(ids).toContain('ApplicationLogs');
    expect(ids).toContain('DBFTPlugin');
    expect(ids).toContain('LevelDBStore');
    expect(ids).toContain('OracleService');
    expect(ids).toContain('RestServer');
    expect(ids).toContain('RocksDBStore');
    expect(ids).toContain('RpcServer');
    expect(ids).toContain('StateService');
    expect(ids).toContain('TokensTracker');
  });

  it('marks signing-key requirements correctly', () => {
    expect(getPluginDefinition('tee-oracle')?.requiresPrivateKey).toBe(true);
    expect(getPluginDefinition('aa-bundler')?.requiresPrivateKey).toBe(true);
    expect(getPluginDefinition('DBFTPlugin')?.requiresPrivateKey).toBe(true);
    
    expect(getPluginDefinition('ApplicationLogs')?.requiresPrivateKey).toBe(false);
    expect(getPluginDefinition('RpcServer')?.requiresPrivateKey).toBe(false);
  });

  it('provides schema definitions for configurable plugins', () => {
    const rpcServer = getPluginDefinition('RpcServer');
    expect(rpcServer?.schema).toBeDefined();
    expect(rpcServer?.schema?.[0].key).toBe('DisabledMethods');

    const appLogs = getPluginDefinition('ApplicationLogs');
    expect(appLogs?.schema).toBeDefined();
  });

  it('rejects unsupported plugin ids', () => {
    expect(isSupportedPlugin('unknown-plugin' as any)).toBe(false);
    expect(getPluginDefinition('unknown-plugin' as any)).toBeNull();
  });
});
