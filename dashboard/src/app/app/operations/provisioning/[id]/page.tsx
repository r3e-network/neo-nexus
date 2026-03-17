import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { filterProvisioningActivities } from '@/services/provisioning/ProvisioningActivity';
import { formatEndpointActivityMetadata } from '@/services/endpoints/EndpointActivityFormatter';

export const dynamic = 'force-dynamic';

export default async function ProvisioningOrderDetailPage(
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const { id } = await params;
  const orderId = Number.parseInt(id, 10);
  if (!Number.isInteger(orderId)) {
    notFound();
  }

  const order = await prisma.provisioningOrder.findUnique({
    where: { id: orderId },
    select: {
      id: true,
      provider: true,
      status: true,
      currentStep: true,
      attemptCount: true,
      lastAttemptAt: true,
      nextAttemptAt: true,
      errorMessage: true,
      createdAt: true,
      updatedAt: true,
      organization: {
        select: {
          name: true,
        },
      },
      endpoint: {
        select: {
          id: true,
          name: true,
          status: true,
          cloudProvider: true,
          providerPublicIp: true,
          region: true,
          url: true,
          activities: {
            orderBy: { createdAt: 'desc' },
            take: 20,
            select: {
              id: true,
              category: true,
              action: true,
              status: true,
              message: true,
              metadata: true,
              createdAt: true,
            },
          },
        },
      },
    },
  });

  if (!order) {
    notFound();
  }

  const provisioningActivities = filterProvisioningActivities(
    order.endpoint.activities.map((activity) => ({
      id: activity.id,
      category: activity.category,
      message: activity.message,
      createdAt: activity.createdAt.toISOString(),
    })),
  );

  const activityById = new Map(order.endpoint.activities.map((activity) => [activity.id, activity]));

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations/provisioning" className="text-sm text-gray-400 hover:text-white">
          Back to Provisioning Queue
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Provisioning Order #{order.id}</h1>
        <p className="text-gray-400 mt-1">Inspect the current order state, endpoint status, and recent provisioning activity trail.</p>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <Link href={`/app/endpoints/${order.endpoint.id}`} className="text-lg font-medium text-white hover:text-[#00E599]">
              {order.endpoint.name}
            </Link>
            <p className="text-xs text-gray-500 mt-1">
              Org: {order.organization.name} • Provider: {order.provider}
            </p>
          </div>
          <span className="text-xs text-gray-500">Created {order.createdAt.toLocaleString()}</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 text-sm text-gray-300">
          <div>Order status: {order.status}</div>
          <div>Current step: {order.currentStep}</div>
          <div>Attempts: {order.attemptCount}</div>
          <div>Endpoint status: {order.endpoint.status}</div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-300">
          <div>Region: {order.endpoint.region ?? 'Pending'}</div>
          <div>Cloud provider: {order.endpoint.cloudProvider ?? 'Pending'}</div>
          <div>Public IP: {order.endpoint.providerPublicIp ?? 'Pending'}</div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-300">
          <div>Last attempt: {order.lastAttemptAt?.toLocaleString() ?? 'Not attempted yet'}</div>
          <div>Next retry: {order.nextAttemptAt?.toLocaleString() ?? 'No retry scheduled'}</div>
        </div>

        {order.errorMessage && (
          <div className="border border-red-500/30 bg-red-500/10 rounded-lg p-4 text-sm text-red-300">
            {order.errorMessage}
          </div>
        )}
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <h2 className="text-lg font-medium text-white">Recent Provisioning Activity</h2>
        {provisioningActivities.length === 0 ? (
          <p className="text-sm text-gray-400">No provisioning activity recorded yet for this endpoint.</p>
        ) : (
          <div className="space-y-3">
            {provisioningActivities.map((activity) => {
              const fullActivity = activityById.get(activity.id);
              return (
                <div key={activity.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                  <div className="flex items-center justify-between gap-4 mb-2">
                    <span className="text-xs uppercase tracking-wide text-gray-500">{fullActivity?.action ?? 'provisioning'}</span>
                    <span className="text-xs text-gray-500">{new Date(activity.createdAt).toLocaleString()}</span>
                  </div>
                  <p className="text-sm text-gray-300">{activity.message}</p>
                  {fullActivity && formatEndpointActivityMetadata(fullActivity.metadata).length > 0 && (
                    <div className="mt-3 grid grid-cols-1 md:grid-cols-2 gap-2 text-xs text-gray-500">
                      {formatEndpointActivityMetadata(fullActivity.metadata).map((row) => (
                        <div key={row.label}>
                          <span className="text-gray-400">{row.label}:</span> {row.value}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
