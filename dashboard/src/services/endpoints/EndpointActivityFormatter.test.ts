import { describe, expect, it } from 'vitest';
import { formatEndpointActivityMetadata } from './EndpointActivityFormatter';

describe('EndpointActivityFormatter', () => {
  it('formats provider metadata into readable rows', () => {
    const rows = formatEndpointActivityMetadata({
      provider: 'hetzner',
      region: 'fsn1',
      nextAttemptAt: '2026-03-16T12:00:00.000Z',
    });

    expect(rows).toEqual([
      { label: 'Provider', value: 'hetzner' },
      { label: 'Region', value: 'fsn1' },
      { label: 'Next Attempt At', value: '2026-03-16T12:00:00.000Z' },
    ]);
  });

  it('returns an empty list for unsupported metadata types', () => {
    expect(formatEndpointActivityMetadata('text')).toEqual([]);
  });
});
