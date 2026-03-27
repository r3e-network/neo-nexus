import { describe, it, expect } from 'vitest';
import { NetworkHeightTracker } from '../../src/core/NetworkHeightTracker';

describe('NetworkHeightTracker', () => {
  it('returns 0 for unknown network', () => {
    const tracker = new NetworkHeightTracker();
    expect(tracker.getHeight('mainnet')).toBe(0);
  });

  it('calculates sync progress correctly', () => {
    const tracker = new NetworkHeightTracker();
    tracker.setHeight('testnet', 10000);
    expect(tracker.getSyncProgress(5000, 'testnet')).toBeCloseTo(0.5);
    expect(tracker.getSyncProgress(10000, 'testnet')).toBe(1.0);
    expect(tracker.getSyncProgress(0, 'testnet')).toBe(0);
  });

  it('detects stalling', () => {
    const tracker = new NetworkHeightTracker();
    tracker.setHeight('testnet', 10000);
    expect(tracker.isStalled('node-1', 5000)).toBe(false);
    tracker.recordNodeHeight('node-1', 5000);
    // Simulate staleness
    const record = (tracker as any).nodeHeights.get('node-1');
    record.firstSeenAt = Date.now() - 6 * 60 * 1000;
    expect(tracker.isStalled('node-1', 5000)).toBe(true);
    // Height change resets stale
    tracker.recordNodeHeight('node-1', 5001);
    expect(tracker.isStalled('node-1', 5001)).toBe(false);
  });
});
