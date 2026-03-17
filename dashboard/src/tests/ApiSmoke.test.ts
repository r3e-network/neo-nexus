import { beforeEach, describe, expect, it, vi } from 'vitest';
import { GET as getMetrics } from '../app/api/metrics/route';

const mocks = vi.hoisted(() => ({
  endpointFindMany: vi.fn(),
  apiKeyFindMany: vi.fn(),
  nodePluginFindMany: vi.fn(),
  endpointActivityFindMany: vi.fn(),
  getCurrentUserContext: vi.fn(),
}));

vi.mock('@/server/organization', () => ({
  getCurrentUserContext: mocks.getCurrentUserContext,
  isDatabaseConfigured: vi.fn(() => true),
}));

vi.mock('@/utils/prisma', () => ({
  prisma: {
    endpoint: { findMany: mocks.endpointFindMany },
    apiKey: { findMany: mocks.apiKeyFindMany },
    nodePlugin: { findMany: mocks.nodePluginFindMany },
    endpointActivity: { findMany: mocks.endpointActivityFindMany },
  },
}));

describe('API Routes Smoke Test', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getCurrentUserContext.mockResolvedValue({
      userId: 'test_user',
      organizationId: 'test_org',
    });
  });

  it('GET /api/metrics returns structured dashboard analytics payload', async () => {
    // Mock the Prisma responses
    mocks.endpointFindMany.mockResolvedValue([
      { id: 1, name: 'Prod N3', networkKey: 'mainnet', network: 'N3 Mainnet', requests: 15000, status: 'Active', createdAt: new Date() },
      { id: 2, name: 'Test X', networkKey: 'testnet', network: 'Neo X Testnet', requests: 500, status: 'Syncing', createdAt: new Date() },
    ]);
    mocks.apiKeyFindMany.mockResolvedValue([
      { id: 'key1', name: 'Frontend Key', createdAt: new Date() },
    ]);
    mocks.nodePluginFindMany.mockResolvedValue([
      { endpointId: 1, pluginId: 'DBFTPlugin', status: 'Running', createdAt: new Date() },
    ]);
    mocks.endpointActivityFindMany.mockResolvedValue([
      { category: 'provisioning', message: 'Node created', status: 'success', createdAt: new Date() },
    ]);

    // Call the App Router route handler directly (no request param needed for this GET)
    const response = await getMetrics();

    // Verify it returns a standard JSON response
    expect(response.status).toBe(200);
    const data = await response.json();

    // Verify the business logic shape holds together
    expect(data.stats).toBeDefined();
    expect(data.stats.activeEndpoints).toBe(1);
    expect(data.stats.syncingEndpoints).toBe(1);
    expect(data.stats.apiKeys).toBe(1);

    expect(data.endpointUsageData.length).toBe(2);
    expect(data.recentEvents.length).toBeGreaterThan(0);
  });
});
