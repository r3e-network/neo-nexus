import { describe, expect, it } from 'vitest';
import { buildOperationsSummary } from './OperationsSummaryService';

describe('OperationsSummaryService', () => {
  it('builds counts for pending provisioning and alert retries', () => {
    const summary = buildOperationsSummary({
      pendingOrders: [
        { nextAttemptAt: '2026-03-16T12:00:00.000Z' },
        { nextAttemptAt: null },
      ],
      alertRetries: [
        { nextDeliveryAt: '2026-03-16T12:05:00.000Z' },
      ],
      openIncidentsCount: 3,
      latestSuccessfulRunAt: '2026-03-16T11:00:00.000Z',
      latestFailedRunAt: null,
      latestScheduledSuccessAt: '2026-03-16T11:55:00.000Z',
      latestScheduledFailureAt: null,
      now: new Date('2026-03-16T12:00:00Z'),
    });

    expect(summary.pendingProvisioningCount).toBe(2);
    expect(summary.alertRetriesCount).toBe(1);
    expect(summary.openIncidentsCount).toBe(3);
    expect(summary.latestSuccessfulRunAt).toBe('2026-03-16T11:00:00.000Z');
    expect(summary.schedulerIsStale).toBe(false);
  });

  it('counts past-due retries as due and marks recent scheduled failures as degraded', () => {
    const summary = buildOperationsSummary({
      pendingOrders: [
        { nextAttemptAt: '2026-03-16T11:50:00.000Z' },
      ],
      alertRetries: [],
      openIncidentsCount: 0,
      latestSuccessfulRunAt: null,
      latestFailedRunAt: null,
      latestScheduledSuccessAt: '2026-03-16T11:40:00.000Z',
      latestScheduledFailureAt: '2026-03-16T11:59:00.000Z',
      latestScheduledRunAt: '2026-03-16T11:59:00.000Z',
      latestScheduledRunStatus: 'error',
      now: new Date('2026-03-16T12:00:00.000Z'),
    });

    expect(summary.dueProvisioningCount).toBe(1);
    expect(summary.schedulerStatus).toBe('degraded');
    expect(summary.schedulerIsStale).toBe(false);
  });
});
