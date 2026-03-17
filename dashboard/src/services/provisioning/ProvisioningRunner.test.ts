import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  provisioningOrderFindUnique: vi.fn(),
  provisioningOrderUpdate: vi.fn(),
  endpointUpdate: vi.fn(),
  endpointFindUnique: vi.fn(),
  createRoute: vi.fn(),
  recordEndpointActivity: vi.fn(),
  getSharedBackendTarget: vi.fn(),
  loadOrganizationRoutePlugins: vi.fn(),
}));

vi.mock('../../utils/prisma', () => ({
  prisma: {
    provisioningOrder: {
      findUnique: mocks.provisioningOrderFindUnique,
      update: mocks.provisioningOrderUpdate,
    },
    endpoint: {
      update: mocks.endpointUpdate,
      findUnique: mocks.endpointFindUnique,
    },
  },
}));

vi.mock('../apisix/ApisixService', () => ({
  ApisixService: {
    createRoute: mocks.createRoute,
  },
}));

vi.mock('../endpoints/EndpointActivityService', () => ({
  recordEndpointActivity: mocks.recordEndpointActivity,
}));

vi.mock('../endpoints/SharedEndpointConfig', () => ({
  getSharedBackendTarget: mocks.getSharedBackendTarget,
}));

vi.mock('../security/RouteSecuritySync', () => ({
  loadOrganizationRoutePlugins: mocks.loadOrganizationRoutePlugins,
}));

vi.mock('../neo/NeoNodeService', () => ({
  NeoNodeService: {
    checkHealth: vi.fn(),
  },
}));

vi.mock('./VmProvisioner', () => ({
  provisionDedicatedNode: vi.fn(),
}));

import { kickoffProvisioningOrder } from './ProvisioningRunner';

describe('ProvisioningRunner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.provisioningOrderUpdate.mockResolvedValue({});
    mocks.endpointUpdate.mockResolvedValue({});
    mocks.endpointFindUnique.mockResolvedValue({
      id: 42,
      url: 'https://rpc.example.com',
      clientEngine: 'neo-go',
    });
    mocks.createRoute.mockResolvedValue(true);
    mocks.recordEndpointActivity.mockResolvedValue(undefined);
    mocks.getSharedBackendTarget.mockReturnValue({
      host: 'shared.neo.test',
      port: 10332,
    });
    mocks.loadOrganizationRoutePlugins.mockResolvedValue({});
  });

  it('does not kick off provisioning before a scheduled retry is due', async () => {
    mocks.provisioningOrderFindUnique
      .mockResolvedValueOnce({
        id: 5,
        organizationId: 'org_123',
        provider: 'shared',
        endpointId: 42,
        endpoint: {
          id: 42,
          name: 'Mainnet Shared',
          protocol: 'neo-n3',
          networkKey: 'mainnet',
          type: 'Shared',
          clientEngine: 'neo-go',
          syncMode: 'full',
          url: 'https://rpc.example.com',
          region: null,
          cloudProvider: null,
        },
      })
      .mockResolvedValueOnce({
        status: 'pending',
        nextAttemptAt: new Date('2030-01-01T00:00:00.000Z'),
      });

    const result = await kickoffProvisioningOrder(5);

    expect(result).toBe(false);
    expect(mocks.provisioningOrderUpdate).not.toHaveBeenCalled();
    expect(mocks.createRoute).not.toHaveBeenCalled();
  });
});
