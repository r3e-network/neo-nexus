import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  apiKeyCount: vi.fn(),
  apiKeyCreate: vi.fn(),
  apiKeyDelete: vi.fn(),
  firewallCreate: vi.fn(),
  firewallDelete: vi.fn(),
  transaction: vi.fn(),
  requireCurrentOrganizationContext: vi.fn(),
  createConsumer: vi.fn(),
  listOrganizationFirewallRules: vi.fn(),
  syncOrganizationRoutes: vi.fn(),
  normalizeFirewallRuleValue: vi.fn(),
  getMethodFirewallPreset: vi.fn(),
  getErrorMessage: vi.fn((error: unknown) => error instanceof Error ? error.message : 'Unknown error'),
}));

vi.mock('@/utils/prisma', () => ({
  prisma: {
    $transaction: mocks.transaction,
    apiKey: {
      count: mocks.apiKeyCount,
      create: mocks.apiKeyCreate,
      delete: mocks.apiKeyDelete,
    },
    firewall: {
      create: mocks.firewallCreate,
      delete: mocks.firewallDelete,
    },
  },
}));

vi.mock('@/server/organization', () => ({
  assertDatabaseConfigured: vi.fn(),
  requireCurrentOrganizationContext: mocks.requireCurrentOrganizationContext,
}));

vi.mock('@/services/apisix/ApisixService', () => ({
  ApisixService: {
    createConsumer: mocks.createConsumer,
    deleteConsumer: vi.fn(),
  },
}));

vi.mock('@/server/errors', () => ({
  getErrorMessage: mocks.getErrorMessage,
}));

vi.mock('@/services/security/FirewallPolicy', () => ({
  normalizeFirewallRuleValue: mocks.normalizeFirewallRuleValue,
}));

vi.mock('@/services/security/MethodFirewallPresets', () => ({
  getMethodFirewallPreset: mocks.getMethodFirewallPreset,
}));

vi.mock('@/services/security/RouteSecuritySync', () => ({
  listOrganizationFirewallRules: mocks.listOrganizationFirewallRules,
  syncOrganizationRoutes: mocks.syncOrganizationRoutes,
}));

vi.mock('next/cache', () => ({
  revalidatePath: vi.fn(),
}));

import {
  createApiKeyAction,
  createFirewallRuleAction,
  deleteFirewallRuleAction,
  setMethodFirewallPresetAction,
} from './actions';

describe('security actions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.requireCurrentOrganizationContext.mockResolvedValue({
      userId: 'user_123',
      organizationId: 'org_123',
      billingPlan: 'growth',
      role: 'member',
    });
    mocks.apiKeyCount.mockResolvedValue(0);
    mocks.apiKeyCreate.mockResolvedValue({
      id: 'key_123',
      name: 'Server Key',
      keyHash: 'hashed',
      isActive: true,
      createdAt: new Date('2026-03-17T00:00:00.000Z'),
    });
    mocks.normalizeFirewallRuleValue.mockImplementation((_type: string, value: string) => value.trim());
    mocks.listOrganizationFirewallRules.mockResolvedValue([
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
    ]);
    mocks.syncOrganizationRoutes.mockResolvedValue(undefined);
    mocks.transaction.mockResolvedValue(undefined);
    mocks.getMethodFirewallPreset.mockReturnValue({
      id: 'write_methods',
      methods: ['sendrawtransaction'],
    });
  });

  it('rolls back the created api key when APISIX consumer creation throws', async () => {
    mocks.createConsumer.mockRejectedValue(new Error('APISIX unavailable'));

    const result = await createApiKeyAction('Server Key');

    expect(result).toEqual({
      success: false,
      error: 'APISIX unavailable',
    });
    expect(mocks.apiKeyDelete).toHaveBeenCalledWith({
      where: { id: 'key_123' },
    });
  });

  it('restores the previous firewall rules when db persistence fails after route sync', async () => {
    mocks.firewallCreate.mockRejectedValue(new Error('db write failed'));

    const result = await createFirewallRuleAction('origin_allow', 'https://app.example.com');

    expect(result).toEqual({
      success: false,
      error: 'db write failed',
    });
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(1, 'org_123', [
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
      {
        type: 'origin_allow',
        value: 'https://app.example.com',
      },
    ]);
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(2, 'org_123', [
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
    ]);
  });

  it('restores the previous firewall rules when db deletion fails after route sync', async () => {
    mocks.firewallDelete.mockRejectedValue(new Error('db delete failed'));

    const result = await deleteFirewallRuleAction(10);

    expect(result).toEqual({
      success: false,
      error: 'db delete failed',
    });
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(1, 'org_123', []);
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(2, 'org_123', [
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
    ]);
  });

  it('restores the previous firewall rules when method preset persistence fails after route sync', async () => {
    mocks.transaction.mockRejectedValue(new Error('preset write failed'));

    const result = await setMethodFirewallPresetAction('write_methods', true);

    expect(result).toEqual({
      success: false,
      error: 'preset write failed',
    });
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(1, 'org_123', [
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
      {
        type: 'method_block',
        value: 'sendrawtransaction',
      },
    ]);
    expect(mocks.syncOrganizationRoutes).toHaveBeenNthCalledWith(2, 'org_123', [
      {
        id: 10,
        type: 'ip_allow',
        value: '203.0.113.10',
      },
    ]);
  });
});
