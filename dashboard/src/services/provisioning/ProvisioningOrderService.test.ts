import { describe, expect, it } from 'vitest';
import { deriveProvisioningOrderStatus } from './ProvisioningOrderService';

describe('ProvisioningOrderService', () => {
  it('upgrades syncing orders to ready when the endpoint becomes active', () => {
    expect(
      deriveProvisioningOrderStatus({
        orderStatus: 'syncing',
        endpointStatus: 'Active',
      }),
    ).toEqual({
      status: 'ready',
      currentStep: 'ready',
    });
  });

  it('preserves failed status regardless of endpoint data', () => {
    expect(
      deriveProvisioningOrderStatus({
        orderStatus: 'failed',
        endpointStatus: 'Active',
      }),
    ).toEqual({
      status: 'failed',
      currentStep: 'failed',
    });
  });
});
