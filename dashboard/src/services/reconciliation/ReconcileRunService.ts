import type { Prisma } from '@prisma/client';
import { prisma } from '../../utils/prisma';

type ReconcileRunRow = {
  id: number;
  source: string;
  status: string;
  provisioningChecked: number;
  provisioningResumed: number;
  alertingEndpointsEvaluated: number;
  metadata: unknown;
  errorMessage: string | null;
  createdAt: Date;
};

export function toReconcileRunView(run: ReconcileRunRow) {
  return {
    ...run,
    createdAt: run.createdAt.toISOString(),
  };
}

export async function recordReconcileRun(input: {
  source: 'manual' | 'scheduled';
  status: 'success' | 'error';
  provisioningChecked: number;
  provisioningResumed: number;
  alertingEndpointsEvaluated: number;
  metadata?: Prisma.InputJsonValue;
  errorMessage?: string | null;
}) {
  return prisma.reconcileRun.create({
    data: {
      source: input.source,
      status: input.status,
      provisioningChecked: input.provisioningChecked,
      provisioningResumed: input.provisioningResumed,
      alertingEndpointsEvaluated: input.alertingEndpointsEvaluated,
      metadata: input.metadata,
      errorMessage: input.errorMessage ?? null,
    },
  });
}
