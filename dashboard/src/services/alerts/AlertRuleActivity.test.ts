import { describe, expect, it } from 'vitest';
import { filterAlertRuleActivities } from './AlertRuleActivity';

describe('AlertRuleActivity', () => {
  it('filters endpoint activities to a specific alert rule id', () => {
    const rows = filterAlertRuleActivities(12, [
      {
        id: 1,
        action: 'delivery_failed',
        message: 'Alert delivery failed.',
        createdAt: new Date('2026-03-16T10:00:00Z'),
        metadata: { alertRuleId: 12 },
      },
      {
        id: 2,
        action: 'delivery_failed',
        message: 'Other alert delivery failed.',
        createdAt: new Date('2026-03-16T10:01:00Z'),
        metadata: { alertRuleId: 99 },
      },
      {
        id: 3,
        action: 'incident_opened',
        message: 'Alert incident opened.',
        createdAt: new Date('2026-03-16T10:02:00Z'),
        metadata: null,
      },
    ]);

    expect(rows.map((row) => row.id)).toEqual([1]);
  });
});
