import { describe, expect, it } from 'vitest';
import {
  MAX_ALERT_DELIVERY_ATTEMPTS,
  buildAlertDeliveryRetryUpdate,
} from './AlertDeliveryRetryPolicy';

describe('AlertDeliveryRetryPolicy', () => {
  it('schedules a retry after a failed alert delivery', () => {
    const now = new Date('2026-03-16T12:00:00Z');
    const update = buildAlertDeliveryRetryUpdate({
      now,
      deliverySucceeded: false,
      currentAttemptCount: 0,
      lastTriggeredAt: now,
      errorMessage: 'webhook failed',
    });

    expect(update.deliveryAttemptCount).toBe(1);
    expect(update.nextDeliveryAt?.toISOString()).toBe('2026-03-16T12:01:00.000Z');
    expect(update.lastDeliveryError).toBe('webhook failed');
  });

  it('stops scheduling retries after the max attempt count', () => {
    const now = new Date('2026-03-16T12:00:00Z');
    const update = buildAlertDeliveryRetryUpdate({
      now,
      deliverySucceeded: false,
      currentAttemptCount: MAX_ALERT_DELIVERY_ATTEMPTS - 1,
      lastTriggeredAt: now,
      errorMessage: 'hard failure',
    });

    expect(update.deliveryAttemptCount).toBe(MAX_ALERT_DELIVERY_ATTEMPTS);
    expect(update.nextDeliveryAt).toBeNull();
  });
});
