import { describe, expect, it } from 'vitest';
import { filterProvisioningActivities } from './ProvisioningActivity';

describe('ProvisioningActivity', () => {
  it('filters endpoint activities to provisioning rows in reverse chronological order', () => {
    const rows = filterProvisioningActivities([
      {
        id: 1,
        category: 'alert',
        message: 'Alert triggered',
        createdAt: '2026-03-17T10:00:00.000Z',
      },
      {
        id: 2,
        category: 'provisioning',
        message: 'VM created',
        createdAt: '2026-03-17T10:05:00.000Z',
      },
      {
        id: 3,
        category: 'provisioning',
        message: 'Route configured',
        createdAt: '2026-03-17T10:10:00.000Z',
      },
    ]);

    expect(rows.map((row) => row.id)).toEqual([3, 2]);
  });
});
