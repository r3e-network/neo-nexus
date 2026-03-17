import { prisma } from '@/utils/prisma';
import EndpointDetailsClient from './EndpointDetailsClient';
import { Endpoint } from '../EndpointsList';
import { notFound } from 'next/navigation';
import { synchronizeEndpointStatus } from '@/services/endpoints/EndpointStatusService';
import {
  buildDefaultNodeSettings,
  mergeNodeSettings,
} from '@/services/settings/NodeSettings';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function EndpointDetailsPage({ params }: { params: { id: string } }) {
  const { id } = await params;
  let endpoint: Endpoint | null = null;
  let provisioningOrderId: number | null = null;
  let nodeSettings = buildDefaultNodeSettings('neo-go');
  let nodePlugins: Array<{
    id: number;
    pluginId: string;
    status: string;
    errorMessage: string | null;
    lastAppliedAt: string | null;
    createdAt: string;
  }> = [];

  try {
    const userContext = await getCurrentUserContext();

    if (process.env.DATABASE_URL && userContext?.organizationId) {
      const data = await prisma.endpoint.findFirst({
        where: {
          id: parseInt(id, 10),
          organizationId: userContext.organizationId,
        },
      });

      if (data) {
        const status = await synchronizeEndpointStatus({
          id: data.id,
          url: data.url,
          status: data.status,
          clientEngine: data.clientEngine,
          providerPublicIp: data.providerPublicIp,
        });

        const latestOrder = await prisma.provisioningOrder.findFirst({
          where: { endpointId: data.id },
          orderBy: { createdAt: 'desc' },
          select: { id: true },
        });

        const pluginRows = await prisma.nodePlugin.findMany({
          where: { endpointId: data.id },
          orderBy: { createdAt: 'asc' },
          select: {
            id: true,
            pluginId: true,
            status: true,
            errorMessage: true,
            lastAppliedAt: true,
            createdAt: true,
          },
        });

        nodePlugins = pluginRows.map((plugin) => ({
          ...plugin,
          lastAppliedAt: plugin.lastAppliedAt?.toISOString() ?? null,
          createdAt: plugin.createdAt.toISOString(),
        }));

        provisioningOrderId = latestOrder?.id ?? null;
        nodeSettings = mergeNodeSettings(data.clientEngine, data.settings);

        endpoint = {
          id: data.id,
          name: data.name,
          protocol: data.protocol,
          networkKey: data.networkKey,
          network: data.network,
          type: data.type,
          url: data.url,
          wssUrl: data.wssUrl,
          providerPublicIp: data.providerPublicIp,
          status: status,
          requests: data.requests.toString(),
          clientEngine: data.clientEngine,
          syncMode: data.syncMode,
          cloudProvider: data.cloudProvider,
          region: data.region,
        };
      }
    } else if (!process.env.DATABASE_URL) {
      console.warn('DATABASE_URL is not set.');
    }
  } catch (error) {
    console.error('Database connection failed:', error);
  }

  if (!endpoint) {
    notFound();
  }

  return <EndpointDetailsClient endpoint={endpoint} provisioningOrderId={provisioningOrderId} nodePlugins={nodePlugins} nodeSettings={nodeSettings} />;
}
