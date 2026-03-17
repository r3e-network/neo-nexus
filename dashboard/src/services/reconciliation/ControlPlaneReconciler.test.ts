import { describe, expect, it, vi } from 'vitest';
import { reconcileControlPlane } from './ControlPlaneReconciler';

describe('ControlPlaneReconciler', () => {
  it('summarizes resumed orders and evaluated endpoints', async () => {
    const result = await reconcileControlPlane(
      {
        loadPendingOrderIds: async () => [1, 2],
        resumeOrders: async () => ({ checked: 2, resumed: 1, resumedOrderIds: [1] }),
        loadAlertEndpointIds: async () => [5, 6, 7],
        evaluateEndpointAlerts: vi.fn().mockResolvedValue({ rules: [], incidents: [] }),
      },
    );

    expect(result.provisioning.checked).toBe(2);
    expect(result.provisioning.resumed).toBe(1);
    expect(result.provisioning.resumedOrderIds).toEqual([1]);
    expect(result.alerting.endpointsEvaluated).toBe(3);
    expect(result.alerting.endpointIds).toEqual([5, 6, 7]);
  });
});
