import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WatchdogManager } from '../../src/core/WatchdogManager';

describe('WatchdogManager', () => {
  let watchdog: WatchdogManager;
  let mockRestart: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockRestart = vi.fn().mockResolvedValue(undefined);
    watchdog = new WatchdogManager(mockRestart);
  });

  it('schedules a restart on unexpected exit', () => {
    vi.useFakeTimers();
    watchdog.onNodeStarted('node-1');
    watchdog.onNodeExited('node-1', false);
    vi.advanceTimersByTime(2100);
    expect(mockRestart).toHaveBeenCalledWith('node-1');
    vi.useRealTimers();
  });

  it('does not restart on expected stop', () => {
    vi.useFakeTimers();
    watchdog.onNodeStarted('node-1');
    watchdog.onNodeExited('node-1', true);
    vi.advanceTimersByTime(10000);
    expect(mockRestart).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it('uses exponential backoff', () => {
    watchdog.onNodeStarted('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(2000);
    watchdog.recordFailure('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(4000);
    watchdog.recordFailure('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(8000);
  });

  it('caps backoff at 30 seconds', () => {
    watchdog.onNodeStarted('node-1');
    for (let i = 0; i < 10; i++) watchdog.recordFailure('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(30000);
  });

  it('gives up after max failures', () => {
    vi.useFakeTimers();
    watchdog.onNodeStarted('node-1');
    for (let i = 0; i < 5; i++) watchdog.recordFailure('node-1');
    watchdog.onNodeExited('node-1', false);
    vi.advanceTimersByTime(60000);
    expect(mockRestart).not.toHaveBeenCalled();
    expect(watchdog.isExhausted('node-1')).toBe(true);
    vi.useRealTimers();
  });

  it('resets backoff after resetBackoff call', () => {
    watchdog.onNodeStarted('node-1');
    watchdog.recordFailure('node-1');
    watchdog.recordFailure('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(8000);
    watchdog.resetBackoff('node-1');
    expect(watchdog.getBackoffMs('node-1')).toBe(2000);
  });

  it('can disable auto-restart per node', () => {
    vi.useFakeTimers();
    watchdog.onNodeStarted('node-1');
    watchdog.disable('node-1');
    watchdog.onNodeExited('node-1', false);
    vi.advanceTimersByTime(10000);
    expect(mockRestart).not.toHaveBeenCalled();
    vi.useRealTimers();
  });
});
