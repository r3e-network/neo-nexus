import { describe, expect, it } from 'vitest';
import { buildAlertRuleSnoozeUpdate } from './AlertRuleSnooze';

describe('AlertRuleSnooze', () => {
  it('builds a snooze update and defers the next delivery time', () => {
    const now = new Date('2026-03-17T12:00:00.000Z');
    const update = buildAlertRuleSnoozeUpdate({
      now,
      snoozeMinutes: 60,
    });

    expect(update.snoozedUntil?.toISOString()).toBe('2026-03-17T13:00:00.000Z');
    expect(update.nextDeliveryAt?.toISOString()).toBe('2026-03-17T13:00:00.000Z');
  });

  it('clears the snooze and schedules immediate reevaluation for open alerts', () => {
    const update = buildAlertRuleSnoozeUpdate({
      now: new Date('2026-03-17T12:00:00.000Z'),
      snoozeMinutes: null,
      hasOpenAlert: true,
    });

    expect(update.snoozedUntil).toBeNull();
    expect(update.nextDeliveryAt?.toISOString()).toBe('1970-01-01T00:00:00.000Z');
  });
});
