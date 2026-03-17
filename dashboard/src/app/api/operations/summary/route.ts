import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { buildOperationsSummary } from '@/services/reconciliation/OperationsSummaryService';
import { getCurrentUserContext } from '@/server/organization';

export async function GET() {
  const userContext = await getCurrentUserContext();

  if (!userContext) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }
  if (userContext.role !== 'operator') {
    return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
  }

  const [pendingOrders, alertRetries, openIncidentsCount, latestSuccess, latestFailure, latestScheduledSuccess, latestScheduledFailure, latestScheduledRun] = await Promise.all([
    prisma.provisioningOrder.findMany({
      where: {
        status: { notIn: ['ready', 'failed'] },
      },
      orderBy: [{ nextAttemptAt: 'asc' }, { updatedAt: 'asc' }],
      take: 10,
      select: {
        id: true,
        endpointId: true,
        currentStep: true,
        attemptCount: true,
        nextAttemptAt: true,
        errorMessage: true,
        endpoint: {
          select: {
            name: true,
          },
        },
      },
    }),
    prisma.alertRule.findMany({
      where: {
        isActive: true,
        nextDeliveryAt: { not: null },
      },
      orderBy: { nextDeliveryAt: 'asc' },
      take: 10,
      select: {
        id: true,
        name: true,
        endpointId: true,
        nextDeliveryAt: true,
        deliveryAttemptCount: true,
        lastDeliveryError: true,
        endpoint: {
          select: {
            name: true,
          },
        },
      },
    }),
    prisma.alertIncident.count({
      where: {
        status: 'Open',
      },
    }),
    prisma.reconcileRun.findFirst({
      where: { status: 'success' },
      orderBy: { createdAt: 'desc' },
      select: { createdAt: true },
    }),
    prisma.reconcileRun.findFirst({
      where: { status: 'error' },
      orderBy: { createdAt: 'desc' },
      select: { createdAt: true },
    }),
    prisma.reconcileRun.findFirst({
      where: { status: 'success', source: 'scheduled' },
      orderBy: { createdAt: 'desc' },
      select: { createdAt: true },
    }),
    prisma.reconcileRun.findFirst({
      where: { status: 'error', source: 'scheduled' },
      orderBy: { createdAt: 'desc' },
      select: { createdAt: true },
    }),
    prisma.reconcileRun.findFirst({
      where: { source: 'scheduled' },
      orderBy: { createdAt: 'desc' },
      select: { createdAt: true, status: true },
    }),
  ]);

  const summary = buildOperationsSummary({
    pendingOrders: pendingOrders.map((order) => ({
      nextAttemptAt: order.nextAttemptAt?.toISOString() ?? null,
    })),
    alertRetries: alertRetries.map((rule) => ({
      nextDeliveryAt: rule.nextDeliveryAt?.toISOString() ?? null,
    })),
    openIncidentsCount,
    latestSuccessfulRunAt: latestSuccess?.createdAt.toISOString() ?? null,
    latestFailedRunAt: latestFailure?.createdAt.toISOString() ?? null,
    latestScheduledSuccessAt: latestScheduledSuccess?.createdAt.toISOString() ?? null,
    latestScheduledFailureAt: latestScheduledFailure?.createdAt.toISOString() ?? null,
    latestScheduledRunAt: latestScheduledRun?.createdAt.toISOString() ?? null,
    latestScheduledRunStatus: latestScheduledRun?.status === 'success' || latestScheduledRun?.status === 'error'
      ? latestScheduledRun.status
      : null,
    now: new Date(),
  });

  return NextResponse.json({
    summary,
    pendingOrders: pendingOrders.map((order) => ({
      id: order.id,
      endpointId: order.endpointId,
      endpointName: order.endpoint.name,
      currentStep: order.currentStep,
      attemptCount: order.attemptCount,
      nextAttemptAt: order.nextAttemptAt?.toISOString() ?? null,
      errorMessage: order.errorMessage,
    })),
    alertRetries: alertRetries.map((rule) => ({
      id: rule.id,
      endpointId: rule.endpointId,
      endpointName: rule.endpoint.name,
      name: rule.name,
      nextDeliveryAt: rule.nextDeliveryAt?.toISOString() ?? null,
      deliveryAttemptCount: rule.deliveryAttemptCount,
      lastDeliveryError: rule.lastDeliveryError,
    })),
  });
}
