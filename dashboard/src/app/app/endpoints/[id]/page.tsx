import { prisma } from '@/utils/prisma';
import EndpointDetailsClient from './EndpointDetailsClient';
import { Endpoint } from '../EndpointsList';
import { notFound } from 'next/navigation';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function EndpointDetailsPage({ params }: { params: { id: string } }) {
  const { id } = await params;
  let endpoint: Endpoint | null = null;

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
        endpoint = {
          id: data.id,
          name: data.name,
          network: data.network,
          type: data.type,
          url: data.url,
          status: data.status,
          requests: data.requests.toString(),
          clientEngine: data.clientEngine
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

  return <EndpointDetailsClient endpoint={endpoint} />;
}
