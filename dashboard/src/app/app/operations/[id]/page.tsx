import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function ReconcileRunDetailsPage(
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const { id } = await params;
  const runId = Number.parseInt(id, 10);
  if (!Number.isInteger(runId)) {
    notFound();
  }

  const run = await prisma.reconcileRun.findUnique({
    where: { id: runId },
  });

  if (!run) {
    notFound();
  }

  const metadata = (run.metadata ?? {}) as {
    resumedOrderIds?: number[];
    alertEndpointIds?: number[];
  };

  const linkedEndpoints = metadata.alertEndpointIds?.length
    ? await prisma.endpoint.findMany({
        where: {
          id: {
            in: metadata.alertEndpointIds,
          },
        },
        select: {
          id: true,
          name: true,
          status: true,
        },
      })
    : [];

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations" className="text-sm text-gray-400 hover:text-white">
          Back to Operations
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Reconcile Run #{run.id}</h1>
        <p className="text-gray-400 mt-1">Inspect the details of an individual control-plane reconcile execution.</p>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <div className="flex items-center justify-between">
          <span className={`text-xs px-2 py-1 rounded font-bold ${run.status === 'success' ? 'bg-[#00E599]/20 text-[#00E599]' : 'bg-red-500/20 text-red-300'}`}>
            {run.status}
          </span>
          <span className="text-sm text-gray-500">{run.createdAt.toLocaleString()}</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-300">
          <div className="border border-[var(--color-dark-border)] rounded-lg p-4">
            <div className="text-gray-500 text-xs uppercase tracking-wide mb-2">Provisioning Checked</div>
            <div className="text-xl font-bold text-white">{run.provisioningChecked}</div>
          </div>
          <div className="border border-[var(--color-dark-border)] rounded-lg p-4">
            <div className="text-gray-500 text-xs uppercase tracking-wide mb-2">Provisioning Resumed</div>
            <div className="text-xl font-bold text-white">{run.provisioningResumed}</div>
          </div>
          <div className="border border-[var(--color-dark-border)] rounded-lg p-4">
            <div className="text-gray-500 text-xs uppercase tracking-wide mb-2">Alerts Evaluated</div>
            <div className="text-xl font-bold text-white">{run.alertingEndpointsEvaluated}</div>
          </div>
        </div>

        {run.errorMessage && (
          <div className="border border-red-500/30 bg-red-500/10 rounded-lg p-4 text-sm text-red-300">
            {run.errorMessage}
          </div>
        )}

        {(metadata.resumedOrderIds?.length || linkedEndpoints.length || run.metadata) && (
          <div className="border border-[var(--color-dark-border)] rounded-lg p-4">
            <div className="text-gray-500 text-xs uppercase tracking-wide mb-3">Metadata</div>
            <div className="space-y-4">
              {metadata.resumedOrderIds?.length ? (
                <div>
                  <div className="text-xs text-gray-500 mb-2">Resumed Orders</div>
                  <div className="flex flex-wrap gap-2">
                    {metadata.resumedOrderIds.map((orderId) => (
                      <Link
                        key={orderId}
                        href={`/app/operations/provisioning/${orderId}`}
                        className="text-xs px-2 py-1 rounded bg-[var(--color-dark-border)] text-gray-300 hover:text-white"
                      >
                        Order #{orderId}
                      </Link>
                    ))}
                  </div>
                </div>
              ) : null}

              {linkedEndpoints.length ? (
                <div>
                  <div className="text-xs text-gray-500 mb-2">Alert Endpoints Evaluated</div>
                  <div className="space-y-2">
                    {linkedEndpoints.map((endpoint) => (
                      <Link
                        key={endpoint.id}
                        href={`/app/endpoints/${endpoint.id}`}
                        className="block text-sm text-[#00E599] hover:text-[#00cc88]"
                      >
                        {endpoint.name} (#{endpoint.id}) - {endpoint.status}
                      </Link>
                    ))}
                  </div>
                </div>
              ) : null}

              {run.metadata && (
                <pre className="text-xs text-gray-400 whitespace-pre-wrap overflow-x-auto">
                  {JSON.stringify(run.metadata, null, 2)}
                </pre>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
