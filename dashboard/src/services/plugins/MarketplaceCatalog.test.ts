import { describe, expect, it } from 'vitest';
import { listSupportedPlugins } from './PluginCatalog';
import { buildMarketplacePluginCards } from './MarketplaceCatalog';

describe('MarketplaceCatalog', () => {
  it('requires a dedicated node when the organization has none', () => {
    const cards = buildMarketplacePluginCards({
      plugins: listSupportedPlugins(),
      dedicatedEndpoints: [],
    });

    expect(cards[0].actionHref).toBe('/app/endpoints/new');
    expect(cards[0].actionLabel).toBe('Create Dedicated Node');
    expect(cards[0].statusLabel).toBe('Requires a dedicated node');
  });

  it('routes a single dedicated endpoint directly to that node', () => {
    const cards = buildMarketplacePluginCards({
      plugins: listSupportedPlugins(),
      dedicatedEndpoints: [
        {
          id: 7,
          name: 'alpha',
          installedPluginIds: ['tee-oracle'],
        },
      ],
    });

    const oracleCard = cards.find((card) => card.id === 'tee-oracle');
    const bundlerCard = cards.find((card) => card.id === 'aa-bundler');

    expect(oracleCard?.installedOnCount).toBe(1);
    expect(oracleCard?.statusLabel).toBe('Installed on 1 node');
    expect(oracleCard?.actionHref).toBe('/app/endpoints/7');
    expect(oracleCard?.actionLabel).toBe('Manage on Node');

    expect(bundlerCard?.statusLabel).toBe('Ready for dedicated nodes');
    expect(bundlerCard?.actionHref).toBe('/app/endpoints/7');
    expect(bundlerCard?.actionLabel).toBe('Configure on Node');
  });

  it('routes multi-node organizations to the endpoints list', () => {
    const cards = buildMarketplacePluginCards({
      plugins: listSupportedPlugins(),
      dedicatedEndpoints: [
        {
          id: 7,
          name: 'alpha',
          installedPluginIds: ['tee-oracle'],
        },
        {
          id: 8,
          name: 'beta',
          installedPluginIds: ['tee-oracle', 'aa-bundler'],
        },
      ],
    });

    const oracleCard = cards.find((card) => card.id === 'tee-oracle');
    const mempoolCard = cards.find((card) => card.id === 'tee-mempool');

    expect(oracleCard?.installedOnCount).toBe(2);
    expect(oracleCard?.statusLabel).toBe('Installed on 2 nodes');
    expect(oracleCard?.actionHref).toBe('/app/endpoints');
    expect(oracleCard?.actionLabel).toBe('Manage on Nodes');

    expect(mempoolCard?.actionHref).toBe('/app/endpoints');
    expect(mempoolCard?.actionLabel).toBe('Choose Node');
  });
});
