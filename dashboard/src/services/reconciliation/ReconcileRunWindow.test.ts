import { describe, expect, it } from 'vitest';
import { clampReconcileRunsLimit } from './ReconcileRunWindow';

describe('ReconcileRunWindow', () => {
  it('defaults invalid limits and clamps oversized requests', () => {
    expect(clampReconcileRunsLimit(undefined)).toBe(10);
    expect(clampReconcileRunsLimit('abc')).toBe(10);
    expect(clampReconcileRunsLimit('200')).toBe(50);
    expect(clampReconcileRunsLimit('25')).toBe(25);
  });
});
