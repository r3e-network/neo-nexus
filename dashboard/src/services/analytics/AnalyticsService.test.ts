import { describe, expect, it } from 'vitest';
import { buildAnalyticsSnapshot } from './AnalyticsService';

describe('AnalyticsService', () => {
  it('builds truthful organization analytics from stored resources', () => {
    const snapshot = buildAnalyticsSnapshot({
      endpoints: [
        {
          id: 1,
          name: 'Mainnet RPC',
          network: 'N3 Mainnet',
          status: 'Active',
          requests: 1200,
          createdAt: new Date('2025-03-10T10:00:00Z'),
        },
        {
          id: 2,
          name: 'Backup RPC',
          network: 'N3 Mainnet',
          status: 'Syncing',
          requests: 300,
          createdAt: new Date('2025-03-12T12:00:00Z'),
        },
      ],
      apiKeys: [
        { id: 'key-1', name: 'Primary Key', createdAt: new Date('2025-03-11T09:00:00Z') },
      ],
      plugins: [
        { id: 9, pluginId: 'tee-oracle', endpointId: 1, status: 'Active', createdAt: new Date('2025-03-13T08:00:00Z') },
      ],
      activities: [],
    });

    expect(snapshot.stats.totalRequests).toBe('1.5K');
    expect(snapshot.stats.activeEndpoints).toBe(1);
    expect(snapshot.stats.syncingEndpoints).toBe(1);
    expect(snapshot.stats.apiKeys).toBe(1);
    expect(snapshot.endpointUsageData[0]).toMatchObject({ name: 'Mainnet RPC', requests: 1200 });
    expect(snapshot.networkData[0]).toMatchObject({ name: 'N3 Mainnet', requests: 1500, endpoints: 2 });
    expect(snapshot.recentEvents[0]?.category).toBe('Plugin');
  });

  it('prefers persisted endpoint activities for recent events when available', () => {
    const snapshot = buildAnalyticsSnapshot({
      endpoints: [],
      apiKeys: [],
      plugins: [],
      activities: [
        {
          id: 44,
          category: 'plugin',
          action: 'install',
          status: 'success',
          message: 'TEE Oracle applied on the managed node.',
          createdAt: new Date('2025-03-13T08:00:00Z'),
        },
      ],
    });

    expect(snapshot.recentEvents).toEqual([
      {
        time: '2025-03-13T08:00:00.000Z',
        category: 'Plugin',
        detail: 'TEE Oracle applied on the managed node.',
        status: 'success',
      },
    ]);
  });
});
