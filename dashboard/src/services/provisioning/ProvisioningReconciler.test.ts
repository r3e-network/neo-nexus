import { describe, expect, it, vi } from 'vitest';
import { resumePendingProvisioningOrders } from './ProvisioningReconciler';

describe('ProvisioningReconciler', () => {
  it('resumes only non-terminal orders', async () => {
    const kickoff = vi
      .fn()
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(false);

    const result = await resumePendingProvisioningOrders(
      [1, 2],
      kickoff,
    );

    expect(kickoff).toHaveBeenCalledTimes(2);
    expect(result.resumed).toBe(1);
    expect(result.checked).toBe(2);
    expect(result.resumedOrderIds).toEqual([1]);
  });
});
