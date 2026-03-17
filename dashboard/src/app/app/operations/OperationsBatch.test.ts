import { describe, expect, it } from 'vitest';
import { normalizeBatchIds } from './OperationsBatch';

describe('OperationsBatch', () => {
  it('deduplicates and filters invalid ids', () => {
    expect(normalizeBatchIds([1, 2, 2, -1, 0, 3.5, 4])).toEqual([1, 2, 4]);
  });
});
