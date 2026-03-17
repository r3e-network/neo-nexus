import { describe, expect, it, vi } from 'vitest';
import type { AlertRuleDefinition } from './AlertRuleCatalog';
import {
  buildAlertDeliveryAttemptRecord,
  buildAlertDeliveryStateUpdate,
  deliverAlertNotification,
} from './AlertDeliveryService';

const baseRule: AlertRuleDefinition = {
  id: 'endpoint_down',
  name: 'Endpoint Down',
  description: 'Triggers when the endpoint is no longer active.',
  actionTypes: ['email', 'webhook'],
};

describe('AlertDeliveryService', () => {
  it('builds a triggered state transition for a new incident', () => {
    const update = buildAlertDeliveryStateUpdate({
      triggered: true,
      deliverySucceeded: true,
      now: new Date('2026-03-16T10:00:00Z'),
    });

    expect(update.lastTriggeredAt?.toISOString()).toBe('2026-03-16T10:00:00.000Z');
    expect(update.lastDeliveredAt?.toISOString()).toBe('2026-03-16T10:00:00.000Z');
    expect(update.lastDeliveryError).toBeNull();
  });

  it('builds a resolved state transition when an incident clears', () => {
    const update = buildAlertDeliveryStateUpdate({
      triggered: false,
      deliverySucceeded: null,
      now: new Date('2026-03-16T11:00:00Z'),
    });

    expect(update.lastTriggeredAt).toBeNull();
    expect(update.lastResolvedAt?.toISOString()).toBe('2026-03-16T11:00:00.000Z');
  });

  it('delivers webhook notifications with a structured payload', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      text: async () => '',
    });

    await deliverAlertNotification(
      {
        rule: {
          id: 1,
          name: 'Endpoint Down',
          actionType: 'webhook',
          target: 'https://hooks.example.com',
        },
        definition: baseRule,
        evaluation: {
          triggered: true,
          severity: 'critical',
          message: 'Endpoint status is Error.',
        },
        endpoint: {
          id: 5,
          name: 'Mainnet RPC',
          status: 'Error',
        },
      },
      {
        fetchImpl: fetchMock as typeof fetch,
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      'https://hooks.example.com',
      expect.objectContaining({
        method: 'POST',
      }),
    );
  });

  it('builds a persisted delivery attempt payload', () => {
    const now = new Date('2026-03-17T10:00:00Z');
    const attempt = buildAlertDeliveryAttemptRecord({
      alertRuleId: 1,
      endpointId: 5,
      actionType: 'webhook',
      target: 'https://hooks.example.com',
      status: 'failed',
      errorMessage: 'Webhook delivery failed',
      now,
    });

    expect(attempt).toEqual({
      alertRuleId: 1,
      endpointId: 5,
      actionType: 'webhook',
      target: 'https://hooks.example.com',
      status: 'failed',
      errorMessage: 'Webhook delivery failed',
      createdAt: now,
    });
  });
});
