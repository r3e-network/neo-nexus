import type { PluginDefinition } from './PluginCatalog';

type DedicatedEndpointLike = {
  id: number;
  name: string;
  installedPluginIds: string[];
};

export type MarketplacePluginCard = {
  id: string;
  name: string;
  description: string;
  badge: string;
  category: string;
  requiresPrivateKey: boolean;
  defaultImage: string;
  installedOnCount: number;
  statusLabel: string;
  actionLabel: string;
  actionHref: string;
};

function getActionTarget(dedicatedEndpoints: DedicatedEndpointLike[], installedOnCount: number) {
  if (dedicatedEndpoints.length === 0) {
    return {
      actionLabel: 'Create Dedicated Node',
      actionHref: '/app/endpoints/new',
      statusLabel: 'Requires a dedicated node',
    };
  }

  if (dedicatedEndpoints.length === 1) {
    return {
      actionLabel: installedOnCount > 0 ? 'Manage on Node' : 'Configure on Node',
      actionHref: `/app/endpoints/${dedicatedEndpoints[0].id}`,
      statusLabel: installedOnCount > 0 ? `Installed on ${installedOnCount} node` : 'Ready for dedicated nodes',
    };
  }

  return {
    actionLabel: installedOnCount > 0 ? 'Manage on Nodes' : 'Choose Node',
    actionHref: '/app/endpoints',
    statusLabel: installedOnCount > 0 ? `Installed on ${installedOnCount} nodes` : 'Ready for dedicated nodes',
  };
}

export function buildMarketplacePluginCards(input: {
  plugins: PluginDefinition[];
  dedicatedEndpoints: DedicatedEndpointLike[];
}): MarketplacePluginCard[] {
  return input.plugins.map((plugin) => {
    const installedOnCount = input.dedicatedEndpoints.filter((endpoint) => (
      endpoint.installedPluginIds.includes(plugin.id)
    )).length;
    const target = getActionTarget(input.dedicatedEndpoints, installedOnCount);

    return {
      id: plugin.id,
      name: plugin.name,
      description: plugin.description,
      badge: plugin.badge,
      category: plugin.category,
      requiresPrivateKey: plugin.requiresPrivateKey,
      defaultImage: plugin.defaultImage,
      installedOnCount,
      statusLabel: target.statusLabel,
      actionLabel: target.actionLabel,
      actionHref: target.actionHref,
    };
  });
}
