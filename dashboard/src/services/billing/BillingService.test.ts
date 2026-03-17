import { describe, expect, it } from 'vitest';
import { BillingService } from './BillingService';

describe('BillingService', () => {
  it('prices Hetzner dedicated nodes as the primary profile', () => {
    expect(
      BillingService.calculateProjectedCost({
        type: 'dedicated',
        syncMode: 'full',
        plugins: [],
        provider: 'hetzner',
      }),
    ).toBe(79);
  });

  it('prices DigitalOcean dedicated nodes as the fallback profile', () => {
    expect(
      BillingService.calculateProjectedCost({
        type: 'dedicated',
        syncMode: 'full',
        plugins: [],
        provider: 'digitalocean',
      }),
    ).toBe(99);
  });

  it('adds archive and plugin costs on top of the provider base', () => {
    expect(
      BillingService.calculateProjectedCost({
        type: 'dedicated',
        syncMode: 'archive',
        plugins: ['aa-bundler', 'tee-oracle'],
        provider: 'hetzner',
      }),
    ).toBe(277);
  });
});
