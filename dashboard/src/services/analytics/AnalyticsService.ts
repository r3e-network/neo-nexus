function formatCompactNumber(value: number): string {
  return new Intl.NumberFormat('en-US', {
    notation: 'compact',
    maximumFractionDigits: value >= 1000 ? 1 : 0,
  }).format(value);
}

export type AnalyticsEndpointRecord = {
  id: number;
  name: string;
  network: string;
  status: string;
  requests: number;
  createdAt: Date;
};

export type AnalyticsApiKeyRecord = {
  id: string;
  name: string;
  createdAt: Date;
};

export type AnalyticsPluginRecord = {
  id: number;
  endpointId: number;
  pluginId: string;
  status: string;
  createdAt: Date;
};

export type AnalyticsSnapshot = {
  stats: {
    totalRequests: string;
    activeEndpoints: number;
    syncingEndpoints: number;
    apiKeys: number;
  };
  endpointUsageData: Array<{
    name: string;
    requests: number;
    status: string;
  }>;
  networkData: Array<{
    name: string;
    requests: number;
    endpoints: number;
  }>;
  recentEvents: Array<{
    time: string;
    category: 'Endpoint' | 'API Key' | 'Plugin';
    detail: string;
    status: string;
  }>;
};

export const EMPTY_ANALYTICS_SNAPSHOT: AnalyticsSnapshot = {
  stats: {
    totalRequests: '0',
    activeEndpoints: 0,
    syncingEndpoints: 0,
    apiKeys: 0,
  },
  endpointUsageData: [],
  networkData: [],
  recentEvents: [],
};

type AnalyticsBuildInput = {
  endpoints: AnalyticsEndpointRecord[];
  apiKeys: AnalyticsApiKeyRecord[];
  plugins: AnalyticsPluginRecord[];
};

export function buildAnalyticsSnapshot(input: AnalyticsBuildInput): AnalyticsSnapshot {
  const totalRequests = input.endpoints.reduce((sum, endpoint) => sum + endpoint.requests, 0);
  const activeEndpoints = input.endpoints.filter((endpoint) => endpoint.status === 'Active').length;
  const syncingEndpoints = input.endpoints.filter((endpoint) => endpoint.status === 'Syncing').length;

  const networkMap = new Map<string, { name: string; requests: number; endpoints: number }>();
  for (const endpoint of input.endpoints) {
    const existing = networkMap.get(endpoint.network) ?? {
      name: endpoint.network,
      requests: 0,
      endpoints: 0,
    };

    existing.requests += endpoint.requests;
    existing.endpoints += 1;
    networkMap.set(endpoint.network, existing);
  }

  const recentEvents = [
    ...input.endpoints.map((endpoint) => ({
      time: endpoint.createdAt.toISOString(),
      category: 'Endpoint' as const,
      detail: `${endpoint.name} deployed on ${endpoint.network}`,
      status: endpoint.status,
    })),
    ...input.apiKeys.map((apiKey) => ({
      time: apiKey.createdAt.toISOString(),
      category: 'API Key' as const,
      detail: `${apiKey.name} created`,
      status: 'Active',
    })),
    ...input.plugins.map((plugin) => ({
      time: plugin.createdAt.toISOString(),
      category: 'Plugin' as const,
      detail: `${plugin.pluginId} attached to endpoint #${plugin.endpointId}`,
      status: plugin.status,
    })),
  ]
    .sort((left, right) => right.time.localeCompare(left.time))
    .slice(0, 10);

  return {
    stats: {
      totalRequests: formatCompactNumber(totalRequests),
      activeEndpoints,
      syncingEndpoints,
      apiKeys: input.apiKeys.length,
    },
    endpointUsageData: [...input.endpoints]
      .sort((left, right) => right.requests - left.requests)
      .slice(0, 8)
      .map((endpoint) => ({
        name: endpoint.name,
        requests: endpoint.requests,
        status: endpoint.status,
      })),
    networkData: [...networkMap.values()].sort((left, right) => right.requests - left.requests),
    recentEvents,
  };
}
