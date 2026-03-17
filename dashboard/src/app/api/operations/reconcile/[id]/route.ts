import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { toReconcileRunView } from '@/services/reconciliation/ReconcileRunService';

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();

  if (!userContext) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }
  if (userContext.role !== 'operator') {
    return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
  }

  const { id } = await params;
  const runId = Number.parseInt(id, 10);
  if (!Number.isInteger(runId)) {
    return NextResponse.json({ error: 'Invalid reconcile run id' }, { status: 400 });
  }

  const run = await prisma.reconcileRun.findUnique({
    where: { id: runId },
  });

  if (!run) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  return NextResponse.json(toReconcileRunView(run));
}
