export type InfrastructureProvider = 'hetzner' | 'digitalocean';

export type ProviderRegion = {
  id: string;
  name: string;
  flag: string;
};

export type ProviderSummary = {
  id: InfrastructureProvider;
  name: string;
  shortName: string;
  role: 'Primary' | 'Fallback';
  description: string;
  storageClass: string;
  regions: ProviderRegion[];
};

export const DEFAULT_PROVIDER: InfrastructureProvider = 'hetzner';

const PROVIDERS: Record<InfrastructureProvider, ProviderSummary> = {
  hetzner: {
    id: 'hetzner',
    name: 'Hetzner Cloud',
    shortName: 'Hetzner',
    role: 'Primary',
    description: 'Primary deployment target with the best cost-performance for dedicated nodes.',
    storageClass: 'hcloud-volumes',
    regions: [
      { id: 'fsn1', name: 'Falkenstein', flag: '🇩🇪' },
      { id: 'nbg1', name: 'Nuremberg', flag: '🇩🇪' },
      { id: 'hel1', name: 'Helsinki', flag: '🇫🇮' },
    ],
  },
  digitalocean: {
    id: 'digitalocean',
    name: 'DigitalOcean',
    shortName: 'DigitalOcean',
    role: 'Fallback',
    description: 'Backup deployment target for overflow capacity and regional failover.',
    storageClass: 'do-block-storage',
    regions: [
      { id: 'fra1', name: 'Frankfurt', flag: '🇩🇪' },
      { id: 'nyc3', name: 'New York', flag: '🇺🇸' },
      { id: 'sfo3', name: 'San Francisco', flag: '🇺🇸' },
      { id: 'sgp1', name: 'Singapore', flag: '🇸🇬' },
    ],
  },
};

export function listSupportedProviders(): ProviderSummary[] {
  return [PROVIDERS.hetzner, PROVIDERS.digitalocean];
}

export function isSupportedProvider(value: string): value is InfrastructureProvider {
  return value in PROVIDERS;
}

export function getProviderSummary(provider: InfrastructureProvider): ProviderSummary {
  return PROVIDERS[provider];
}

export function getDefaultRegion(provider: InfrastructureProvider): string {
  return PROVIDERS[provider].regions[0].id;
}

export function resolveStorageClass(provider: InfrastructureProvider): string {
  return PROVIDERS[provider].storageClass;
}

function isSupportedRegion(provider: InfrastructureProvider, region: string): boolean {
  return PROVIDERS[provider].regions.some((item) => item.id === region);
}

export function resolveInfrastructureSelection(provider: string, region: string) {
  const normalizedProvider = isSupportedProvider(provider) ? provider : DEFAULT_PROVIDER;
  const normalizedRegion = isSupportedRegion(normalizedProvider, region)
    ? region
    : getDefaultRegion(normalizedProvider);

  return {
    provider: normalizedProvider,
    region: normalizedRegion,
  };
}
