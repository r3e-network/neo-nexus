import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  requireCurrentOrganizationContext: vi.fn(),
  endpointFindFirst: vi.fn(),
  nodeSecretDeleteMany: vi.fn(),
  nodePluginUpdateMany: vi.fn(),
  storeNodeSecret: vi.fn(),
  renderPluginConfig: vi.fn(),
  recordEndpointActivity: vi.fn(),
  getErrorMessage: vi.fn((error: unknown) => error instanceof Error ? error.message : 'Unknown error'),
}));

vi.mock('@/server/organization', () => ({
  requireCurrentOrganizationContext: mocks.requireCurrentOrganizationContext,
}));

vi.mock('@/utils/prisma', () => ({
  prisma: {
    endpoint: {
      findFirst: mocks.endpointFindFirst,
    },
    nodeSecret: {
      deleteMany: mocks.nodeSecretDeleteMany,
    },
    nodePlugin: {
      upsert: vi.fn(),
      update: vi.fn(),
      updateMany: mocks.nodePluginUpdateMany,
      delete: vi.fn(),
    },
  },
}));

vi.mock('@/services/vault/VaultService', () => ({
  VaultService: {
    storeNodeSecret: mocks.storeNodeSecret,
  },
}));

vi.mock('@/server/errors', () => ({
  getErrorMessage: mocks.getErrorMessage,
}));

vi.mock('@/services/endpoints/EndpointActivityService', () => ({
  recordEndpointActivity: mocks.recordEndpointActivity,
}));

vi.mock('@/services/plugins/PluginCatalog', () => ({
  isSupportedPlugin: vi.fn(() => true),
  getPluginDefinition: vi.fn(() => ({
    id: 'tee-oracle',
    name: 'TEE Privacy Oracle',
    description: 'desc',
    badge: 'Privacy',
    requiresPrivateKey: true,
    category: 'Security',
    defaultImage: 'ghcr.io/neonexus/tee-oracle:latest',
  })),
}));

vi.mock('@/services/plugins/PluginConfigRenderer', () => ({
  renderPluginConfig: mocks.renderPluginConfig,
}));

vi.mock('@/services/plugins/RemotePluginSync', () => ({
  buildRemotePluginRemovalCommand: vi.fn(),
  buildRemotePluginSyncCommand: vi.fn(),
}));

vi.mock('next/cache', () => ({
  revalidatePath: vi.fn(),
}));

import { addNodePluginAction } from './pluginActions';

describe('plugin actions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.requireCurrentOrganizationContext.mockResolvedValue({
      userId: 'user_123',
      organizationId: 'org_123',
      billingPlan: 'growth',
      role: 'member',
    });
    mocks.endpointFindFirst.mockResolvedValue({
      id: 42,
      organizationId: 'org_123',
      type: 'Dedicated',
      providerPublicIp: null,
    });
    mocks.storeNodeSecret.mockResolvedValue({
      id: 1,
    });
    mocks.nodeSecretDeleteMany.mockResolvedValue({ count: 1 });
    mocks.nodePluginUpdateMany.mockResolvedValue({ count: 0 });
    mocks.renderPluginConfig.mockImplementation(() => {
      throw new Error('render failed');
    });
  });

  it('does not persist a plugin secret when config rendering fails', async () => {
    const result = await addNodePluginAction(
      42,
      'tee-oracle',
      { network: 'mainnet' },
      'L1-private-key',
    );

    expect(result).toEqual({
      success: false,
      error: 'render failed',
    });
    expect(mocks.storeNodeSecret).not.toHaveBeenCalled();
    expect(mocks.nodeSecretDeleteMany).not.toHaveBeenCalled();
  });
});
