import { buildNodeBootstrapScript } from './NodeBootstrap';
import type { DeploymentConfig } from '../infrastructure/DeploymentConfig';
import {
  DEFAULT_PROVIDER,
  getDefaultRegion,
  type InfrastructureProvider,
} from '../infrastructure/ProviderCatalog';

type ProvisionerEnv = Record<string, string | undefined>;

type ProvisionerDependencies = {
  env?: ProvisionerEnv;
  fetchImpl?: typeof fetch;
  sleep?: (ms: number) => Promise<void>;
};

type ProviderResult = {
  provider: InfrastructureProvider;
  region: string;
  publicIp: string;
  providerServerId: string;
  fallbackUsed: boolean;
};

function getEnvValue(env: ProvisionerEnv, key: string, fallbackKey?: string): string {
  const value = env[key] || (fallbackKey ? env[fallbackKey] : undefined);
  if (!value) {
    throw new Error(`${key}${fallbackKey ? ` or ${fallbackKey}` : ''} must be configured for VM provisioning.`);
  }

  return value;
}

function getProviderOrder(selectedProvider: InfrastructureProvider): InfrastructureProvider[] {
  if (selectedProvider === DEFAULT_PROVIDER) {
    return ['hetzner', 'digitalocean'];
  }

  return [selectedProvider];
}

function getPrimaryBootstrapConfig(config: DeploymentConfig) {
  return {
    endpointName: config.name,
    protocol: config.protocol,
    clientEngine: config.clientEngine,
    network: config.network,
    syncMode: config.syncMode,
    operatorPublicKey: process.env.VM_OPERATOR_PUBLIC_KEY,
  } as const;
}

async function createHetznerServer(
  config: DeploymentConfig,
  bootstrapScript: string,
  env: ProvisionerEnv,
  fetchImpl: typeof fetch,
): Promise<ProviderResult> {
  const token = getEnvValue(env, 'HETZNER_API_TOKEN', 'NEO_NEXUS_HETZNER');
  const image = env.HETZNER_IMAGE ?? 'ubuntu-24.04';
  const serverType = config.clientEngine === 'neo-x-geth' || config.syncMode === 'archive' ? 'cpx41' : 'cpx31';
  const response = await fetchImpl('https://api.hetzner.cloud/v1/servers', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      name: config.name.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
      server_type: serverType,
      image,
      location: config.region,
      user_data: bootstrapScript,
      labels: {
        app: 'neonexus',
        protocol: config.protocol,
        provider: 'hetzner',
      },
    }),
  });

  if (!response.ok) {
    throw new Error(`Hetzner provisioning failed: ${response.status} ${await response.text()}`);
  }

  const payload = await response.json() as {
    server: {
      id: number;
      public_net?: {
        ipv4?: { ip?: string | null } | null;
      } | null;
    };
  };

  const publicIp = payload.server.public_net?.ipv4?.ip ?? '';
  if (!publicIp) {
    throw new Error('Hetzner provisioning succeeded but no public IPv4 was returned.');
  }

  return {
    provider: 'hetzner',
    region: config.region,
    publicIp,
    providerServerId: String(payload.server.id),
    fallbackUsed: false,
  };
}

async function createDigitalOceanDroplet(
  config: DeploymentConfig,
  bootstrapScript: string,
  env: ProvisionerEnv,
  fetchImpl: typeof fetch,
): Promise<ProviderResult> {
  const token = getEnvValue(env, 'DIGITALOCEAN_API_TOKEN');
  const image = env.DIGITALOCEAN_IMAGE ?? 'ubuntu-24-04-x64';
  const size = config.clientEngine === 'neo-x-geth' || config.syncMode === 'archive' ? 's-8vcpu-16gb' : 's-4vcpu-8gb';
  const response = await fetchImpl('https://api.digitalocean.com/v2/droplets', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      name: config.name.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
      region: config.region,
      size,
      image,
      user_data: bootstrapScript,
      ipv6: true,
      monitoring: true,
      tags: ['neonexus', config.protocol],
    }),
  });

  if (!response.ok) {
    throw new Error(`DigitalOcean provisioning failed: ${response.status} ${await response.text()}`);
  }

  const payload = await response.json() as {
    droplet: {
      id: number;
      networks?: {
        v4?: Array<{ ip_address?: string; type?: string }>;
      };
    };
  };

  const publicIp = payload.droplet.networks?.v4?.find((network) => network.type === 'public')?.ip_address ?? '';
  if (!publicIp) {
    throw new Error('DigitalOcean provisioning succeeded but no public IPv4 was returned.');
  }

  return {
    provider: 'digitalocean',
    region: config.region,
    publicIp,
    providerServerId: String(payload.droplet.id),
    fallbackUsed: config.provider !== 'digitalocean',
  };
}

export async function provisionDedicatedNode(
  config: DeploymentConfig,
  dependencies: ProvisionerDependencies = {},
) {
  const env = dependencies.env ?? process.env;
  const fetchImpl = dependencies.fetchImpl ?? fetch;
  const bootstrapScript = buildNodeBootstrapScript(getPrimaryBootstrapConfig(config));
  const attempts = getProviderOrder(config.provider);
  const errors: string[] = [];

  for (const provider of attempts) {
    try {
      if (provider === 'hetzner') {
        const result = await createHetznerServer(config, bootstrapScript, env, fetchImpl);
        return result;
      }

      const result = await createDigitalOceanDroplet(
        { ...config, region: provider === config.provider ? config.region : getDefaultRegion('digitalocean') },
        bootstrapScript,
        env,
        fetchImpl,
      );
      return result;
    } catch (error) {
      errors.push(error instanceof Error ? error.message : 'Unknown provisioning error');
    }
  }

  throw new Error(errors.join(' | '));
}
