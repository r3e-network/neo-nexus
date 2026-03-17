import { prisma } from '../../utils/prisma';
import { ApisixService } from '../apisix/ApisixService';
import { recordEndpointActivity } from '../endpoints/EndpointActivityService';
import { getSharedBackendTarget } from '../endpoints/SharedEndpointConfig';
import { NeoNodeService } from '../neo/NeoNodeService';
import { loadOrganizationRoutePlugins } from '../security/RouteSecuritySync';
import type { DeploymentConfig } from '../infrastructure/DeploymentConfig';
import type { InfrastructureProvider } from '../infrastructure/ProviderCatalog';
import { provisionDedicatedNode } from './VmProvisioner';
import type { ProvisioningOrderStatus } from './ProvisioningOrderService';
import { buildProvisioningFailureUpdate } from './ProvisioningRetryPolicy';

type SharedBackendTarget = {
  host: string;
  port: number;
};

type ProvisioningJob = {
  orderId: number;
  organizationId: string;
  endpointId: number;
  publicUrl: string;
  routeKey: string;
  providerLabel: string;
  deploymentConfig: DeploymentConfig;
  sharedBackend?: SharedBackendTarget;
};

const activeJobs = new Set<number>();

function getProvisioningIntervalMs(): number {
  return Number.parseInt(process.env.VM_PROVISIONING_POLL_INTERVAL_MS ?? '5000', 10);
}

function getProvisioningAttempts(): number {
  return Number.parseInt(process.env.VM_PROVISIONING_MAX_ATTEMPTS ?? '60', 10);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function updateOrder(orderId: number, status: ProvisioningOrderStatus, errorMessage?: string) {
  await prisma.provisioningOrder.update({
    where: { id: orderId },
    data: {
      status,
      currentStep: status,
      nextAttemptAt: null,
      errorMessage,
    },
  });
}

async function markFailed(orderId: number, endpointId: number, errorMessage: string) {
  const order = await prisma.provisioningOrder.findUnique({
    where: { id: orderId },
    select: { attemptCount: true },
  });
  const failureUpdate = buildProvisioningFailureUpdate({
    attemptCount: order?.attemptCount ?? 0,
    errorMessage,
    now: new Date(),
  });

  await prisma.endpoint.update({
    where: { id: endpointId },
    data: {
      status: 'Error',
    },
  });
  await recordEndpointActivity({
    endpointId,
    category: 'provisioning',
    action: failureUpdate.status === 'failed' ? 'failed' : 'retry_scheduled',
    status: failureUpdate.status === 'failed' ? 'error' : 'pending',
    message: errorMessage,
    metadata: failureUpdate.nextAttemptAt
      ? { nextAttemptAt: failureUpdate.nextAttemptAt.toISOString() }
      : undefined,
  });
  await prisma.provisioningOrder.update({
    where: { id: orderId },
    data: failureUpdate,
  });
}

async function waitForEndpointReady(job: ProvisioningJob) {
  const attempts = getProvisioningAttempts();
  const intervalMs = getProvisioningIntervalMs();

  for (let attempt = 0; attempt < attempts; attempt += 1) {
    await sleep(intervalMs);

    const endpoint = await prisma.endpoint.findUnique({
      where: { id: job.endpointId },
      select: {
        id: true,
        url: true,
        clientEngine: true,
      },
    });

    if (!endpoint?.url) {
      break;
    }

    const isHealthy = await NeoNodeService.checkHealth(endpoint.url, endpoint.clientEngine);
    if (isHealthy) {
      await prisma.endpoint.update({
        where: { id: job.endpointId },
        data: { status: 'Active' },
      });
      await recordEndpointActivity({
        endpointId: job.endpointId,
        category: 'provisioning',
        action: 'ready',
        status: 'success',
        message: 'Endpoint is healthy and serving traffic.',
      });
      await updateOrder(job.orderId, 'ready');
      return;
    }
  }

  await markFailed(
    job.orderId,
    job.endpointId,
    'Provisioning timed out before the node became healthy.',
  );
}

async function runSharedProvisioning(job: ProvisioningJob) {
  if (!job.sharedBackend) {
    throw new Error('Shared provisioning requires a configured upstream target.');
  }

  await updateOrder(job.orderId, 'syncing');
  const routePlugins = await loadOrganizationRoutePlugins(job.organizationId);
  const routeCreated = await ApisixService.createRoute(
    job.routeKey,
    job.publicUrl,
    job.sharedBackend.host,
    job.sharedBackend.port,
    routePlugins,
  );

  if (!routeCreated) {
    throw new Error('Failed to create APISIX route for the shared endpoint.');
  }

  await prisma.endpoint.update({
    where: { id: job.endpointId },
    data: { status: 'Active' },
  });
  await recordEndpointActivity({
    endpointId: job.endpointId,
    category: 'provisioning',
    action: 'shared_route_configured',
    status: 'success',
    message: 'Shared endpoint route configured and marked active.',
  });
  await updateOrder(job.orderId, 'ready');
}

async function runDedicatedProvisioning(job: ProvisioningJob) {
  await updateOrder(job.orderId, 'vm_creating');
  await recordEndpointActivity({
    endpointId: job.endpointId,
    category: 'provisioning',
    action: 'vm_creating',
    status: 'pending',
    message: 'Starting dedicated VM provisioning.',
    metadata: {
      provider: job.deploymentConfig.provider,
      region: job.deploymentConfig.region,
    },
  });

  const provisionedNode = await provisionDedicatedNode(job.deploymentConfig);

  await prisma.endpoint.update({
    where: { id: job.endpointId },
    data: {
      cloudProvider: provisionedNode.provider as InfrastructureProvider,
      providerServerId: provisionedNode.providerServerId,
      providerPublicIp: provisionedNode.publicIp,
      region: provisionedNode.region,
      status: 'Provisioning',
    },
  });
  await recordEndpointActivity({
    endpointId: job.endpointId,
    category: 'provisioning',
    action: 'vm_created',
    status: 'success',
    message: 'Dedicated VM created successfully.',
    metadata: {
      provider: provisionedNode.provider,
      providerServerId: provisionedNode.providerServerId,
      publicIp: provisionedNode.publicIp,
      fallbackUsed: provisionedNode.fallbackUsed,
    },
  });

  await updateOrder(job.orderId, 'software_installing');

  const routePlugins = await loadOrganizationRoutePlugins(job.organizationId);
  const routeCreated = await ApisixService.createRoute(
    job.routeKey,
    job.publicUrl,
    provisionedNode.publicIp,
    job.deploymentConfig.protocol === 'neo-x' ? 8545 : 10332,
    routePlugins,
  );

  if (!routeCreated) {
    throw new Error('Failed to create APISIX route for the dedicated endpoint.');
  }

  await prisma.endpoint.update({
    where: { id: job.endpointId },
    data: { status: 'Syncing' },
  });
  await recordEndpointActivity({
    endpointId: job.endpointId,
    category: 'provisioning',
    action: 'route_configured',
    status: 'success',
    message: 'APISIX route configured for dedicated endpoint.',
  });
  await updateOrder(job.orderId, 'syncing');
  await waitForEndpointReady(job);
}

async function runProvisioningJob(job: ProvisioningJob) {
  try {
    await prisma.provisioningOrder.update({
      where: { id: job.orderId },
      data: {
        attemptCount: { increment: 1 },
        lastAttemptAt: new Date(),
        nextAttemptAt: null,
      },
    });

    if (job.deploymentConfig.type === 'shared') {
      await runSharedProvisioning(job);
      return;
    }

    await runDedicatedProvisioning(job);
  } catch (error) {
    await markFailed(
      job.orderId,
      job.endpointId,
      error instanceof Error ? error.message : 'Unknown provisioning error',
    );
  }
}

export function enqueueProvisioningJob(job: ProvisioningJob) {
  if (activeJobs.has(job.orderId)) {
    return;
  }

  activeJobs.add(job.orderId);
  void runProvisioningJob(job).finally(() => {
    activeJobs.delete(job.orderId);
  });
}

function inferProtocol(protocol: string): DeploymentConfig['protocol'] {
  return protocol === 'neo-x' ? 'neo-x' : 'neo-n3';
}

function inferNetwork(networkKey: string): DeploymentConfig['network'] {
  if (networkKey === 'testnet' || networkKey === 'private') {
    return networkKey;
  }

  return 'mainnet';
}

function inferType(type: string): DeploymentConfig['type'] {
  return type.toLowerCase() === 'shared' ? 'shared' : 'dedicated';
}

function inferClientEngine(clientEngine: string): DeploymentConfig['clientEngine'] {
  if (clientEngine === 'neo-cli' || clientEngine === 'neo-x-geth') {
    return clientEngine;
  }

  return 'neo-go';
}

function inferProvider(provider: string | null): DeploymentConfig['provider'] {
  return provider === 'digitalocean' ? 'digitalocean' : 'hetzner';
}

function inferSyncMode(syncMode: string): DeploymentConfig['syncMode'] {
  return syncMode === 'archive' ? 'archive' : 'full';
}

async function buildJobFromOrder(orderId: number): Promise<ProvisioningJob | null> {
  const order = await prisma.provisioningOrder.findUnique({
    where: { id: orderId },
    include: {
      endpoint: {
        select: {
          id: true,
          name: true,
          protocol: true,
          networkKey: true,
          type: true,
          clientEngine: true,
          syncMode: true,
          url: true,
          region: true,
          cloudProvider: true,
        },
      },
    },
  });

  if (!order) {
    return null;
  }

  const endpoint = order.endpoint;
  const deploymentConfig: DeploymentConfig = {
    name: endpoint.name,
    protocol: inferProtocol(endpoint.protocol),
    network: inferNetwork(endpoint.networkKey),
    type: inferType(endpoint.type),
    clientEngine: inferClientEngine(endpoint.clientEngine),
    provider: inferProvider(endpoint.cloudProvider),
    region: endpoint.region ?? 'fsn1',
    syncMode: inferSyncMode(endpoint.syncMode),
  };

  return {
    orderId: order.id,
    organizationId: order.organizationId,
    endpointId: endpoint.id,
    publicUrl: endpoint.url,
    routeKey: String(endpoint.id),
    providerLabel: order.provider,
    deploymentConfig,
    sharedBackend: deploymentConfig.type === 'shared'
      ? getSharedBackendTarget(deploymentConfig.protocol, endpoint.networkKey)
      : undefined,
  };
}

export async function kickoffProvisioningOrder(orderId: number) {
  if (activeJobs.has(orderId)) {
    return false;
  }

  const job = await buildJobFromOrder(orderId);
  if (!job) {
    return false;
  }

  const order = await prisma.provisioningOrder.findUnique({
    where: { id: orderId },
    select: {
      status: true,
      nextAttemptAt: true,
    },
  });

  if (!order || order.status === 'ready' || order.status === 'failed') {
    return false;
  }

  if (order.nextAttemptAt && order.nextAttemptAt > new Date()) {
    return false;
  }

  enqueueProvisioningJob(job);
  return true;
}
