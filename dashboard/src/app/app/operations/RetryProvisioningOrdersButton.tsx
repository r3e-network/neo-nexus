'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { retryProvisioningOrdersAction } from './actions';

export default function RetryProvisioningOrdersButton({ orderIds }: { orderIds: number[] }) {
  const router = useRouter();
  const [isRetrying, setIsRetrying] = useState(false);

  const handleRetry = async () => {
    setIsRetrying(true);
    const result = await retryProvisioningOrdersAction(orderIds);
    if (result.success) {
      toast.success(`Retried ${result.retriedCount} provisioning order${result.retriedCount === 1 ? '' : 's'}.`);
      router.refresh();
    } else {
      toast.error('Failed to retry provisioning orders.');
    }
    setIsRetrying(false);
  };

  return (
    <button
      onClick={handleRetry}
      disabled={isRetrying || orderIds.length === 0}
      className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
    >
      {isRetrying ? 'Retrying…' : 'Retry All Visible'}
    </button>
  );
}
