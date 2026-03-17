import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { synchronizeEndpointStatus } from '@/services/endpoints/EndpointStatusService';
import { getCurrentUserContext } from '@/server/organization';

export async function GET() {
  const userContext = await getCurrentUserContext();

  if (!userContext?.organizationId) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  try {
    const endpoints = await prisma.endpoint.findMany({
      where: { organizationId: userContext.organizationId },
      orderBy: { createdAt: 'desc' },
    });

    const statuses = await Promise.all(
      endpoints.map((endpoint) =>
        synchronizeEndpointStatus({
          id: endpoint.id,
          url: endpoint.url,
          status: endpoint.status,
          clientEngine: endpoint.clientEngine,
          providerPublicIp: endpoint.providerPublicIp,
        }),
      ),
    );

    return NextResponse.json(
      endpoints.map((endpoint, index) => ({
        ...endpoint,
        status: statuses[index],
      })),
    );
  } catch (error) {
    console.error('Error fetching endpoints:', error);
    return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
  }
}
