import { describe, it, expect } from 'vitest';
import { calculateDaysUntilFull, getDiskAlertLevel } from '../../src/utils/diskMonitor';

describe('calculateDaysUntilFull', () => {
  it('calculates days correctly for positive growth', () => {
    // 100 GB free, 1 GB/hour growth = 100 / 24 ≈ 4.17 days
    const freeBytes = 100 * 1024 * 1024 * 1024;
    const growthPerHour = 1 * 1024 * 1024 * 1024;
    const days = calculateDaysUntilFull(freeBytes, growthPerHour);
    expect(days).toBeCloseTo(4.17, 1);
  });

  it('returns Infinity when growthBytesPerHour is 0', () => {
    expect(calculateDaysUntilFull(1000000, 0)).toBe(Infinity);
  });

  it('returns Infinity when growthBytesPerHour is negative', () => {
    expect(calculateDaysUntilFull(1000000, -100)).toBe(Infinity);
  });

  it('returns 0 when freeBytes is 0', () => {
    expect(calculateDaysUntilFull(0, 1000)).toBe(0);
  });

  it('calculates correctly for 24 hours of growth at 1 byte/hour', () => {
    // 24 bytes free, 1 byte/hour = 1 day
    expect(calculateDaysUntilFull(24, 1)).toBe(1);
  });
});

describe('getDiskAlertLevel', () => {
  it('returns critical at 95% or above', () => {
    expect(getDiskAlertLevel(95)).toBe('critical');
    expect(getDiskAlertLevel(100)).toBe('critical');
    expect(getDiskAlertLevel(96)).toBe('critical');
  });

  it('returns warning at 90% to 94%', () => {
    expect(getDiskAlertLevel(90)).toBe('warning');
    expect(getDiskAlertLevel(94)).toBe('warning');
    expect(getDiskAlertLevel(91)).toBe('warning');
  });

  it('returns null below 90%', () => {
    expect(getDiskAlertLevel(89)).toBeNull();
    expect(getDiskAlertLevel(50)).toBeNull();
    expect(getDiskAlertLevel(0)).toBeNull();
  });

  it('returns null at exactly 89.9%', () => {
    expect(getDiskAlertLevel(89.9)).toBeNull();
  });

  it('returns warning at exactly 90%', () => {
    expect(getDiskAlertLevel(90)).toBe('warning');
  });

  it('returns critical at exactly 95%', () => {
    expect(getDiskAlertLevel(95)).toBe('critical');
  });
});
