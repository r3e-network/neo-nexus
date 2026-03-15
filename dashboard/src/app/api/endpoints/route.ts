import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
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
    return NextResponse.json(endpoints);
  } catch (error) {
    console.error('Error fetching endpoints:', error);
    return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
  }
}
