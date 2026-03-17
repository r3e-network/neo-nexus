import { describe, expect, it } from 'vitest';
import { toEndpointActivityView } from './EndpointActivityService';

describe('EndpointActivityService', () => {
  it('serializes endpoint activity rows for API responses', () => {
    const view = toEndpointActivityView({
      id: 1,
      endpointId: 9,
      category: 'provisioning',
      action: 'vm_created',
      status: 'success',
      message: 'Dedicated VM created successfully.',
      metadata: { provider: 'hetzner' },
      createdAt: new Date('2026-03-16T12:00:00Z'),
    });

    expect(view).toEqual({
      id: 1,
      endpointId: 9,
      category: 'provisioning',
      action: 'vm_created',
      status: 'success',
      message: 'Dedicated VM created successfully.',
      metadata: { provider: 'hetzner' },
      createdAt: '2026-03-16T12:00:00.000Z',
    });
  });
});
