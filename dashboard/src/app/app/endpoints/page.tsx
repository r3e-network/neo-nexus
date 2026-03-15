import { prisma } from '@/utils/prisma';
import EndpointsList, { Endpoint } from './EndpointsList';
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
        // Map Prisma Endpoint to our component Endpoint type
        endpoints = data.map(ep => ({
          id: ep.id,
          name: ep.name,
          network: ep.network,
          type: ep.type,
          url: ep.url,
          status: ep.status,
          requests: ep.requests.toString(),
          clientEngine: ep.clientEngine
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
