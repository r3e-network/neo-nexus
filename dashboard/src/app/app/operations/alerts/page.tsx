import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { clampOperationsItemsLimit } from '@/services/reconciliation/OperationsDrilldownWindow';
import {
  buildAlertQueueWhere,
  buildOperationsQueueHref,
  normalizeOperationsEnumFilter,
  normalizeOperationsSearchQuery,
} from '@/services/reconciliation/OperationsQueueFilters';
import RecoverAlertRulesButton from '../RecoverAlertRulesButton';
import RecoverAlertRuleButton from '../RecoverAlertRuleButton';
import RetryAlertQueueButton from '../RetryAlertQueueButton';
import SnoozeAlertRuleButton from '../SnoozeAlertRuleButton';

export const dynamic = 'force-dynamic';

export default async function AlertRetryQueuePage(
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
  const snoozed = normalizeOperationsEnumFilter(
    typeof params.snoozed === 'string' ? params.snoozed : undefined,
    ['snoozed', 'unsnoozed'] as const,
  );
  const actionType = normalizeOperationsEnumFilter(
    typeof params.actionType === 'string' ? params.actionType : undefined,
    ['email', 'webhook'] as const,
  );
  const timing = normalizeOperationsEnumFilter(
    typeof params.timing === 'string' ? params.timing : undefined,
    ['due', 'scheduled'] as const,
  );
  const now = new Date();

  const rules = await prisma.alertRule.findMany({
    where: buildAlertQueueWhere({
      query,
      snoozed,
      actionType,
      timing,
      now,
    }),
    orderBy: { nextDeliveryAt: 'asc' },
    take: limit,
    select: {
      id: true,
      name: true,
      endpointId: true,
      actionType: true,
      target: true,
      nextDeliveryAt: true,
      snoozedUntil: true,
      deliveryAttemptCount: true,
      lastDeliveryError: true,
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
        <h1 className="text-2xl font-bold tracking-tight mt-3">Alert Delivery Retry Queue</h1>
        <p className="text-gray-400 mt-1">Showing up to {limit} alert rules waiting for another delivery attempt.</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', {})}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            snoozed === 'all' && actionType === 'all' && timing === 'all' && !query
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          All
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { timing: 'due' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            timing === 'due'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Due Now
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { timing: 'scheduled' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            timing === 'scheduled'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Scheduled
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { actionType: 'webhook' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            actionType === 'webhook'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Webhook
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { actionType: 'email' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            actionType === 'email'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Email
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { snoozed: 'snoozed' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            snoozed === 'snoozed'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Snoozed
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/alerts', { snoozed: 'unsnoozed' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            snoozed === 'unsnoozed'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Unsnoozed
        </Link>
      </div>
      <form className="grid grid-cols-1 md:grid-cols-6 gap-3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
        <input
          type="text"
          name="q"
          defaultValue={query ?? ''}
          placeholder="Search org, endpoint, rule, or target"
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        />
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
          name="actionType"
          defaultValue={actionType}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All delivery types</option>
          <option value="email">Email</option>
          <option value="webhook">Webhook</option>
        </select>
        <select
          name="timing"
          defaultValue={timing}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All retry timing</option>
          <option value="due">Due now</option>
          <option value="scheduled">Scheduled later</option>
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
          <Link href="/app/operations/alerts" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors">
            Reset
          </Link>
        </div>
      </form>
      <div className="flex justify-end">
        <RecoverAlertRulesButton alertRuleIds={rules.map((rule) => rule.id)} />
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden">
        {rules.length === 0 ? (
          <div className="p-6 text-sm text-gray-400">No pending alert delivery retries.</div>
        ) : (
          <div className="divide-y divide-[var(--color-dark-border)]">
            {rules.map((rule) => (
              <div key={rule.id} className="p-6 space-y-3">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <Link href={`/app/endpoints/${rule.endpoint.id}`} className="text-base font-medium text-white hover:text-[#00E599]">
                      {rule.endpoint.name}
                    </Link>
                    <p className="text-xs text-gray-500 mt-1">
                      Org: {rule.endpoint.organization?.name ?? 'Unknown'} • Rule: {rule.name}
                    </p>
                  </div>
                  <span className="text-xs text-gray-500">Rule #{rule.id}</span>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-300">
                  <div>Attempts: {rule.deliveryAttemptCount}</div>
                  <div>Next retry: {rule.nextDeliveryAt?.toLocaleString() ?? 'Immediate'}</div>
                </div>
                <div className="text-xs text-gray-500">Action: {rule.actionType} • Target: {rule.target}</div>
                {rule.snoozedUntil && (
                  <p className="text-sm text-amber-300">Snoozed until {rule.snoozedUntil.toLocaleString()}</p>
                )}
                {rule.lastDeliveryError && (
                  <p className="text-sm text-red-300">{rule.lastDeliveryError}</p>
                )}
                <Link href={`/app/operations/alerts/${rule.id}`} className="inline-flex text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                  View Details
                </Link>
                <div className="flex flex-wrap gap-2">
                  <RecoverAlertRuleButton alertRuleId={rule.id} />
                  <RetryAlertQueueButton alertRuleId={rule.id} />
                  <SnoozeAlertRuleButton alertRuleId={rule.id} snoozeMinutes={60} label="Snooze 1h" />
                  {rule.snoozedUntil && (
                    <SnoozeAlertRuleButton alertRuleId={rule.id} snoozeMinutes={null} label="Clear Snooze" />
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
