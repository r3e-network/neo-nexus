import { describe, expect, it } from 'vitest';
import {
  buildAlertQueueWhere,
  buildIncidentQueueWhere,
  buildProvisioningQueueWhere,
  buildOperationsQueueHref,
  normalizeOperationsEnumFilter,
  normalizeOperationsSearchQuery,
} from './OperationsQueueFilters';

describe('OperationsQueueFilters', () => {
  it('normalizes empty search queries to null', () => {
    expect(normalizeOperationsSearchQuery(undefined)).toBeNull();
    expect(normalizeOperationsSearchQuery('   ')).toBeNull();
    expect(normalizeOperationsSearchQuery('mainnet rpc')).toBe('mainnet rpc');
  });

  it('normalizes enum filters against an allowed set', () => {
    expect(normalizeOperationsEnumFilter('critical', ['critical', 'warning'] as const)).toBe('critical');
    expect(normalizeOperationsEnumFilter('bad', ['critical', 'warning'] as const)).toBe('all');
    expect(normalizeOperationsEnumFilter(undefined, ['critical', 'warning'] as const)).toBe('all');
  });

  it('builds stable queue hrefs without empty or default filters', () => {
    expect(buildOperationsQueueHref('/app/operations/provisioning', {
      provider: 'hetzner',
      timing: 'all',
      limit: 50,
      q: 'mainnet',
      empty: '',
    })).toBe('/app/operations/provisioning?limit=50&provider=hetzner&q=mainnet');
  });

  it('builds provisioning queue filters with due timing and search in the same AND clause', () => {
    const now = new Date('2026-03-17T00:00:00.000Z');

    expect(buildProvisioningQueueWhere({
      provider: 'hetzner',
      query: 'acme',
      timing: 'due',
      now,
    })).toEqual({
      AND: [
        { status: { notIn: ['ready', 'failed'] } },
        { provider: 'hetzner' },
        {
          OR: [
            { nextAttemptAt: null },
            { nextAttemptAt: { lte: now } },
          ],
        },
        {
          OR: [
            { endpoint: { name: { contains: 'acme', mode: 'insensitive' } } },
            { organization: { name: { contains: 'acme', mode: 'insensitive' } } },
          ],
        },
      ],
    });
  });

  it('keeps alert snooze filters and search filters together instead of overwriting one OR with another', () => {
    const now = new Date('2026-03-17T00:00:00.000Z');

    expect(buildAlertQueueWhere({
      query: 'ops@example.com',
      snoozed: 'unsnoozed',
      actionType: 'webhook',
      timing: 'scheduled',
      now,
    })).toEqual({
      AND: [
        {
          isActive: true,
          nextDeliveryAt: { not: null },
        },
        {
          OR: [
            { snoozedUntil: null },
            { snoozedUntil: { lte: now } },
          ],
        },
        { actionType: 'webhook' },
        { nextDeliveryAt: { gt: now } },
        {
          OR: [
            { name: { contains: 'ops@example.com', mode: 'insensitive' } },
            { target: { contains: 'ops@example.com', mode: 'insensitive' } },
            { endpoint: { name: { contains: 'ops@example.com', mode: 'insensitive' } } },
            { endpoint: { organization: { name: { contains: 'ops@example.com', mode: 'insensitive' } } } },
          ],
        },
      ],
    });
  });

  it('keeps incident snooze filters and search filters together instead of overwriting one OR with another', () => {
    const now = new Date('2026-03-17T00:00:00.000Z');

    expect(buildIncidentQueueWhere({
      query: 'rpc degraded',
      severity: 'critical',
      snoozed: 'unsnoozed',
      now,
    })).toEqual({
      AND: [
        { status: 'Open' },
        { severity: 'critical' },
        {
          OR: [
            { alertRule: { snoozedUntil: null } },
            { alertRule: { snoozedUntil: { lte: now } } },
          ],
        },
        {
          OR: [
            { message: { contains: 'rpc degraded', mode: 'insensitive' } },
            { endpoint: { name: { contains: 'rpc degraded', mode: 'insensitive' } } },
            { endpoint: { organization: { name: { contains: 'rpc degraded', mode: 'insensitive' } } } },
            { alertRule: { name: { contains: 'rpc degraded', mode: 'insensitive' } } },
          ],
        },
      ],
    });
  });
});
