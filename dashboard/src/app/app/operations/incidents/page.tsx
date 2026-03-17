import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { clampOperationsItemsLimit } from '@/services/reconciliation/OperationsDrilldownWindow';
import {
  buildIncidentQueueWhere,
  buildOperationsQueueHref,
  normalizeOperationsEnumFilter,
  normalizeOperationsSearchQuery,
} from '@/services/reconciliation/OperationsQueueFilters';
import ResolveIncidentsButton from '../ResolveIncidentsButton';
import ResolveIncidentButton from '../ResolveIncidentButton';
import SnoozeAlertRuleButton from '../SnoozeAlertRuleButton';

export const dynamic = 'force-dynamic';

export default async function OpenIncidentsPage(
  { searchParams }: { searchParams?: Promise<Record<string, string | string[] | undefined>> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const params = searchParams ? await searchParams : {};
  const rawLimit = typeof params.limit === 'string' ? params.limit : undefined;
  const limit = clampOperationsItemsLimit(rawLimit);
  const query = normalizeOperationsSearchQuery(typeof params.q === 'string' ? params.q : undefined);
  const severity = normalizeOperationsEnumFilter(
    typeof params.severity === 'string' ? params.severity : undefined,
    ['critical', 'warning'] as const,
  );
  const snoozed = normalizeOperationsEnumFilter(
    typeof params.snoozed === 'string' ? params.snoozed : undefined,
    ['snoozed', 'unsnoozed'] as const,
  );
  const now = new Date();

  const incidents = await prisma.alertIncident.findMany({
    where: buildIncidentQueueWhere({
      query,
      severity,
      snoozed,
      now,
    }),
    orderBy: { openedAt: 'desc' },
    take: limit,
    select: {
      id: true,
      severity: true,
      message: true,
      openedAt: true,
      lastDeliveredAt: true,
      lastDeliveryError: true,
      alertRule: {
        select: {
          id: true,
          name: true,
          snoozedUntil: true,
        },
      },
      endpoint: {
        select: {
          id: true,
          name: true,
          organization: {
            select: {
              name: true,
            },
          },
        },
      },
    },
  });

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations" className="text-sm text-gray-400 hover:text-white">
          Back to Operations
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Open Incidents</h1>
        <p className="text-gray-400 mt-1">Showing up to {limit} currently open alert incidents across all organizations.</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Link
          href={buildOperationsQueueHref('/app/operations/incidents', {})}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            severity === 'all' && snoozed === 'all' && !query
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          All
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/incidents', { severity: 'critical' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            severity === 'critical'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Critical
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/incidents', { severity: 'warning' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            severity === 'warning'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Warning
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/incidents', { snoozed: 'snoozed' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            snoozed === 'snoozed'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Snoozed
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/incidents', { snoozed: 'unsnoozed' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            snoozed === 'unsnoozed'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Unsnoozed
        </Link>
      </div>
      <form className="grid grid-cols-1 md:grid-cols-5 gap-3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
        <input
          type="text"
          name="q"
          defaultValue={query ?? ''}
          placeholder="Search org, endpoint, rule, or incident"
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        />
        <select
          name="severity"
          defaultValue={severity}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All severities</option>
          <option value="critical">Critical</option>
          <option value="warning">Warning</option>
        </select>
        <select
          name="snoozed"
          defaultValue={snoozed}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All snooze states</option>
          <option value="snoozed">Snoozed</option>
          <option value="unsnoozed">Unsnoozed</option>
        </select>
        <select
          name="limit"
          defaultValue={String(limit)}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="25">25 rows</option>
          <option value="50">50 rows</option>
          <option value="100">100 rows</option>
          <option value="200">200 rows</option>
        </select>
        <div className="flex gap-2">
          <button type="submit" className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-md text-sm font-bold transition-colors">
            Apply
          </button>
          <Link href="/app/operations/incidents" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors">
            Reset
          </Link>
        </div>
      </form>
      <div className="flex justify-end">
        <ResolveIncidentsButton incidentIds={incidents.map((incident) => incident.id)} />
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden">
        {incidents.length === 0 ? (
          <div className="p-6 text-sm text-gray-400">No open incidents.</div>
        ) : (
          <div className="divide-y divide-[var(--color-dark-border)]">
            {incidents.map((incident) => (
              <div key={incident.id} className="p-6 space-y-3">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <Link href={`/app/endpoints/${incident.endpoint.id}`} className="text-base font-medium text-white hover:text-[#00E599]">
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
                <p className="text-sm text-gray-300">{incident.message}</p>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs text-gray-500">
                  <div>Opened: {incident.openedAt.toLocaleString()}</div>
                  <div>Last delivered: {incident.lastDeliveredAt?.toLocaleString() ?? 'Not delivered yet'}</div>
                  <div>Incident #{incident.id}</div>
                </div>
                {incident.alertRule.snoozedUntil && (
                  <p className="text-sm text-amber-300">Rule snoozed until {incident.alertRule.snoozedUntil.toLocaleString()}</p>
                )}
                {incident.lastDeliveryError && (
                  <p className="text-sm text-red-300">{incident.lastDeliveryError}</p>
                )}
                <div className="flex flex-wrap gap-2">
                  <Link href={`/app/operations/incidents/${incident.id}`} className="inline-flex text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                    View Details
                  </Link>
                  <ResolveIncidentButton incidentId={incident.id} />
                  <SnoozeAlertRuleButton alertRuleId={incident.alertRule.id} snoozeMinutes={60} label="Snooze 1h" />
                  {incident.alertRule.snoozedUntil && (
                    <SnoozeAlertRuleButton alertRuleId={incident.alertRule.id} snoozeMinutes={null} label="Clear Snooze" />
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
