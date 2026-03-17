import { describe, expect, it, vi } from 'vitest';
import { provisionDedicatedNode } from './VmProvisioner';

describe('VmProvisioner', () => {
  it('falls back to DigitalOcean when Hetzner provisioning fails', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: false,
        status: 500,
        text: async () => 'hetzner unavailable',
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          droplet: {
            id: 4242,
            status: 'active',
            networks: {
              v4: [{ ip_address: '203.0.113.10', type: 'public' }],
            },
          },
        }),
      });

    const result = await provisionDedicatedNode(
      {
        name: 'node-1',
        protocol: 'neo-n3',
        network: 'mainnet',
        type: 'dedicated',
        clientEngine: 'neo-go',
        provider: 'hetzner',
        region: 'fsn1',
        syncMode: 'full',
      },
      {
        env: {
          HETZNER_API_TOKEN: 'hetzner-token',
          DIGITALOCEAN_API_TOKEN: 'do-token',
        },
        fetchImpl: fetchMock as typeof fetch,
        sleep: async () => undefined,
      },
    );

    expect(result.provider).toBe('digitalocean');
    expect(result.fallbackUsed).toBe(true);
    expect(result.providerServerId).toBe('4242');
    expect(result.publicIp).toBe('203.0.113.10');
  });
});
