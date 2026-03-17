import { prisma } from '@/utils/prisma';
import { NeoNodeService } from '@/services/neo/NeoNodeService';
import { fetchRemoteNodeRuntimeStatus } from '@/services/nodes/RemoteNodeRuntime';

type EndpointRecord = {
  id: number;
  url: string;
  status: string;
  clientEngine: string;
  providerPublicIp?: string | null;
};

const PENDING_STATUSES = new Set(['Provisioning', 'Syncing']);

export async function synchronizeEndpointStatus(endpoint: EndpointRecord): Promise<string> {
  if (!endpoint.url) {
    return endpoint.status;
  }

  if (endpoint.providerPublicIp && process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
    const runtime = await fetchRemoteNodeRuntimeStatus({
      host: endpoint.providerPublicIp,
      user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
      identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
    });

    if (runtime.status !== endpoint.status) {
      await prisma.endpoint.update({
        where: { id: endpoint.id },
        data: {
          status: runtime.status,
        },
      });
      return runtime.status;
    }
  }

  if (!PENDING_STATUSES.has(endpoint.status)) {
    return endpoint.status;
  }

  const isHealthy = await NeoNodeService.checkHealth(endpoint.url, endpoint.clientEngine);
  if (!isHealthy) {
    return endpoint.status;
  }

  await prisma.endpoint.update({
    where: { id: endpoint.id },
    data: { status: 'Active' },
  });

  return 'Active';
}
