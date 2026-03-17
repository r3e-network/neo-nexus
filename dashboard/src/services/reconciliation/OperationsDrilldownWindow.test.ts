import { describe, expect, it } from 'vitest';
import { clampOperationsItemsLimit } from './OperationsDrilldownWindow';

describe('OperationsDrilldownWindow', () => {
  it('clamps drilldown limits between 10 and 200', () => {
    expect(clampOperationsItemsLimit(undefined)).toBe(25);
    expect(clampOperationsItemsLimit('abc')).toBe(25);
    expect(clampOperationsItemsLimit('5')).toBe(10);
    expect(clampOperationsItemsLimit('75')).toBe(75);
    expect(clampOperationsItemsLimit('500')).toBe(200);
  });
});
