import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import {
  buildAnalyticsSnapshot,
  EMPTY_ANALYTICS_SNAPSHOT,
} from '@/services/analytics/AnalyticsService';
import { getCurrentUserContext, isDatabaseConfigured } from '@/server/organization';

export async function GET() {
  const userContext = await getCurrentUserContext();

  if (!userContext?.organizationId) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  if (!isDatabaseConfigured()) {
    return NextResponse.json(EMPTY_ANALYTICS_SNAPSHOT);
  }

  const [endpoints, apiKeys, plugins] = await Promise.all([
    prisma.endpoint.findMany({
      where: { organizationId: userContext.organizationId },
      select: {
        id: true,
        name: true,
        network: true,
        status: true,
        requests: true,
        createdAt: true,
      },
    }),
    prisma.apiKey.findMany({
      where: { organizationId: userContext.organizationId },
      select: {
        id: true,
        name: true,
        createdAt: true,
      },
    }),
    prisma.nodePlugin.findMany({
      where: {
        endpoint: {
          organizationId: userContext.organizationId,
        },
      },
      select: {
        id: true,
        endpointId: true,
        pluginId: true,
        status: true,
        createdAt: true,
      },
    }),
  ]);

  return NextResponse.json(
    buildAnalyticsSnapshot({
      endpoints,
      apiKeys,
      plugins,
    }),
  );
}
