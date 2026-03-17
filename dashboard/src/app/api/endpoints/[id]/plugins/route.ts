import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { fetchRemotePluginRuntimeStatus } from '@/services/plugins/RemotePluginRuntime';

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();

  if (!userContext?.organizationId) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const { id } = await params;
  const endpointId = Number.parseInt(id, 10);
  if (!Number.isInteger(endpointId)) {
    return NextResponse.json({ error: 'Invalid endpoint id' }, { status: 400 });
  }

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId: userContext.organizationId,
    },
    select: { id: true },
  });

  if (!endpoint) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  const plugins = await prisma.nodePlugin.findMany({
    where: { endpointId: endpoint.id },
    orderBy: { createdAt: 'asc' },
    select: {
      id: true,
      pluginId: true,
      status: true,
      errorMessage: true,
      lastAppliedAt: true,
      createdAt: true,
    },
  });

  if (process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
    const endpointDetails = await prisma.endpoint.findUnique({
      where: { id: endpoint.id },
      select: {
        providerPublicIp: true,
      },
    });

    if (endpointDetails?.providerPublicIp) {
      for (const plugin of plugins) {
        const runtime = await fetchRemotePluginRuntimeStatus({
          host: endpointDetails.providerPublicIp,
          user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
          identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
          pluginId: plugin.pluginId,
        });

        if (runtime.status !== plugin.status || runtime.errorMessage !== plugin.errorMessage) {
          await prisma.nodePlugin.update({
            where: { id: plugin.id },
            data: {
              status: runtime.status,
              errorMessage: runtime.errorMessage,
              lastAppliedAt: runtime.status === 'Active' ? new Date() : plugin.lastAppliedAt,
            },
          });

          plugin.status = runtime.status;
          plugin.errorMessage = runtime.errorMessage;
          if (runtime.status === 'Active') {
            plugin.lastAppliedAt = new Date();
          }
        }
      }
    }
  }

  return NextResponse.json(
    plugins.map((plugin) => ({
      ...plugin,
      lastAppliedAt: plugin.lastAppliedAt?.toISOString() ?? null,
      createdAt: plugin.createdAt.toISOString(),
    })),
  );
}
