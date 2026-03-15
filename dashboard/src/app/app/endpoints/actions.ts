'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { KubernetesDeployer, DeploymentConfig } from '@/services/KubernetesDeployer';
import { ApisixService } from '@/services/apisix/ApisixService';
import { auth } from '@/auth';

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
  const isDatabaseConfigured = !!process.env.DATABASE_URL;

  if (!isDatabaseConfigured) {
    throw new Error('Database is not configured for production deployment.');
  }

  // Auth & Billing Verification
  const session = await auth();
  if (!session?.user) {
    return { success: false, error: 'Unauthorized: You must be logged in.' };
  }

  let orgId = (session.user as any).organizationId;
  let billingPlan = 'developer';

  if (!orgId) {
    const userDb = await prisma.user.findUnique({
      where: { id: session.user.id },
      include: { organization: true }
    });
    if (userDb?.organization) {
      orgId = userDb.organization.id;
      billingPlan = userDb.organization.billingPlan;
    } else {
      return { success: false, error: 'User does not belong to an organization.' };
    }
  } else {
      const org = await prisma.organization.findUnique({ where: { id: orgId } });
      if (org) billingPlan = org.billingPlan;
  }

  if (formData.type === 'dedicated' && billingPlan === 'developer') {
    return { success: false, error: 'Billing Plan Error: You must upgrade to Growth or Dedicated plan to deploy Dedicated Nodes.' };
  }

  // Enforce Endpoint Limits based on Billing Plan
  const currentSharedCount = await prisma.endpoint.count({
    where: { 
        organizationId: orgId, 
        type: 'Shared' 
    }
  });

  if (formData.type === 'shared') {
      if (billingPlan === 'developer' && currentSharedCount >= 1) {
          return { success: false, error: 'Plan Limit Reached: Developer plan is limited to 1 Shared Endpoint. Please upgrade your plan.' };
      }
      if (billingPlan === 'growth' && currentSharedCount >= 3) {
          return { success: false, error: 'Plan Limit Reached: Growth plan is limited to 3 Shared Endpoints.' };
      }
  }

  // 1. Simulate Control Plane Deployment
  const k8sConfig: DeploymentConfig = {
    name: formData.name,
    protocol: formData.protocol as 'neo-n3' | 'neo-x',
    network: formData.network as 'mainnet' | 'testnet',
    type: formData.type as 'shared' | 'dedicated',
    clientEngine: formData.clientEngine as 'neo-go' | 'neo-cli' | 'neo-x-geth',
    provider: formData.provider as 'aws' | 'gcp',
    region: formData.region,
    syncMode: formData.syncMode as 'full' | 'archive'
  };

  const k8sResult = await KubernetesDeployer.deployNode(k8sConfig);

  if (!k8sResult.success) {
    return { success: false, error: 'Control Plane failed to schedule deployment on Kubernetes cluster. ' + (k8sResult.error || '') };
  }

  // 2. Persist to Database
  // Generate a realistic routing URL
  const randomId = Math.random().toString(36).substring(2, 8);
  const url = formData.type === 'dedicated' 
    ? `https://node-${formData.region}-${randomId}.neonexus.com/v1`
    : `https://${formData.network}.neonexus.com/v1/${randomId}`;

  try {
    const networkString = formData.protocol === 'neo-x' 
        ? (formData.network === 'mainnet' ? 'Neo X Mainnet' : 'Neo X Testnet')
        : (formData.network === 'mainnet' ? 'N3 Mainnet' : 'N3 Testnet');

    const endpoint = await prisma.endpoint.create({
      data: {
        name: formData.name,
        network: networkString,
        type: formData.type.charAt(0).toUpperCase() + formData.type.slice(1),
        clientEngine: formData.clientEngine,
        cloudProvider: formData.type === 'dedicated' ? formData.provider : null,
        region: formData.type === 'dedicated' ? formData.region : null,
        url: url,
        status: 'Syncing', // Starts in syncing state
        requests: 0,
        k8sNamespace: k8sResult.namespace,
        k8sDeploymentName: k8sResult.releaseName,
        organizationId: orgId
      }
    });

    // 3. Register route with APISIX Gateway
    const internalHost = `${k8sResult.releaseName}.${k8sResult.namespace}.svc.cluster.local`;
    await ApisixService.createRoute(randomId, internalHost, formData.protocol === 'neo-x' ? 8545 : 10332);

    revalidatePath('/endpoints');
    return { success: true, id: endpoint.id };
  } catch (error: any) {
    console.error('Error creating endpoint:', error);
    return { success: false, error: error.message };
  }
}
