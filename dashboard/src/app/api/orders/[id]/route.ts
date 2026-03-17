import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { synchronizeEndpointStatus } from '@/services/endpoints/EndpointStatusService';
import { kickoffProvisioningOrder } from '@/services/provisioning/ProvisioningRunner';
import {
  deriveProvisioningOrderStatus,
  type ProvisioningOrderStatus,
} from '@/services/provisioning/ProvisioningOrderService';
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
  const orderId = Number.parseInt(id, 10);
  if (!Number.isInteger(orderId)) {
    return NextResponse.json({ error: 'Invalid order id' }, { status: 400 });
  }

  const order = await prisma.provisioningOrder.findFirst({
    where: {
      id: orderId,
      organizationId: userContext.organizationId,
    },
    include: {
      endpoint: {
        select: {
          id: true,
          url: true,
          status: true,
          clientEngine: true,
          cloudProvider: true,
          providerPublicIp: true,
          region: true,
        },
      },
    },
  });

  if (!order) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  if (order.status !== 'ready' && order.status !== 'failed') {
    await kickoffProvisioningOrder(order.id);
  }

  const endpointStatus = await synchronizeEndpointStatus({
    id: order.endpoint.id,
    url: order.endpoint.url,
    status: order.endpoint.status,
    clientEngine: order.endpoint.clientEngine,
    providerPublicIp: order.endpoint.providerPublicIp,
  });

  const derived = deriveProvisioningOrderStatus({
    orderStatus: order.status as ProvisioningOrderStatus,
    endpointStatus,
  });

  if (derived.status !== order.status || derived.currentStep !== order.currentStep) {
    await prisma.provisioningOrder.update({
      where: { id: order.id },
      data: {
        status: derived.status,
        currentStep: derived.currentStep,
      },
    });
  }

  return NextResponse.json({
    id: order.id,
    status: derived.status,
    currentStep: derived.currentStep,
    provider: order.provider,
    attemptCount: order.attemptCount,
    lastAttemptAt: order.lastAttemptAt?.toISOString() ?? null,
    nextAttemptAt: order.nextAttemptAt?.toISOString() ?? null,
    errorMessage: order.errorMessage,
    endpointId: order.endpointId,
    endpointStatus,
    region: order.endpoint.region,
    cloudProvider: order.endpoint.cloudProvider,
    updatedAt: order.updatedAt,
  });
}
