import { prisma } from '@/utils/prisma';
import EndpointsList, { Endpoint } from './EndpointsList';
import { synchronizeEndpointStatus } from '@/services/endpoints/EndpointStatusService';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function EndpointsPage() {
  let endpoints: Endpoint[] = [];

  try {
    const userContext = await getCurrentUserContext();

    if (process.env.DATABASE_URL && userContext?.organizationId) {
      const data = await prisma.endpoint.findMany({
        where: { organizationId: userContext.organizationId },
        orderBy: { createdAt: 'desc' },
      });

      if (data && data.length > 0) {
        const statuses = await Promise.all(
          data.map((endpoint) =>
          synchronizeEndpointStatus({
            id: endpoint.id,
            url: endpoint.url,
            status: endpoint.status,
            clientEngine: endpoint.clientEngine,
            providerPublicIp: endpoint.providerPublicIp,
          }),
        ),
      );

        // Map Prisma Endpoint to our component Endpoint type
        endpoints = data.map((ep, index) => ({
          id: ep.id,
          name: ep.name,
          protocol: ep.protocol,
          networkKey: ep.networkKey,
          network: ep.network,
          type: ep.type,
          url: ep.url,
          wssUrl: ep.wssUrl,
          providerPublicIp: ep.providerPublicIp,
          status: statuses[index],
          requests: ep.requests.toString(),
          clientEngine: ep.clientEngine,
          syncMode: ep.syncMode,
          cloudProvider: ep.cloudProvider,
          region: ep.region,
        }));
      }
    } else if (!process.env.DATABASE_URL) {
      console.warn('DATABASE_URL is not set. Cannot fetch endpoints.');
    }
  } catch (error) {
    console.error('Database connection failed:', error);
  }

  return <EndpointsList endpoints={endpoints} />;
}
