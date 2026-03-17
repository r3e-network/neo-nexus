import { describe, expect, it } from 'vitest';
import { buildIncidentTimeline } from './IncidentTimeline';

describe('IncidentTimeline', () => {
  it('builds a reverse-chronological timeline from incident and activity data', () => {
    const timeline = buildIncidentTimeline({
      incident: {
        openedAt: '2026-03-16T10:00:00.000Z',
        resolvedAt: null,
        lastDeliveredAt: '2026-03-16T10:05:00.000Z',
        lastDeliveryError: 'Webhook failed',
      },
      activities: [
        {
          id: 1,
          category: 'alert',
          action: 'delivery_failed',
          status: 'error',
          message: 'Alert delivery failed.',
          createdAt: '2026-03-16T10:06:00.000Z',
        },
      ],
    });

    expect(timeline[0]).toMatchObject({
      kind: 'activity',
      message: 'Alert delivery failed.',
    });
    expect(timeline[1]).toMatchObject({
      kind: 'delivery',
      status: 'success',
    });
    expect(timeline.some((item) => item.kind === 'incident' && item.status === 'opened')).toBe(true);
    expect(timeline[timeline.length - 1]).toMatchObject({
      kind: 'incident',
      status: 'opened',
    });
  });
});
