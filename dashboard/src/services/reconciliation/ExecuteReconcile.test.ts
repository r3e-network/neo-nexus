import { describe, expect, it, vi } from 'vitest';
import { executeAndRecordReconcile } from './ExecuteReconcile';

describe('ExecuteReconcile', () => {
  it('records a successful reconcile run', async () => {
    const reconcile = vi.fn().mockResolvedValue({
      provisioning: { checked: 2, resumed: 1 },
      alerting: { endpointsEvaluated: 3 },
      reconciledAt: '2026-03-16T12:00:00.000Z',
    });
    const record = vi.fn().mockResolvedValue({ id: 9 });

    const result = await executeAndRecordReconcile('manual', reconcile, record);

    expect(record).toHaveBeenCalledWith({
      source: 'manual',
      status: 'success',
      provisioningChecked: 2,
      provisioningResumed: 1,
      alertingEndpointsEvaluated: 3,
      metadata: {
        resumedOrderIds: undefined,
        alertEndpointIds: undefined,
      },
    });
    expect(result.runId).toBe(9);
  });

  it('records a failed reconcile run', async () => {
    const reconcile = vi.fn().mockRejectedValue(new Error('boom'));
    const record = vi.fn().mockResolvedValue({ id: 10 });

    const result = await executeAndRecordReconcile('scheduled', reconcile, record);

    expect(record).toHaveBeenCalledWith({
      source: 'scheduled',
      status: 'error',
      provisioningChecked: 0,
      provisioningResumed: 0,
      alertingEndpointsEvaluated: 0,
      errorMessage: 'boom',
    });
    expect(result.ok).toBe(false);
    expect(result.runId).toBe(10);
  });
});
