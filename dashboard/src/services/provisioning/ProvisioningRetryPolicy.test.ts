import { describe, expect, it } from 'vitest';
import {
  MAX_PROVISIONING_ATTEMPTS,
  buildProvisioningFailureUpdate,
} from './ProvisioningRetryPolicy';

describe('ProvisioningRetryPolicy', () => {
  it('schedules a retry before the max attempt threshold', () => {
    const now = new Date('2026-03-16T12:00:00Z');
    const update = buildProvisioningFailureUpdate({
      attemptCount: 1,
      errorMessage: 'provider unavailable',
      now,
    });

    expect(update.status).toBe('pending');
    expect(update.currentStep).toBe('pending');
    expect(update.nextAttemptAt?.toISOString()).toBe('2026-03-16T12:01:00.000Z');
  });

  it('marks the order as failed once max attempts are exceeded', () => {
    const now = new Date('2026-03-16T12:00:00Z');
    const update = buildProvisioningFailureUpdate({
      attemptCount: MAX_PROVISIONING_ATTEMPTS,
      errorMessage: 'permanent failure',
      now,
    });

    expect(update.status).toBe('failed');
    expect(update.nextAttemptAt).toBeNull();
  });
});
