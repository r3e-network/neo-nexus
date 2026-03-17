import type { InfrastructureProvider } from '@/services/infrastructure/ProviderCatalog';

type LifecycleAction = 'restart' | 'stop' | 'destroy';

type LifecycleInput = {
  provider: InfrastructureProvider;
  providerServerId: string;
  action: LifecycleAction;
};

type LifecycleDependencies = {
  env?: Record<string, string | undefined>;
  fetchImpl?: typeof fetch;
};

function getEnvValue(env: Record<string, string | undefined>, key: string): string {
  const value = env[key];
  if (!value) {
    throw new Error(`${key} must be configured for node lifecycle actions.`);
  }

  return value;
}

async function handleHetznerAction(
  input: LifecycleInput,
  env: Record<string, string | undefined>,
  fetchImpl: typeof fetch,
) {
  const token = getEnvValue(env, 'HETZNER_API_TOKEN');
  const serverId = input.providerServerId;

  if (input.action === 'destroy') {
    const response = await fetchImpl(`https://api.hetzner.cloud/v1/servers/${serverId}`, {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    if (!response.ok) {
      throw new Error(`Hetzner destroy failed: ${response.status} ${await response.text()}`);
    }

    return true;
  }

  const actionPath = input.action === 'restart' ? 'reboot' : 'shutdown';
  const response = await fetchImpl(`https://api.hetzner.cloud/v1/servers/${serverId}/actions/${actionPath}`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Hetzner lifecycle action failed: ${response.status} ${await response.text()}`);
  }

  return true;
}

async function handleDigitalOceanAction(
  input: LifecycleInput,
  env: Record<string, string | undefined>,
  fetchImpl: typeof fetch,
) {
  const token = getEnvValue(env, 'DIGITALOCEAN_API_TOKEN');
  const dropletId = input.providerServerId;

  if (input.action === 'destroy') {
    const response = await fetchImpl(`https://api.digitalocean.com/v2/droplets/${dropletId}`, {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    if (!response.ok) {
      throw new Error(`DigitalOcean destroy failed: ${response.status} ${await response.text()}`);
    }

    return true;
  }

  const actionType = input.action === 'restart' ? 'reboot' : 'shutdown';
  const response = await fetchImpl(`https://api.digitalocean.com/v2/droplets/${dropletId}/actions`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ type: actionType }),
  });

  if (!response.ok) {
    throw new Error(`DigitalOcean lifecycle action failed: ${response.status} ${await response.text()}`);
  }

  return true;
}

export async function performProviderLifecycleAction(
  input: LifecycleInput,
  dependencies: LifecycleDependencies = {},
) {
  const env = dependencies.env ?? process.env;
  const fetchImpl = dependencies.fetchImpl ?? fetch;

  if (input.provider === 'hetzner') {
    return handleHetznerAction(input, env, fetchImpl);
  }

  return handleDigitalOceanAction(input, env, fetchImpl);
}
