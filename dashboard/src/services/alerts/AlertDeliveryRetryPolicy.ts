export const MAX_ALERT_DELIVERY_ATTEMPTS = 5;

function getRetryDelayMs(attemptCount: number) {
  const minutes = Math.min(2 ** Math.max(attemptCount - 1, 0), 30);
  return minutes * 60_000;
}

export function buildAlertDeliveryRetryUpdate(input: {
  now: Date;
  deliverySucceeded: boolean | null;
  currentAttemptCount: number;
  lastTriggeredAt: Date | null;
  errorMessage?: string | null;
}) {
  if (input.deliverySucceeded === null) {
    return {
      deliveryAttemptCount: 0,
      nextDeliveryAt: null,
      lastTriggeredAt: null,
      lastDeliveredAt: null,
      lastResolvedAt: input.now,
      lastDeliveryError: null,
    };
  }

  if (input.deliverySucceeded) {
    return {
      deliveryAttemptCount: 0,
      nextDeliveryAt: null,
      lastTriggeredAt: input.lastTriggeredAt ?? input.now,
      lastDeliveredAt: input.now,
      lastResolvedAt: null,
      lastDeliveryError: null,
    };
  }

  const nextAttemptCount = input.currentAttemptCount + 1;

  return {
    deliveryAttemptCount: nextAttemptCount,
    nextDeliveryAt: nextAttemptCount >= MAX_ALERT_DELIVERY_ATTEMPTS
      ? null
      : new Date(input.now.getTime() + getRetryDelayMs(nextAttemptCount)),
    lastTriggeredAt: input.lastTriggeredAt ?? input.now,
    lastDeliveredAt: null,
    lastResolvedAt: null,
    lastDeliveryError: input.errorMessage ?? 'Unknown alert delivery error.',
  };
}
