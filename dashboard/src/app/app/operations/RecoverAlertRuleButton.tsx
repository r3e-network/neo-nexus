'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { recoverAlertRuleAction } from './actions';

export default function RecoverAlertRuleButton({ alertRuleId }: { alertRuleId: number }) {
  const router = useRouter();
  const [isRecovering, setIsRecovering] = useState(false);

  const handleRecover = async () => {
    setIsRecovering(true);
    const result = await recoverAlertRuleAction(alertRuleId);
    if (result.success) {
      toast.success('Alert unsnoozed and queued for immediate retry.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to recover alert.');
    }
    setIsRecovering(false);
  };

  return (
    <button
      onClick={handleRecover}
      disabled={isRecovering}
      className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-3 py-2 rounded-md text-xs font-medium transition-colors"
    >
      {isRecovering ? 'Recovering…' : 'Recover Now'}
    </button>
  );
}
