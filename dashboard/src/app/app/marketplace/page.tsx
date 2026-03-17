import MarketplaceClient from './MarketplaceClient';
import { isDatabaseConfigured, getCurrentUserContext } from '@/server/organization';
import { prisma } from '@/utils/prisma';
import { listSupportedPlugins } from '@/services/plugins/PluginCatalog';
import { buildMarketplacePluginCards } from '@/services/plugins/MarketplaceCatalog';

export const dynamic = 'force-dynamic';

export default async function MarketplacePage() {
  const userContext = await getCurrentUserContext();
  const plugins = listSupportedPlugins();
  let dedicatedEndpoints: Array<{
    id: number;
    name: string;
    installedPluginIds: string[];
  }> = [];

  if (isDatabaseConfigured() && userContext?.organizationId) {
    const rows = await prisma.endpoint.findMany({
      where: {
        organizationId: userContext.organizationId,
        type: 'Dedicated',
      },
      orderBy: { createdAt: 'desc' },
      select: {
        id: true,
        name: true,
        plugins: {
          select: {
            pluginId: true,
          },
        },
      },
    });

    dedicatedEndpoints = rows.map((row) => ({
      id: row.id,
      name: row.name,
      installedPluginIds: row.plugins.map((plugin) => plugin.pluginId),
    }));
  }

  const cards = buildMarketplacePluginCards({
    plugins,
    dedicatedEndpoints,
  });

  return (
    <MarketplaceClient
      cards={cards}
      dedicatedEndpointCount={dedicatedEndpoints.length}
    />
  );
}
