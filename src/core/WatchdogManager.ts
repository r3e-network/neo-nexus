const INITIAL_BACKOFF_MS = 2000;
const MAX_BACKOFF_MS = 30000;
const MAX_CONSECUTIVE_FAILURES = 5;
const STABLE_RESET_MS = 5 * 60 * 1000;

interface NodeWatchState {
  failures: number;
  enabled: boolean;
  timer: ReturnType<typeof setTimeout> | null;
  stableTimer: ReturnType<typeof setTimeout> | null;
}

export class WatchdogManager {
  private states = new Map<string, NodeWatchState>();

  constructor(private restartFn: (nodeId: string) => Promise<void>) {}

  onNodeStarted(nodeId: string): void {
    const state = this.getOrCreate(nodeId);
    state.enabled = true;
    if (state.stableTimer) clearTimeout(state.stableTimer);
    state.stableTimer = setTimeout(() => this.resetBackoff(nodeId), STABLE_RESET_MS);
  }

  onNodeExited(nodeId: string, wasExpected: boolean): void {
    const state = this.states.get(nodeId);
    if (!state || !state.enabled || wasExpected) return;
    if (state.stableTimer) { clearTimeout(state.stableTimer); state.stableTimer = null; }
    if (state.failures >= MAX_CONSECUTIVE_FAILURES) return;

    const delay = this.getBackoffMs(nodeId);
    console.log(`[watchdog] scheduling restart of ${nodeId} in ${delay}ms (attempt ${state.failures + 1}/${MAX_CONSECUTIVE_FAILURES})`);
    state.timer = setTimeout(async () => {
      try {
        this.recordFailure(nodeId);
        await this.restartFn(nodeId);
      } catch (error) {
        console.error(`Watchdog: failed to restart ${nodeId}:`, error);
      }
    }, delay);
  }

  getBackoffMs(nodeId: string): number {
    const failures = this.states.get(nodeId)?.failures ?? 0;
    return Math.min(INITIAL_BACKOFF_MS * Math.pow(2, failures), MAX_BACKOFF_MS);
  }

  recordFailure(nodeId: string): void {
    this.getOrCreate(nodeId).failures++;
  }

  resetBackoff(nodeId: string): void {
    const state = this.states.get(nodeId);
    if (state) state.failures = 0;
  }

  isExhausted(nodeId: string): boolean {
    return (this.states.get(nodeId)?.failures ?? 0) >= MAX_CONSECUTIVE_FAILURES;
  }

  disable(nodeId: string): void {
    const state = this.states.get(nodeId);
    if (state) {
      state.enabled = false;
      if (state.timer) { clearTimeout(state.timer); state.timer = null; }
      if (state.stableTimer) { clearTimeout(state.stableTimer); state.stableTimer = null; }
    }
  }

  clearAll(): void {
    for (const [, state] of this.states) {
      if (state.timer) clearTimeout(state.timer);
      if (state.stableTimer) clearTimeout(state.stableTimer);
    }
    this.states.clear();
  }

  private getOrCreate(nodeId: string): NodeWatchState {
    if (!this.states.has(nodeId)) {
      this.states.set(nodeId, { failures: 0, enabled: true, timer: null, stableTimer: null });
    }
    return this.states.get(nodeId)!;
  }
}
