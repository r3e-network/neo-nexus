import { NextResponse } from 'next/server';
import { executeAndRecordReconcile } from '@/services/reconciliation/ExecuteReconcile';

function isAuthorized(request: Request) {
  const secret = process.env.INTERNAL_RECONCILE_SECRET;
  if (!secret) {
    return false;
  }

  return request.headers.get('x-internal-reconcile-secret') === secret;
}

export async function POST(request: Request) {
  if (!isAuthorized(request)) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const execution = await executeAndRecordReconcile('scheduled');
  if (execution.ok) {
    return NextResponse.json({
      ...execution.result,
      runId: execution.runId,
    });
  }

  return NextResponse.json({
    error: execution.error,
    runId: execution.runId,
  }, { status: 500 });
}
