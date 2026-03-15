'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { KubernetesDeployer, DeploymentConfig } from '@/services/KubernetesDeployer';
import { ApisixService } from '@/services/apisix/ApisixService';

export async function createEndpointAction(formData: {
  name: string;
  network: string;
  type: string;
  clientEngine: string;
  provider: string;
  region: string;
  syncMode: string;
}) {
  const isDatabaseConfigured = !!process.env.DATABASE_URL;

  // 1. Simulate Control Plane Deployment
  const k8sConfig: DeploymentConfig = {
    name: formData.name,
    network: formData.network as 'mainnet' | 'testnet',
    type: formData.type as 'shared' | 'dedicated',
    clientEngine: formData.clientEngine as 'neo-go' | 'neo-cli',
    provider: formData.provider as 'aws' | 'gcp',
    region: formData.region,
    syncMode: formData.syncMode as 'full' | 'archive'
  };

  const k8sResult = await KubernetesDeployer.deployNode(k8sConfig);

  if (!k8sResult.success) {
    return { success: false, error: 'Control Plane failed to schedule deployment on Kubernetes cluster.' };
  }

  // 2. Persist to Database
  if (isDatabaseConfigured) {
    // Generate a mock URL based on config
    const randomId = Math.random().toString(36).substring(2, 8);
    const url = formData.type === 'dedicated' 
      ? `https://node-${formData.region}-${randomId}.neonexus.io/v1`
      : `https://${formData.network}.neonexus.io/v1/${randomId}`;

    try {
      const endpoint = await prisma.endpoint.create({
        data: {
          name: formData.name,
          network: formData.network === 'mainnet' ? 'N3 Mainnet' : 'N3 Testnet',
          type: formData.type.charAt(0).toUpperCase() + formData.type.slice(1),
          clientEngine: formData.clientEngine,
          cloudProvider: formData.type === 'dedicated' ? formData.provider : null,
          region: formData.type === 'dedicated' ? formData.region : null,
          url: url,
          status: 'Syncing', // Starts in syncing state
          requests: 0,
          k8sNamespace: k8sResult.namespace,
          k8sDeploymentName: k8sResult.releaseName
        }
      });

      // 3. Register route with APISIX Gateway
      // The internal hostname in K8s: <service-name>.<namespace>.svc.cluster.local
      const internalHost = `${k8sResult.releaseName}.${k8sResult.namespace}.svc.cluster.local`;
      await ApisixService.createRoute(randomId, internalHost, 10332);

      revalidatePath('/endpoints');
      return { success: true, id: endpoint.id };
    } catch (error: any) {
      console.error('Error creating endpoint:', error);
      return { success: false, error: error.message };
    }
  } else {
    // Mock successful creation for local dev without DB
    console.log('Database not configured. Mocking endpoint creation:', formData);
    return { success: true, id: 1 }; // Return mock ID 1
  }
}
