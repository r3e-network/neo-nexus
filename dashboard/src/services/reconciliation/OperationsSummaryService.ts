type PendingOrderLike = {
  nextAttemptAt: string | null;
};

type AlertRetryLike = {
  nextDeliveryAt: string | null;
};

export function buildOperationsSummary(input: {
  pendingOrders: PendingOrderLike[];
  alertRetries: AlertRetryLike[];
  openIncidentsCount: number;
  latestSuccessfulRunAt: string | null;
  latestFailedRunAt: string | null;
  latestScheduledSuccessAt: string | null;
  latestScheduledFailureAt: string | null;
  latestScheduledRunAt?: string | null;
  latestScheduledRunStatus?: 'success' | 'error' | null;
  now: Date;
  schedulerStaleThresholdMinutes?: number;
}) {
  const latestScheduledRunAt = input.latestScheduledRunAt
    ?? [input.latestScheduledSuccessAt, input.latestScheduledFailureAt]
      .filter((value): value is string => Boolean(value))
      .sort((left, right) => new Date(right).getTime() - new Date(left).getTime())[0]
    ?? null;
  const latestScheduledRunStatus = input.latestScheduledRunStatus
    ?? (
      latestScheduledRunAt && latestScheduledRunAt === input.latestScheduledFailureAt
        ? 'error'
        : latestScheduledRunAt
          ? 'success'
          : null
    );
  const staleThresholdMs = (input.schedulerStaleThresholdMinutes ?? 10) * 60_000;
  const schedulerIsStale = !latestScheduledRunAt
    || (input.now.getTime() - new Date(latestScheduledRunAt).getTime()) > staleThresholdMs;
  const nowMs = input.now.getTime();
  const dueProvisioningCount = input.pendingOrders.filter((order) => (
    order.nextAttemptAt === null || new Date(order.nextAttemptAt).getTime() <= nowMs
  )).length;
  const schedulerStatus = schedulerIsStale
    ? 'stale'
    : latestScheduledRunStatus === 'error'
      ? 'degraded'
      : 'healthy';

  return {
    pendingProvisioningCount: input.pendingOrders.length,
    dueProvisioningCount,
    alertRetriesCount: input.alertRetries.length,
    openIncidentsCount: input.openIncidentsCount,
    latestSuccessfulRunAt: input.latestSuccessfulRunAt,
    latestFailedRunAt: input.latestFailedRunAt,
    latestScheduledRunAt,
    latestScheduledRunStatus,
    latestScheduledSuccessAt: input.latestScheduledSuccessAt,
    latestScheduledFailureAt: input.latestScheduledFailureAt,
    schedulerStatus,
    schedulerIsStale,
  };
}
