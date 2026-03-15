import { prisma } from '@/utils/prisma';
import EndpointsList, { Endpoint } from './EndpointsList';

export default async function EndpointsPage() {
  let endpoints: Endpoint[] = [];

  try {
    if (process.env.DATABASE_URL) {
      const data = await prisma.endpoint.findMany({
        orderBy: { createdAt: 'desc' }
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
    } else {
        console.warn('DATABASE_URL is not set. Cannot fetch endpoints.');
    }
  } catch (error) {
    console.error('Database connection failed:', error);
  }

  return <EndpointsList endpoints={endpoints} />;
}
