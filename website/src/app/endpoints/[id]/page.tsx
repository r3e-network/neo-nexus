import { prisma } from '@/utils/prisma';
import EndpointDetailsClient from './EndpointDetailsClient';
import { Endpoint } from '../EndpointsList';
import { notFound } from 'next/navigation';

export default async function EndpointDetailsPage({ params }: { params: { id: string } }) {
  const { id } = await params;
  let endpoint: Endpoint | null = null;

  try {
    if (process.env.DATABASE_URL) {
      const data = await prisma.endpoint.findUnique({
        where: { id: parseInt(id, 10) }
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
    } else {
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
