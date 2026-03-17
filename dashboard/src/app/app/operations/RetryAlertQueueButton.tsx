'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { retryAlertDeliveryAction } from './actions';

export default function RetryAlertQueueButton({ alertRuleId }: { alertRuleId: number }) {
  const router = useRouter();
  const [isRetrying, setIsRetrying] = useState(false);

  const handleRetry = async () => {
    setIsRetrying(true);
    const result = await retryAlertDeliveryAction(alertRuleId);
    if (result.success) {
      toast.success('Alert delivery retry requested.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to retry alert delivery.');
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
