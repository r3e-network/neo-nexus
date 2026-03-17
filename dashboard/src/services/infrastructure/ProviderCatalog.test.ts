import { describe, expect, it } from 'vitest';
import {
  DEFAULT_PROVIDER,
  getDefaultRegion,
  getProviderSummary,
  isSupportedProvider,
  listSupportedProviders,
  resolveInfrastructureSelection,
  resolveStorageClass,
} from './ProviderCatalog';

describe('ProviderCatalog', () => {
  it('defaults to Hetzner as the primary provider', () => {
    expect(DEFAULT_PROVIDER).toBe('hetzner');
    expect(getDefaultRegion(DEFAULT_PROVIDER)).toBe('fsn1');
    expect(resolveStorageClass(DEFAULT_PROVIDER)).toBe('hcloud-volumes');
  });

  it('keeps DigitalOcean as the fallback provider', () => {
    expect(resolveStorageClass('digitalocean')).toBe('do-block-storage');
    expect(getDefaultRegion('digitalocean')).toBe('fra1');
  });

  it('lists providers in primary-then-backup order', () => {
    expect(listSupportedProviders().map((provider) => provider.id)).toEqual([
      'hetzner',
      'digitalocean',
    ]);
  });

  it('normalizes invalid input to the primary provider defaults', () => {
    expect(resolveInfrastructureSelection('aws', 'us-east-1')).toEqual({
      provider: 'hetzner',
      region: 'fsn1',
    });
  });

  it('keeps a valid provider and falls back to its default region when needed', () => {
    expect(resolveInfrastructureSelection('digitalocean', 'invalid')).toEqual({
      provider: 'digitalocean',
      region: 'fra1',
    });
  });

  it('exposes provider metadata for UI summaries', () => {
    const summary = getProviderSummary('hetzner');

    expect(summary).toMatchObject({
      id: 'hetzner',
      name: 'Hetzner Cloud',
      role: 'Primary',
    });
    expect(isSupportedProvider('digitalocean')).toBe(true);
    expect(isSupportedProvider('aws')).toBe(false);
  });
});
