export function buildAlertRuleSnoozeUpdate(input: {
  now: Date;
  snoozeMinutes: number | null;
  hasOpenAlert?: boolean;
}) {
  if (input.snoozeMinutes && input.snoozeMinutes > 0) {
    const snoozedUntil = new Date(input.now.getTime() + input.snoozeMinutes * 60_000);
    return {
      snoozedUntil,
      nextDeliveryAt: snoozedUntil,
    };
  }

  return {
    snoozedUntil: null,
    nextDeliveryAt: input.hasOpenAlert ? new Date(0) : null,
  };
}
