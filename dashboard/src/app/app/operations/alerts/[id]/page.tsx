import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { filterAlertRuleActivities } from '@/services/alerts/AlertRuleActivity';
import { formatEndpointActivityMetadata } from '@/services/endpoints/EndpointActivityFormatter';
import SnoozeAlertRuleButton from '../../SnoozeAlertRuleButton';

export const dynamic = 'force-dynamic';

export default async function AlertRuleDetailPage(
  { params }: { params: Promise<{ id: string }> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const { id } = await params;
  const alertRuleId = Number.parseInt(id, 10);
  if (!Number.isInteger(alertRuleId)) {
    notFound();
  }

  const alertRule = await prisma.alertRule.findUnique({
    where: { id: alertRuleId },
    select: {
      id: true,
      name: true,
      condition: true,
      actionType: true,
      target: true,
      isActive: true,
      deliveryAttemptCount: true,
      nextDeliveryAt: true,
      snoozedUntil: true,
      lastTriggeredAt: true,
      lastDeliveredAt: true,
      lastResolvedAt: true,
      lastDeliveryError: true,
      deliveryAttempts: {
        orderBy: { createdAt: 'desc' },
        take: 20,
        select: {
          id: true,
          actionType: true,
          target: true,
          status: true,
          errorMessage: true,
          createdAt: true,
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
            where: { category: 'alert' },
            orderBy: { createdAt: 'desc' },
            take: 20,
            select: {
              id: true,
              action: true,
              status: true,
              message: true,
              metadata: true,
              createdAt: true,
            },
          },
          alertIncidents: {
            where: { alertRuleId },
            orderBy: { openedAt: 'desc' },
            take: 20,
            select: {
              id: true,
              status: true,
              severity: true,
              message: true,
              openedAt: true,
              resolvedAt: true,
              lastDeliveredAt: true,
              lastDeliveryError: true,
            },
          },
        },
      },
    },
  });

  if (!alertRule) {
    notFound();
  }

  const activities = filterAlertRuleActivities(alertRule.id, alertRule.endpoint.activities);

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations/alerts" className="text-sm text-gray-400 hover:text-white">
          Back to Alert Retry Queue
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Alert Rule #{alertRule.id}</h1>
        <p className="text-gray-400 mt-1">Inspect current rule state, incident history, and alert-related endpoint activity for this rule.</p>
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h2 className="text-lg font-medium text-white">{alertRule.name}</h2>
            <p className="text-xs text-gray-500 mt-1">
              Org: {alertRule.endpoint.organization?.name ?? 'Unknown'} • Endpoint:{' '}
              <Link href={`/app/endpoints/${alertRule.endpoint.id}`} className="text-[#00E599] hover:text-[#00cc88]">
                {alertRule.endpoint.name}
              </Link>
            </p>
          </div>
          <span className={`text-xs px-2 py-1 rounded font-bold ${alertRule.isActive ? 'bg-[#00E599]/20 text-[#00E599]' : 'bg-gray-700 text-gray-300'}`}>
            {alertRule.isActive ? 'Active' : 'Disabled'}
          </span>
        </div>
        <div className="flex flex-wrap gap-2">
          <SnoozeAlertRuleButton alertRuleId={alertRule.id} snoozeMinutes={60} label="Snooze 1h" />
          <SnoozeAlertRuleButton alertRuleId={alertRule.id} snoozeMinutes={240} label="Snooze 4h" />
          <SnoozeAlertRuleButton alertRuleId={alertRule.id} snoozeMinutes={1440} label="Snooze 24h" />
          <SnoozeAlertRuleButton alertRuleId={alertRule.id} snoozeMinutes={null} label="Clear Snooze" />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-300">
          <div>Condition: {alertRule.condition}</div>
          <div>Action: {alertRule.actionType}</div>
          <div>Target: {alertRule.target}</div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 text-sm text-gray-300">
          <div>Attempts: {alertRule.deliveryAttemptCount}</div>
          <div>Last triggered: {alertRule.lastTriggeredAt?.toLocaleString() ?? 'Never'}</div>
          <div>Last delivered: {alertRule.lastDeliveredAt?.toLocaleString() ?? 'Never'}</div>
          <div>Next retry: {alertRule.nextDeliveryAt?.toLocaleString() ?? 'None scheduled'}</div>
        </div>
        {alertRule.snoozedUntil && (
          <div className="text-sm text-amber-300">
            Snoozed until: {alertRule.snoozedUntil.toLocaleString()}
          </div>
        )}

        {alertRule.lastDeliveryError && (
          <div className="border border-red-500/30 bg-red-500/10 rounded-lg p-4 text-sm text-red-300">
            Last delivery error: {alertRule.lastDeliveryError}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
          <h2 className="text-lg font-medium text-white">Incidents For This Rule</h2>
          {alertRule.endpoint.alertIncidents.length === 0 ? (
            <p className="text-sm text-gray-400">No incidents recorded for this rule.</p>
          ) : (
            <div className="space-y-3">
              {alertRule.endpoint.alertIncidents.map((incident) => (
                <div key={incident.id} className="border border-[var(--color-dark-border)] rounded-lg p-4 space-y-2">
                  <div className="flex items-center justify-between gap-4">
                    <Link href={`/app/operations/incidents/${incident.id}`} className="text-sm font-medium text-white hover:text-[#00E599]">
                      Incident #{incident.id}
                    </Link>
                    <span className={`text-xs px-2 py-1 rounded font-bold ${incident.status === 'Open' ? 'bg-red-500/20 text-red-300' : 'bg-[#00E599]/20 text-[#00E599]'}`}>
                      {incident.status}
                    </span>
                  </div>
                  <p className="text-sm text-gray-300">{incident.message}</p>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs text-gray-500">
                    <div>Severity: {incident.severity}</div>
                    <div>Opened: {incident.openedAt.toLocaleString()}</div>
                    <div>Resolved: {incident.resolvedAt?.toLocaleString() ?? 'Open'}</div>
                  </div>
                  {incident.lastDeliveryError && (
                    <p className="text-sm text-red-300">{incident.lastDeliveryError}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
          <h2 className="text-lg font-medium text-white">Recent Delivery Attempts</h2>
          {alertRule.deliveryAttempts.length === 0 ? (
            <p className="text-sm text-gray-400">No delivery attempts have been recorded for this rule yet.</p>
          ) : (
            <div className="space-y-3">
              {alertRule.deliveryAttempts.map((attempt) => (
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
          <h2 className="text-lg font-medium text-white">Recent Alert Activity</h2>
          {activities.length === 0 ? (
            <p className="text-sm text-gray-400">No recent alert activity recorded for this rule.</p>
          ) : (
            <div className="space-y-3">
              {activities.map((activity) => (
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
    </div>
  );
}
