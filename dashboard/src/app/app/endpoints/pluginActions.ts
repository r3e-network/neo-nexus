'use server';

import { auth } from '@/auth';
import { prisma } from '@/utils/prisma';
import { VaultService } from '@/services/vault/VaultService';
import { revalidatePath } from 'next/cache';

export async function addNodePluginAction(endpointId: number, pluginId: string, configData: any, privateKey?: string) {
    const session = await auth();
    if (!session?.user?.id) {
        return { success: false, error: 'Unauthorized' };
    }

    // 1. Verify ownership of the endpoint
    let orgId = (session.user as any).organizationId;
    if (!orgId) {
        return { success: false, error: 'No organization linked to user.' };
    }

    const endpoint = await prisma.endpoint.findUnique({
        where: { id: endpointId, organizationId: orgId }
    });

    if (!endpoint) {
        return { success: false, error: 'Endpoint not found or permission denied.' };
    }

    // 2. Validate Plugin Prerequisites
    if (pluginId === 'oracleService' || pluginId === 'tee-oracle' || pluginId === 'aa-bundler') {
        if (!privateKey) {
            return { success: false, error: 'A private key is required to operate this plugin securely.' };
        }
        
        // 3. Store the Secret Securely in the KMS Vault
        const secretName = `${pluginId}_private_key`;
        await VaultService.storeNodeSecret(endpoint.id, secretName, privateKey);
    }

    // 4. Create the Plugin Record
    try {
        await prisma.nodePlugin.upsert({
            where: {
                endpointId_pluginId: {
                    endpointId: endpoint.id,
                    pluginId: pluginId
                }
            },
            update: {
                status: 'Active',
                config: configData
            },
            create: {
                endpointId: endpoint.id,
                pluginId: pluginId,
                status: 'Active',
                config: configData
            }
        });

        // 5. In Production: Trigger KubernetesDeployer to patch the StatefulSet
        // e.g., KubernetesDeployer.patchPlugin(endpoint.k8sNamespace, pluginId, secretRef)
        // This would mount the secret into the pod and restart it.
        console.log(`[Control Plane] Plugin ${pluginId} activated for node ${endpoint.id}. Vault injected.`);

        revalidatePath(`/app/endpoints/${endpointId}`);
        return { success: true };

    } catch (error: any) {
        console.error('Failed to add node plugin:', error);
        return { success: false, error: error.message };
    }
}
