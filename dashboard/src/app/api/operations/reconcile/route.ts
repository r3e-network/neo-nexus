import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { toReconcileRunView } from '@/services/reconciliation/ReconcileRunService';
import { clampReconcileRunsLimit } from '@/services/reconciliation/ReconcileRunWindow';

export async function GET(request: Request) {
  const userContext = await getCurrentUserContext();

  if (!userContext) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }
  if (userContext.role !== 'operator') {
    return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
  }

  const { searchParams } = new URL(request.url);
  const limit = clampReconcileRunsLimit(searchParams.get('limit') ?? undefined);

  const runs = await prisma.reconcileRun.findMany({
    orderBy: { createdAt: 'desc' },
    take: limit,
  });

  return NextResponse.json(runs.map(toReconcileRunView));
}
