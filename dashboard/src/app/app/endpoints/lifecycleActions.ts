'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { ApisixService } from '@/services/apisix/ApisixService';
import { recordEndpointActivity } from '@/services/endpoints/EndpointActivityService';
import { performProviderLifecycleAction } from '@/services/provisioning/VmLifecycleService';
import { getErrorMessage } from '@/server/errors';
import { requireCurrentOrganizationContext } from '@/server/organization';

async function requireOwnedEndpoint(endpointId: number) {
  const { organizationId } = await requireCurrentOrganizationContext();

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId,
    },
  });

  if (!endpoint) {
    throw new Error('Endpoint not found or permission denied.');
  }

  return endpoint;
}

export async function restartEndpointAction(endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    if (!endpoint.cloudProvider || !endpoint.providerServerId) {
      return { success: false, error: 'This endpoint does not support restart actions.' };
    }

    await performProviderLifecycleAction({
      provider: endpoint.cloudProvider as 'hetzner' | 'digitalocean',
      providerServerId: endpoint.providerServerId,
      action: 'restart',
    });

    await prisma.endpoint.update({
      where: { id: endpoint.id },
      data: { status: 'Syncing' },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'lifecycle',
      action: 'restart',
      status: 'success',
      message: 'Restart requested for the dedicated endpoint.',
    });

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    revalidatePath('/app/endpoints');
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function stopEndpointAction(endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    if (!endpoint.cloudProvider || !endpoint.providerServerId) {
      return { success: false, error: 'This endpoint does not support stop actions.' };
    }

    await performProviderLifecycleAction({
      provider: endpoint.cloudProvider as 'hetzner' | 'digitalocean',
      providerServerId: endpoint.providerServerId,
      action: 'stop',
    });

    await prisma.endpoint.update({
      where: { id: endpoint.id },
      data: { status: 'Stopped' },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'lifecycle',
      action: 'stop',
      status: 'success',
      message: 'Stop requested for the dedicated endpoint.',
    });

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    revalidatePath('/app/endpoints');
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function destroyEndpointAction(endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);

    if (endpoint.cloudProvider && endpoint.providerServerId) {
      await performProviderLifecycleAction({
        provider: endpoint.cloudProvider as 'hetzner' | 'digitalocean',
        providerServerId: endpoint.providerServerId,
        action: 'destroy',
      });
    }

    await ApisixService.deleteRoute(String(endpoint.id));
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'lifecycle',
      action: 'destroy',
      status: 'success',
      message: 'Dedicated endpoint destroyed and routing removed.',
    });
    await prisma.endpoint.delete({
      where: { id: endpoint.id },
    });

    revalidatePath('/app/endpoints');
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
