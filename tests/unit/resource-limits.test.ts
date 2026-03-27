import { describe, it, expect } from 'vitest';
import { buildResourceEnv } from '../../src/utils/resourceLimits';

describe('buildResourceEnv', () => {
  it('sets GOMEMLIMIT for neo-go with 4096 MB', () => {
    const env = buildResourceEnv('neo-go', { maxMemoryMB: 4096 });
    expect(env.GOMEMLIMIT).toBe('4096MiB');
    expect(Object.keys(env)).toHaveLength(1);
  });

  it('sets DOTNET_GCHeapHardLimit for neo-cli with 4096 MB', () => {
    const env = buildResourceEnv('neo-cli', { maxMemoryMB: 4096 });
    const expectedHex = (4096 * 1024 * 1024).toString(16);
    expect(env.DOTNET_GCHeapHardLimit).toBe(expectedHex);
    expect(Object.keys(env)).toHaveLength(1);
  });

  it('returns empty object when no limits specified', () => {
    expect(buildResourceEnv('neo-go', {})).toEqual({});
    expect(buildResourceEnv('neo-cli', {})).toEqual({});
  });

  it('returns empty object when maxMemoryMB is 0', () => {
    expect(buildResourceEnv('neo-go', { maxMemoryMB: 0 })).toEqual({});
  });

  it('sets GOMEMLIMIT for neo-go with 512 MB', () => {
    const env = buildResourceEnv('neo-go', { maxMemoryMB: 512 });
    expect(env.GOMEMLIMIT).toBe('512MiB');
  });

  it('sets DOTNET_GCHeapHardLimit correctly for 1024 MB', () => {
    const env = buildResourceEnv('neo-cli', { maxMemoryMB: 1024 });
    expect(env.DOTNET_GCHeapHardLimit).toBe((1024 * 1024 * 1024).toString(16));
  });
});
