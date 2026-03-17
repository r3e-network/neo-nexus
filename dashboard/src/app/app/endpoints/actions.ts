'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { getErrorMessage } from '@/server/errors';
import { resolveInfrastructureSelection } from '@/services/infrastructure/ProviderCatalog';
import { buildPlannedEndpointAddress } from '@/services/endpoints/EndpointAddressing';
import { getSharedBackendTarget } from '@/services/endpoints/SharedEndpointConfig';
import { kickoffProvisioningOrder } from '@/services/provisioning/ProvisioningRunner';
import {
  assertDatabaseConfigured,
  requireCurrentOrganizationContext,
} from '@/server/organization';

export async function createEndpointAction(formData: {
  name: string;
  protocol: string;
  network: string;
  type: string;
  clientEngine: string;
  provider: string;
  region: string;
  syncMode: string;
}) {
  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  let orgId: string;
  let billingPlan: string;

  try {
    const context = await requireCurrentOrganizationContext();
    orgId = context.organizationId;
    billingPlan = context.billingPlan;
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  if (formData.type === 'dedicated' && billingPlan === 'developer') {
    return { success: false, error: 'Billing Plan Error: You must upgrade to Growth or Dedicated plan to deploy Dedicated Nodes.' };
  }

  // Enforce Endpoint Limits based on Billing Plan
  const currentSharedCount = await prisma.endpoint.count({
    where: { 
      organizationId: orgId, 
      type: 'Shared' 
    },
  });

  if (formData.type === 'shared') {
    if (billingPlan === 'developer' && currentSharedCount >= 1) {
      return { success: false, error: 'Plan Limit Reached: Developer plan is limited to 1 Shared Endpoint. Please upgrade your plan.' };
    }
    if (billingPlan === 'growth' && currentSharedCount >= 3) {
      return { success: false, error: 'Plan Limit Reached: Growth plan is limited to 3 Shared Endpoints.' };
    }
  }

  const infrastructureSelection = resolveInfrastructureSelection(formData.provider, formData.region);

  if (formData.type === 'shared') {
    try {
      getSharedBackendTarget(formData.protocol, formData.network);
    } catch (error) {
      return { success: false, error: getErrorMessage(error) };
    }
  }

  try {
    let networkString = formData.protocol === 'neo-x' 
        ? (formData.network === 'mainnet' ? 'Neo X Mainnet' : 'Neo X Testnet')
        : (formData.network === 'mainnet' ? 'N3 Mainnet' : 'N3 Testnet');
    
    if (formData.network === 'private') {
        networkString = formData.protocol === 'neo-x' ? 'Neo X Private Net' : 'Neo N3 Private Net';
    }

    const providerLabel = formData.type === 'dedicated'
      ? infrastructureSelection.provider
      : 'shared';
    const { endpoint, order } = await prisma.$transaction(async (tx) => {
      const createdEndpoint = await tx.endpoint.create({
        data: {
          name: formData.name,
          protocol: formData.protocol,
          networkKey: formData.network,
          network: networkString,
          type: formData.type.charAt(0).toUpperCase() + formData.type.slice(1),
          clientEngine: formData.clientEngine,
          syncMode: formData.syncMode,
          cloudProvider: formData.type === 'dedicated' ? infrastructureSelection.provider : null,
          providerServerId: null,
          region: formData.type === 'dedicated' ? infrastructureSelection.region : null,
          url: 'pending://route-assignment',
          wssUrl: null,
          providerPublicIp: null,
          status: formData.type === 'dedicated' ? 'Provisioning' : 'Syncing',
          requests: 0,
          organizationId: orgId,
        },
      });

      const plannedAddress = buildPlannedEndpointAddress({
        type: formData.type as 'shared' | 'dedicated',
        protocol: formData.protocol as 'neo-n3' | 'neo-x',
        networkKey: formData.network as 'mainnet' | 'testnet' | 'private',
        region: formData.type === 'dedicated' ? infrastructureSelection.region : null,
        routeKey: String(createdEndpoint.id),
      });

      const endpoint = await tx.endpoint.update({
        where: { id: createdEndpoint.id },
        data: {
          url: plannedAddress.httpsUrl,
          wssUrl: plannedAddress.wssUrl,
        },
      });

      const order = await tx.provisioningOrder.create({
        data: {
          organizationId: orgId,
          endpointId: endpoint.id,
          provider: providerLabel,
          status: 'pending',
          currentStep: 'pending',
        },
      });

      return { endpoint, order };
    });

    await kickoffProvisioningOrder(order.id);

    revalidatePath('/app/endpoints');
    return { success: true, id: endpoint.id, orderId: order.id };
  } catch (error) {
    console.error('Error creating endpoint:', error);
    return { success: false, error: getErrorMessage(error) };
  }
}
