import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { fetchRemotePluginLogs } from '@/services/plugins/RemotePluginRuntime';

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string; pluginId: string }> },
) {
  const userContext = await getCurrentUserContext();

  if (!userContext?.organizationId) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const { id, pluginId } = await params;
  const endpointId = Number.parseInt(id, 10);
  if (!Number.isInteger(endpointId)) {
    return NextResponse.json({ error: 'Invalid endpoint id' }, { status: 400 });
  }

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId: userContext.organizationId,
    },
    select: {
      id: true,
      providerPublicIp: true,
    },
  });

  if (!endpoint) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  if (!endpoint.providerPublicIp || !process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
    return NextResponse.json(
      { error: 'Remote plugin log access is not configured for this environment.' },
      { status: 400 },
    );
  }

  const plugin = await prisma.nodePlugin.findFirst({
    where: {
      endpointId: endpoint.id,
      pluginId,
    },
    select: { id: true },
  });

  if (!plugin) {
    return NextResponse.json({ error: 'Plugin not found.' }, { status: 404 });
  }

  const logs = await fetchRemotePluginLogs({
    host: endpoint.providerPublicIp,
    user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
    identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
    pluginId,
  });

  return NextResponse.json({ pluginId, logs });
}
