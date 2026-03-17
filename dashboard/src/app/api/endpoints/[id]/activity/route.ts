import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { toEndpointActivityView } from '@/services/endpoints/EndpointActivityService';
import { getCurrentUserContext } from '@/server/organization';

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();

  if (!userContext?.organizationId) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const { id } = await params;
  const endpointId = Number.parseInt(id, 10);
  if (!Number.isInteger(endpointId)) {
    return NextResponse.json({ error: 'Invalid endpoint id' }, { status: 400 });
  }

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId: userContext.organizationId,
    },
    select: { id: true },
  });

  if (!endpoint) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  const activities = await prisma.endpointActivity.findMany({
    where: { endpointId: endpoint.id },
    orderBy: { createdAt: 'desc' },
    take: 50,
  });

  return NextResponse.json(activities.map(toEndpointActivityView));
}
