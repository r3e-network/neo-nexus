'use server';

import type { Prisma } from '@prisma/client';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { prisma } from '@/utils/prisma';
import { VaultService } from '@/services/vault/VaultService';
import { revalidatePath } from 'next/cache';
import { getErrorMessage } from '@/server/errors';
import { recordEndpointActivity } from '@/services/endpoints/EndpointActivityService';
import {
  getPluginDefinition,
  isSupportedPlugin,
} from '@/services/plugins/PluginCatalog';
import {
  buildRemotePluginRemovalCommand,
  buildRemotePluginSyncCommand,
} from '@/services/plugins/RemotePluginSync';
import { renderPluginConfig } from '@/services/plugins/PluginConfigRenderer';
import { requireCurrentOrganizationContext } from '@/server/organization';

type NodePluginConfig = Prisma.InputJsonValue;
const execFileAsync = promisify(execFile);

export async function addNodePluginAction(
  endpointId: number,
  pluginId: string,
  configData: NodePluginConfig,
  privateKey?: string,
) {
  let orgId: string;

  try {
    ({ organizationId: orgId } = await requireCurrentOrganizationContext());
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId: orgId,
    },
  });

  if (!endpoint) {
    return { success: false, error: 'Endpoint not found or permission denied.' };
  }

  if (!isSupportedPlugin(pluginId)) {
    return { success: false, error: 'Unsupported plugin.' };
  }

  if (endpoint.type !== 'Dedicated') {
    return { success: false, error: 'Plugins are currently supported only on dedicated endpoints.' };
  }

  const pluginDefinition = getPluginDefinition(pluginId);
  if (!pluginDefinition) {
    return { success: false, error: 'Unsupported plugin.' };
  }

  try {
    const secretRefs: string[] = [];
    const secretName = `${pluginId}_private_key`;
    if (pluginDefinition.requiresPrivateKey) {
      if (!privateKey) {
        return { success: false, error: 'A private key is required to operate this plugin securely.' };
      }
      secretRefs.push(secretName);
    }

    const renderedConfig = renderPluginConfig({
      pluginId,
      endpointId: endpoint.id,
      secretRefs,
      secretPayloads: pluginDefinition.requiresPrivateKey
        ? { [`${pluginId}_private_key`]: privateKey ?? '' }
        : {},
      runtimeImage: pluginDefinition.defaultImage,
        configData,
    });

    const pluginRecord = await prisma.$transaction(async (tx) => {
      if (pluginDefinition.requiresPrivateKey) {
        await VaultService.storeNodeSecret(endpoint.id, secretName, privateKey ?? '', tx);
      }

      return tx.nodePlugin.upsert({
        where: {
          endpointId_pluginId: {
            endpointId: endpoint.id,
            pluginId,
          },
        },
        update: {
          status: 'Installing',
          errorMessage: null,
          config: {
            raw: configData,
            renderedConfig,
            secretRefs,
          },
        },
        create: {
          endpointId: endpoint.id,
          pluginId,
          status: 'Installing',
          errorMessage: null,
          config: {
            raw: configData,
            renderedConfig,
            secretRefs,
          },
        },
      });
    });

    if (endpoint.providerPublicIp && process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
      const remotePath = `/etc/neonexus/plugins/${pluginId}.json`;
      const command = buildRemotePluginSyncCommand({
        host: endpoint.providerPublicIp,
        user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
        identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
        remotePath,
        renderedConfig,
      });

      await execFileAsync('/bin/bash', [
        '-lc',
        command,
      ]);

      await prisma.nodePlugin.update({
        where: { id: pluginRecord.id },
        data: {
          status: 'Active',
          errorMessage: null,
          lastAppliedAt: new Date(),
        },
      });
      await recordEndpointActivity({
        endpointId: endpoint.id,
        category: 'plugin',
        action: 'install',
        status: 'success',
        message: `Plugin ${pluginId} applied on the managed node.`,
      });
    } else {
      await prisma.nodePlugin.update({
        where: { id: pluginRecord.id },
        data: {
          status: 'Pending Apply',
          errorMessage: 'Remote plugin sync is not configured for this environment.',
        },
      });
      await recordEndpointActivity({
        endpointId: endpoint.id,
        category: 'plugin',
        action: 'install_pending',
        status: 'pending',
        message: `Plugin ${pluginId} is waiting for remote sync configuration.`,
      });
    }

    revalidatePath(`/app/endpoints/${endpointId}`);
    return { success: true };
  } catch (error) {
    console.error('Failed to add node plugin:', error);
    const message = getErrorMessage(error);
    await prisma.nodePlugin.updateMany({
      where: {
        endpointId: endpoint.id,
        pluginId,
      },
      data: {
        status: 'Error',
        errorMessage: message,
      },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'plugin',
      action: 'install_failed',
      status: 'error',
      message: `Plugin ${pluginId} failed to apply.`,
      metadata: { error: message },
    });
    return { success: false, error: message };
  }
}

export async function removeNodePluginAction(endpointId: number, pluginId: string) {
  let orgId: string;

  try {
    ({ organizationId: orgId } = await requireCurrentOrganizationContext());
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId: orgId,
    },
    include: {
      plugins: {
        where: { pluginId },
        take: 1,
      },
    },
  });

  if (!endpoint) {
    return { success: false, error: 'Endpoint not found or permission denied.' };
  }

  const plugin = endpoint.plugins[0];
  if (!plugin) {
    return { success: false, error: 'Plugin is not installed on this endpoint.' };
  }

  try {
    if (endpoint.providerPublicIp && process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
      const remotePath = `/etc/neonexus/plugins/${pluginId}.json`;
      const command = buildRemotePluginRemovalCommand({
        host: endpoint.providerPublicIp,
        user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
        identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
        remotePath,
      });

      await execFileAsync('/bin/bash', ['-lc', command]);
    }

    const pluginDefinition = getPluginDefinition(pluginId);
    if (pluginDefinition?.requiresPrivateKey) {
      await prisma.nodeSecret.deleteMany({
        where: {
          endpointId: endpoint.id,
          name: `${pluginId}_private_key`,
        },
      });
    }

    await prisma.nodePlugin.delete({
      where: {
        endpointId_pluginId: {
          endpointId: endpoint.id,
          pluginId,
        },
      },
    });
    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'plugin',
      action: 'remove',
      status: 'success',
      message: `Plugin ${pluginId} removed from the endpoint.`,
    });

    revalidatePath(`/app/endpoints/${endpointId}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
