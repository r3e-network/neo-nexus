export type ProvisioningOrderStatus =
  | 'pending'
  | 'vm_creating'
  | 'software_installing'
  | 'syncing'
  | 'ready'
  | 'failed';

type DeriveStatusInput = {
  orderStatus: ProvisioningOrderStatus;
  endpointStatus?: string | null;
};

export function deriveProvisioningOrderStatus({ orderStatus, endpointStatus }: DeriveStatusInput) {
  if (orderStatus === 'failed' || orderStatus === 'ready') {
    return {
      status: orderStatus,
      currentStep: orderStatus,
    };
  }

  if (endpointStatus === 'Active') {
    return {
      status: 'ready' as const,
      currentStep: 'ready',
    };
  }

  return {
    status: orderStatus,
    currentStep: orderStatus,
  };
}
