import { describe, expect, it } from 'vitest';
import { toReconcileRunView } from './ReconcileRunService';

describe('ReconcileRunService', () => {
  it('serializes reconcile run rows for API responses', () => {
    const view = toReconcileRunView({
      id: 1,
      source: 'manual',
      status: 'success',
      provisioningChecked: 4,
      provisioningResumed: 2,
      alertingEndpointsEvaluated: 3,
      metadata: null,
      errorMessage: null,
      createdAt: new Date('2026-03-16T13:00:00Z'),
    });

    expect(view).toEqual({
      id: 1,
      source: 'manual',
      status: 'success',
      provisioningChecked: 4,
      provisioningResumed: 2,
      alertingEndpointsEvaluated: 3,
      metadata: null,
      errorMessage: null,
      createdAt: '2026-03-16T13:00:00.000Z',
    });
  });
});
