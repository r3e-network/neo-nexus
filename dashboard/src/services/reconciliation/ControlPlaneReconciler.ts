import { prisma } from '../../utils/prisma';
import { evaluateEndpointAlerts } from '../alerts/AlertEvaluationService';
import {
  loadPendingProvisioningOrderIds,
  resumePendingProvisioningOrders,
} from '../provisioning/ProvisioningReconciler';

type ReconcileDependencies = {
  loadPendingOrderIds?: (limit?: number) => Promise<number[]>;
  resumeOrders?: (orderIds: number[]) => Promise<{ checked: number; resumed: number; resumedOrderIds: number[] }>;
  loadAlertEndpointIds?: (limit?: number) => Promise<number[]>;
  evaluateEndpointAlerts?: (endpointId: number) => Promise<unknown>;
};

export async function reconcileControlPlane(
  dependencies: ReconcileDependencies = {},
) {
  const loadPendingOrderIds = dependencies.loadPendingOrderIds ?? loadPendingProvisioningOrderIds;
  const resumeOrders = dependencies.resumeOrders ?? resumePendingProvisioningOrders;
  const loadAlertEndpointIds = dependencies.loadAlertEndpointIds ?? (async (limit = 50) => {
    const endpoints = await prisma.endpoint.findMany({
      where: {
        alertRules: {
          some: {
            isActive: true,
          },
        },
      },
      take: limit,
      orderBy: { id: 'asc' },
      select: { id: true },
    });

    return endpoints.map((endpoint) => endpoint.id);
  });
  const evaluateAlerts = dependencies.evaluateEndpointAlerts ?? evaluateEndpointAlerts;

  const pendingOrderIds = await loadPendingOrderIds();
  const provisioning = await resumeOrders(pendingOrderIds);

  const alertEndpointIds = await loadAlertEndpointIds();
  for (const endpointId of alertEndpointIds) {
    await evaluateAlerts(endpointId);
  }

  return {
    provisioning,
    alerting: {
      endpointsEvaluated: alertEndpointIds.length,
      endpointIds: alertEndpointIds,
    },
    reconciledAt: new Date().toISOString(),
  };
}
