'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { retryProvisioningOrderAction } from './actions';

export default function RetryProvisioningQueueButton({ orderId }: { orderId: number }) {
  const router = useRouter();
  const [isRetrying, setIsRetrying] = useState(false);

  const handleRetry = async () => {
    setIsRetrying(true);
    const result = await retryProvisioningOrderAction(orderId);
    if (result.success) {
      toast.success('Provisioning retry requested.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to retry provisioning order.');
    }
    setIsRetrying(false);
  };

  return (
    <button
      onClick={handleRetry}
      disabled={isRetrying}
      className="text-xs text-[#00E599] hover:text-[#00cc88] disabled:opacity-50 font-medium"
    >
      {isRetrying ? 'Retrying…' : 'Retry Now'}
    </button>
  );
}
