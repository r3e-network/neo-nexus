import { describe, expect, it, vi } from 'vitest';
import { performProviderLifecycleAction } from './VmLifecycleService';

describe('VmLifecycleService', () => {
  it('sends the Hetzner reboot action to the server action endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ action: { id: 1 } }),
    });

    await performProviderLifecycleAction(
      {
        provider: 'hetzner',
        providerServerId: '123',
        action: 'restart',
      },
      {
        env: { HETZNER_API_TOKEN: 'token' },
        fetchImpl: fetchMock as typeof fetch,
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      'https://api.hetzner.cloud/v1/servers/123/actions/reboot',
      expect.objectContaining({ method: 'POST' }),
    );
  });

  it('sends the DigitalOcean shutdown action using droplet actions', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ action: { id: 2 } }),
    });

    await performProviderLifecycleAction(
      {
        provider: 'digitalocean',
        providerServerId: '456',
        action: 'stop',
      },
      {
        env: { DIGITALOCEAN_API_TOKEN: 'token' },
        fetchImpl: fetchMock as typeof fetch,
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      'https://api.digitalocean.com/v2/droplets/456/actions',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ type: 'shutdown' }),
      }),
    );
  });

  it('destroys a Hetzner server with DELETE', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      text: async () => '',
    });

    await performProviderLifecycleAction(
      {
        provider: 'hetzner',
        providerServerId: '789',
        action: 'destroy',
      },
      {
        env: { HETZNER_API_TOKEN: 'token' },
        fetchImpl: fetchMock as typeof fetch,
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      'https://api.hetzner.cloud/v1/servers/789',
      expect.objectContaining({ method: 'DELETE' }),
    );
  });
});
