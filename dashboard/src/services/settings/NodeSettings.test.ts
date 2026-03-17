import { describe, expect, it } from 'vitest';
import {
  buildDefaultNodeSettings,
  mergeNodeSettings,
} from './NodeSettings';

describe('NodeSettings', () => {
  it('builds sensible defaults for neo-go nodes', () => {
    const settings = buildDefaultNodeSettings('neo-go');
    expect(settings.maxPeers).toBe(100);
    expect(settings.rpcEnabled).toBe(true);
    expect(settings.websocketEnabled).toBe(false);
  });

  it('builds sensible defaults for neo-x nodes', () => {
    const settings = buildDefaultNodeSettings('neo-x-geth');
    expect(settings.maxPeers).toBe(50);
    expect(settings.cacheMb).toBe(4096);
    expect(settings.websocketEnabled).toBe(true);
  });

  it('merges partial settings onto defaults', () => {
    const merged = mergeNodeSettings('neo-go', { maxPeers: 200 });
    expect(merged.maxPeers).toBe(200);
    expect(merged.rpcEnabled).toBe(true);
  });
});
