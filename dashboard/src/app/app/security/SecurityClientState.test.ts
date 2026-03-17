import { describe, expect, it } from 'vitest';
import { mergeCreatedApiKey } from './SecurityClientState';

describe('SecurityClientState', () => {
  it('appends a newly created api key and keeps keys sorted by created date', () => {
    const next = mergeCreatedApiKey(
      [
        {
          id: 'key-2',
          name: 'Key 2',
          keyHash: 'hash-2',
          createdAt: new Date('2026-03-16T10:00:00Z'),
          isActive: true,
        },
      ],
      {
        id: 'key-1',
        name: 'Key 1',
        keyHash: 'hash-1',
        createdAt: new Date('2026-03-15T10:00:00Z'),
        isActive: true,
      },
    );

    expect(next.map((key) => key.id)).toEqual(['key-1', 'key-2']);
  });
});
