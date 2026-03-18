'use server';

import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { getErrorMessage } from '@/server/errors';
import { recordEndpointActivity } from '@/services/endpoints/EndpointActivityService';
import { buildRemoteResyncCommand, buildRemoteSnapshotCommand } from '@/services/maintenance/RemoteMaintenance';
import { buildRemoteNodeSettingsSyncCommand } from '@/services/settings/RemoteNodeSettings';
import { buildPlannedEndpointAddress } from '@/services/endpoints/EndpointAddressing';
import { ApisixService } from '@/services/apisix/ApisixService';
import { loadOrganizationRoutePlugins } from '@/services/security/RouteSecuritySync';

export async function updateEndpointDomainAction(
  endpointId: number,
  customDomain: string | null,
) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    
    // Only allow custom domains for dedicated nodes
    if (endpoint.type.toLowerCase() === 'shared') {
      return { success: false, error: 'Custom domains are only supported for dedicated nodes.' };
    }

    const domainToSave = customDomain && customDomain.trim() !== '' ? customDomain.trim() : null;

    const plannedAddress = buildPlannedEndpointAddress({
      type: 'dedicated',
      protocol: endpoint.protocol as 'neo-n3' | 'neo-x',
      networkKey: endpoint.networkKey as 'mainnet' | 'testnet' | 'private',
      region: endpoint.region,
      routeKey: String(endpoint.id),
      customDomain: domainToSave,
    });

    await prisma.endpoint.update({
      where: { id: endpoint.id },
      data: {
        customDomain: domainToSave,
        url: plannedAddress.httpsUrl,
        wssUrl: plannedAddress.wssUrl,
      },
    });

    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'settings',
      action: 'domain_updated',
      status: 'success',
      message: domainToSave 
        ? `Custom domain updated to ${domainToSave}`
        : 'Reverted to auto-generated domain',
      metadata: { customDomain: domainToSave },
    });

    // Fire the async apisix route sync to rebuild the URL rules
    try {
      const routePlugins = await loadOrganizationRoutePlugins(endpoint.organizationId || '');
      await ApisixService.createRoute(
        String(endpoint.id),
        plannedAddress.httpsUrl,
        endpoint.providerPublicIp || '127.0.0.1', // Will be 127.0.0.1 if it's pending provision, which is safe to override later
        10332, // Default RPC port for now, the real provisioning engine overwrites this based on engine anyway
        routePlugins
      );

      // If a vanity domain is configured, ensure APISIX terminates SSL for that SNI
      if (domainToSave) {
        await ApisixService.syncSslCertificate(domainToSave);
      }
    } catch (e) {
      console.warn('Failed to sync domain change to apisix right away', e);
    }

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
import { mergeNodeSettings } from '@/services/settings/NodeSettings';
import {
  REMOTE_NEO_GO_CONFIG_PATH,
  REMOTE_NODE_SETTINGS_PATH,
  REMOTE_NODE_SYNC_PATH,
  renderNodeRuntimeArtifacts,
} from '@/services/settings/RenderedNodeRuntime';
import { requireCurrentOrganizationContext } from '@/server/organization';

const execFileAsync = promisify(execFile);

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

export async function updateEndpointSettingsAction(
  endpointId: number,
  rawSettings: unknown,
) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    const settings = mergeNodeSettings(endpoint.clientEngine, rawSettings);
    const runtimeArtifacts = renderNodeRuntimeArtifacts({
      clientEngine: endpoint.clientEngine as 'neo-go' | 'neo-cli' | 'neo-x-geth',
      network: endpoint.networkKey === 'testnet'
        ? 'testnet'
        : endpoint.networkKey === 'private'
          ? 'private'
          : 'mainnet',
      settings,
    });

    await prisma.endpoint.update({
      where: { id: endpoint.id },
      data: {
        settings,
        status: 'Syncing',
      },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'settings',
      action: 'update',
      status: 'success',
      message: 'Node settings updated and node restart requested.',
      metadata: settings,
    });

    if (endpoint.providerPublicIp && process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
      const command = buildRemoteNodeSettingsSyncCommand({
        host: endpoint.providerPublicIp,
        user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
        identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
        settingsPath: REMOTE_NODE_SETTINGS_PATH,
        renderedSettings: JSON.stringify(settings, null, 2),
        syncScriptPath: REMOTE_NODE_SYNC_PATH,
        renderedSyncScript: runtimeArtifacts.runScript,
        neoGoConfigPath: runtimeArtifacts.neoGoConfig ? REMOTE_NEO_GO_CONFIG_PATH : undefined,
        renderedNeoGoConfig: runtimeArtifacts.neoGoConfig,
      });

      await execFileAsync('/bin/bash', ['-lc', command]);
    }

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function triggerEndpointResyncAction(endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    if (!endpoint.providerPublicIp || !process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
      return { success: false, error: 'Remote maintenance is not configured for this environment.' };
    }

    const command = buildRemoteResyncCommand({
      host: endpoint.providerPublicIp,
      user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
      identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
    });

    await prisma.endpoint.update({
      where: { id: endpoint.id },
      data: { status: 'Syncing' },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'maintenance',
      action: 'resync',
      status: 'success',
      message: 'Fast resync requested for the endpoint.',
    });

    await execFileAsync('/bin/bash', ['-lc', command]);
    revalidatePath(`/app/endpoints/${endpoint.id}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function exportEndpointSnapshotAction(endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);
    if (!endpoint.providerPublicIp || !process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
      return { success: false, error: 'Remote maintenance is not configured for this environment.' };
    }

    const snapshotPath = `/var/lib/neonexus/exports/endpoint-${endpoint.id}-${Date.now()}.tar.gz`;
    const command = buildRemoteSnapshotCommand({
      host: endpoint.providerPublicIp,
      user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
      identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
      snapshotPath,
    });

    await execFileAsync('/bin/bash', ['-lc', command]);
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'maintenance',
      action: 'snapshot_export',
      status: 'success',
      message: 'Snapshot export created on the node.',
      metadata: { snapshotPath },
    });
    return { success: true, snapshotPath };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
