'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { recoverAlertRulesAction } from './actions';

export default function RecoverAlertRulesButton({ alertRuleIds }: { alertRuleIds: number[] }) {
  const router = useRouter();
  const [isRecovering, setIsRecovering] = useState(false);

  const handleRecover = async () => {
    setIsRecovering(true);
    const result = await recoverAlertRulesAction(alertRuleIds);
    if (result.success) {
      toast.success(`Recovered ${result.recoveredCount} alert rule${result.recoveredCount === 1 ? '' : 's'}.`);
      router.refresh();
    } else {
      toast.error('Failed to recover alert rules.');
    }
    setIsRecovering(false);
  };

  return (
    <button
      onClick={handleRecover}
      disabled={isRecovering || alertRuleIds.length === 0}
      className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
    >
      {isRecovering ? 'Recovering…' : 'Recover All Visible'}
    </button>
  );
}
