import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { buildIncidentTimeline } from '@/services/alerts/IncidentTimeline';
import { formatEndpointActivityMetadata } from '@/services/endpoints/EndpointActivityFormatter';
import ResolveIncidentButton from '../../ResolveIncidentButton';

export const dynamic = 'force-dynamic';

export default async function IncidentDetailPage(
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const { id } = await params;
  const incidentId = Number.parseInt(id, 10);
  if (!Number.isInteger(incidentId)) {
    notFound();
  }

  const incident = await prisma.alertIncident.findUnique({
    where: { id: incidentId },
    select: {
      id: true,
      status: true,
      severity: true,
      message: true,
      openedAt: true,
      resolvedAt: true,
      lastDeliveredAt: true,
      lastDeliveryError: true,
      alertRule: {
        select: {
          id: true,
          name: true,
          actionType: true,
          target: true,
          condition: true,
          deliveryAttemptCount: true,
          nextDeliveryAt: true,
          lastDeliveryError: true,
          lastTriggeredAt: true,
          lastDeliveredAt: true,
          deliveryAttempts: {
            orderBy: { createdAt: 'desc' },
            take: 10,
            select: {
              id: true,
              actionType: true,
              target: true,
              status: true,
              errorMessage: true,
              createdAt: true,
            },
          },
        },
      },
      endpoint: {
        select: {
          id: true,
          name: true,
          status: true,
          organization: {
            select: {
              name: true,
            },
          },
          activities: {
            where: {
              category: 'alert',
            },
            orderBy: { createdAt: 'desc' },
            take: 12,
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

  if (!incident) {
    notFound();
  }

  const timeline = buildIncidentTimeline({
    incident: {
      openedAt: incident.openedAt.toISOString(),
      resolvedAt: incident.resolvedAt?.toISOString() ?? null,
      lastDeliveredAt: incident.lastDeliveredAt?.toISOString() ?? null,
      lastDeliveryError: incident.lastDeliveryError,
    },
    activities: incident.endpoint.activities.map((activity) => ({
      id: activity.id,
      category: activity.category,
      action: activity.action,
      status: activity.status,
      message: activity.message,
      createdAt: activity.createdAt.toISOString(),
    })),
  });

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations/incidents" className="text-sm text-gray-400 hover:text-white">
          Back to Open Incidents
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Incident #{incident.id}</h1>
        <p className="text-gray-400 mt-1">Inspect the incident record, related alert rule state, and recent alert delivery/activity context.</p>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <Link href={`/app/endpoints/${incident.endpoint.id}`} className="text-lg font-medium text-white hover:text-[#00E599]">
              {incident.endpoint.name}
            </Link>
            <p className="text-xs text-gray-500 mt-1">
              Org: {incident.endpoint.organization?.name ?? 'Unknown'} • Rule: {incident.alertRule.name}
            </p>
          </div>
          <span className={`text-xs px-2 py-1 rounded font-bold ${
            incident.severity === 'critical' ? 'bg-red-500/20 text-red-300' : 'bg-amber-500/20 text-amber-300'
          }`}>
            {incident.severity}
          </span>
        </div>
        {incident.status === 'Open' && (
          <div>
            <ResolveIncidentButton incidentId={incident.id} />
          </div>
        )}

        <p className="text-sm text-gray-300">{incident.message}</p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-300">
          <div>Incident status: {incident.status}</div>
          <div>Endpoint status: {incident.endpoint.status}</div>
          <div>Opened: {incident.openedAt.toLocaleString()}</div>
        </div>

        {incident.lastDeliveryError && (
          <div className="border border-red-500/30 bg-red-500/10 rounded-lg p-4 text-sm text-red-300">
            Last delivery error: {incident.lastDeliveryError}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
          <h2 className="text-lg font-medium text-white">Alert Rule State</h2>
          <Link href={`/app/operations/alerts/${incident.alertRule.id}`} className="inline-flex text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
            Open Alert Rule Detail
          </Link>
          <div className="space-y-2 text-sm text-gray-300">
            <div>Condition: {incident.alertRule.condition}</div>
            <div>Action: {incident.alertRule.actionType}</div>
            <div>Target: {incident.alertRule.target}</div>
            <div>Delivery attempts: {incident.alertRule.deliveryAttemptCount}</div>
            <div>Last triggered: {incident.alertRule.lastTriggeredAt?.toLocaleString() ?? 'Never'}</div>
            <div>Last delivered: {incident.alertRule.lastDeliveredAt?.toLocaleString() ?? 'Never'}</div>
            <div>Next retry: {incident.alertRule.nextDeliveryAt?.toLocaleString() ?? 'None scheduled'}</div>
          </div>
          {incident.alertRule.lastDeliveryError && (
            <p className="text-sm text-red-300">{incident.alertRule.lastDeliveryError}</p>
          )}
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
          <h2 className="text-lg font-medium text-white">Recent Delivery Attempts</h2>
          {incident.alertRule.deliveryAttempts.length === 0 ? (
            <p className="text-sm text-gray-400">No delivery attempts recorded for this rule yet.</p>
          ) : (
            <div className="space-y-3">
              {incident.alertRule.deliveryAttempts.map((attempt) => (
                <div key={attempt.id} className="border border-[var(--color-dark-border)] rounded-lg p-4 space-y-2">
                  <div className="flex items-center justify-between gap-4">
                    <span className={`text-xs px-2 py-1 rounded font-bold ${attempt.status === 'succeeded' ? 'bg-[#00E599]/20 text-[#00E599]' : 'bg-red-500/20 text-red-300'}`}>
                      {attempt.status}
                    </span>
                    <span className="text-xs text-gray-500">{attempt.createdAt.toLocaleString()}</span>
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-300">
                    <div>Action: {attempt.actionType}</div>
                    <div>Target: {attempt.target}</div>
                  </div>
                  {attempt.errorMessage && (
                    <p className="text-sm text-red-300">{attempt.errorMessage}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4 xl:col-span-2">
          <h2 className="text-lg font-medium text-white">Incident Timeline</h2>
          {timeline.length === 0 ? (
            <p className="text-sm text-gray-400">No incident events available.</p>
          ) : (
            <div className="space-y-3">
              {timeline.map((item, index) => (
                <div key={`${item.kind}-${item.createdAt}-${index}`} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                  <div className="flex items-center justify-between gap-4 mb-2">
                    <span className="text-xs uppercase tracking-wide text-gray-500">{item.kind}</span>
                    <span className="text-xs text-gray-500">{new Date(item.createdAt).toLocaleString()}</span>
                  </div>
                  <p className={`text-sm ${
                    item.status === 'error' ? 'text-red-300'
                      : item.status === 'success' || item.status === 'resolved' ? 'text-[#00E599]'
                        : 'text-gray-300'
                  }`}>
                    {item.message}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <h2 className="text-lg font-medium text-white">Recent Alert Activity</h2>
        {incident.endpoint.activities.length === 0 ? (
          <p className="text-sm text-gray-400">No recent alert activity recorded for this endpoint.</p>
        ) : (
          <div className="space-y-3">
            {incident.endpoint.activities.map((activity) => (
              <div key={activity.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                <div className="flex items-center justify-between gap-4 mb-2">
                  <span className="text-xs uppercase tracking-wide text-gray-500">{activity.action}</span>
                  <span className="text-xs text-gray-500">{activity.createdAt.toLocaleString()}</span>
                </div>
                <p className="text-sm text-gray-300">{activity.message}</p>
                {formatEndpointActivityMetadata(activity.metadata).length > 0 && (
                  <div className="mt-3 grid grid-cols-1 md:grid-cols-2 gap-2 text-xs text-gray-500">
                    {formatEndpointActivityMetadata(activity.metadata).map((row) => (
                      <div key={row.label}>
                        <span className="text-gray-400">{row.label}:</span> {row.value}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
