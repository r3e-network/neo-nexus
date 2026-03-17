'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { snoozeAlertRuleAction } from './actions';

type SnoozeAlertRuleButtonProps = {
  alertRuleId: number;
  snoozeMinutes: number | null;
  label: string;
};

export default function SnoozeAlertRuleButton({
  alertRuleId,
  snoozeMinutes,
  label,
}: SnoozeAlertRuleButtonProps) {
  const router = useRouter();
  const [isSaving, setIsSaving] = useState(false);

  const handleClick = async () => {
    setIsSaving(true);
    const result = await snoozeAlertRuleAction(alertRuleId, snoozeMinutes);
    if (result.success) {
      toast.success(snoozeMinutes ? `Alert snoozed for ${snoozeMinutes} minutes.` : 'Alert snooze cleared.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to update alert snooze.');
    }
    setIsSaving(false);
  };

  return (
    <button
      onClick={handleClick}
      disabled={isSaving}
      className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] disabled:opacity-50 text-white px-3 py-2 rounded-md text-xs font-medium transition-colors"
    >
      {isSaving ? 'Saving…' : label}
    </button>
  );
}
