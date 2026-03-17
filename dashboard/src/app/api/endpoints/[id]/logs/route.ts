import { NextResponse } from 'next/server';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { fetchRemoteNodeLogs } from '@/services/nodes/RemoteNodeRuntime';

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
    select: {
      id: true,
      type: true,
      providerPublicIp: true,
    },
  });

  if (!endpoint) {
    return NextResponse.json({ error: 'Not found' }, { status: 404 });
  }

  if (endpoint.type !== 'Dedicated') {
    return NextResponse.json(
      { error: 'Shared endpoint logs are not directly available from the managed node UI.' },
      { status: 400 },
    );
  }

  if (!endpoint.providerPublicIp || !process.env.VM_OPERATOR_PRIVATE_KEY_PATH) {
    return NextResponse.json(
      { error: 'Remote node log access is not configured for this environment.' },
      { status: 400 },
    );
  }

  const logs = await fetchRemoteNodeLogs({
    host: endpoint.providerPublicIp,
    user: process.env.VM_OPERATOR_SSH_USER ?? 'root',
    identityPath: process.env.VM_OPERATOR_PRIVATE_KEY_PATH,
  });

  return NextResponse.json({ logs });
}
