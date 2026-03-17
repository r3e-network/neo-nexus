import { describe, expect, it } from 'vitest';
import { buildManualIncidentResolution } from './AlertIncidentService';

describe('AlertIncidentService', () => {
  it('builds a manual resolution update for an open incident', () => {
    const now = new Date('2026-03-17T00:00:00.000Z');
    const update = buildManualIncidentResolution(now);

    expect(update).toEqual({
      status: 'Resolved',
      resolvedAt: now,
    });
  });
});
