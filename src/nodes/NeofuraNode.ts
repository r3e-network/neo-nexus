import { BaseNode } from './BaseNode';
import type { LogEntry, NodeConfig } from '../types/index';

/**
 * NeofuraNode — observe-only adapter for an externally-managed
 * neo3fura indexer process.
 *
 * neo3fura ships as Go source (docker-compose + start.sh in the
 * upstream repo) and does not publish GitHub release binaries. The
 * common deployment shape is a sidecar managed by docker-compose,
 * systemd, or a process supervisor outside NeoNexus's view. Rather
 * than fork that responsibility, NeoNexus models neofura as an
 * observable target:
 *
 *   - start() / stop() / restart() do not spawn or kill anything;
 *     they flip in-memory state so the rest of NeoNexus's status
 *     plumbing keeps working.
 *   - getBlockHeight() polls the indexer's /summary endpoint and
 *     reads `last_indexed_block` — the same number the explorer's
 *     home-page summary card displays.
 *   - getPeersCount() returns null (a neo3fura process has no peer
 *     concept; it consumes RPC + WebSocket from one upstream node).
 *
 * Required config:
 *   settings.customConfig.endpoint = absolute URL of the neofura
 *     HTTP API root, e.g. "https://api.n3index.dev/mainnet".
 *
 * The class deliberately doesn't override getBinaryPath /
 * getStartArgs / getWorkingDirectory — they're abstract on BaseNode
 * but never called once start() is overridden, and surfacing them
 * as `throw` would just produce noisier stack traces if some
 * unrelated code ever stumbled in.
 */
export class NeofuraNode extends BaseNode {
  constructor(config: NodeConfig) {
    super(config);
  }

  /** Endpoint URL for the running neofura HTTP API, or null if unset. */
  private getEndpoint(): string | null {
    const raw = this.config.settings.customConfig?.endpoint;
    if (typeof raw !== 'string') return null;
    const trimmed = raw.trim().replace(/\/+$/, '');
    return trimmed || null;
  }

  // BaseNode abstract methods — neofura is observe-only, so these
  // are never reached via the overridden start() path. They're
  // implemented to satisfy TypeScript and to give a clear error if
  // future code calls them by mistake.
  async getBinaryPath(): Promise<string> {
    throw new Error(
      'NeofuraNode is observe-only; binaries are managed by the external supervisor (docker-compose / systemd).',
    );
  }

  getStartArgs(): string[] {
    return [];
  }

  getWorkingDirectory(): string {
    return this.config.paths.base;
  }

  /**
   * Override BaseNode.start(): no process spawn. Flip state to
   * 'running' and emit a status event so dashboards see the row
   * as online. The actual neofura process must already be up;
   * health is verified lazily via getBlockHeight().
   *
   * Don't gate on BaseNode.isRunning() — that helper checks
   * `this.process !== null`, which is always null for an
   * observe-only node. Use the status field directly.
   */
  async start(): Promise<void> {
    if (this.processStatus.status === 'running' || this.processStatus.status === 'starting') {
      throw new Error('NeofuraNode is already marked as running');
    }
    const previousStatus = this.processStatus.status;
    const now = Date.now();
    this.processStatus = {
      status: 'running',
      lastStarted: now,
    };
    this.startTime = now;
    this.emit('status', 'running', previousStatus);
  }

  /**
   * Override BaseNode.stop(): mark observed-target as stopped
   * without killing any process — NeoNexus does not own the
   * neofura process lifecycle. Mirror the start() reasoning: gate
   * on the status field, not BaseNode.isRunning(), since
   * `this.process` is always null.
   */
  async stop(_force = false): Promise<void> {
    if (this.processStatus.status !== 'running' && this.processStatus.status !== 'starting') {
      this.processStatus.status = 'stopped';
      return;
    }
    const previousStatus = this.processStatus.status;
    const stoppedTime = Date.now();
    this.processStatus = {
      status: 'stopped',
      lastStopped: stoppedTime,
      uptime: this.startTime ? stoppedTime - this.startTime : 0,
    };
    this.startTime = null;
    this.emit('status', 'stopped', previousStatus);
  }

  async restart(): Promise<void> {
    await this.stop();
    await this.delay(250);
    await this.start();
  }

  /**
   * Pull the latest indexed block height from the neofura HTTP API
   * (/<root>/summary returns { data: { last_indexed_block, ... } }).
   * Returns null on any failure so the caller can render
   * "indexer offline" without crashing.
   */
  async getBlockHeight(): Promise<number | null> {
    const endpoint = this.getEndpoint();
    if (!endpoint) return null;

    try {
      const controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
      const timer = controller ? setTimeout(() => controller.abort(), 8000) : null;
      try {
        const res = await fetch(`${endpoint}/summary`, {
          headers: { Accept: 'application/json' },
          signal: controller?.signal,
        });
        if (!res.ok) return null;
        const json = (await res.json()) as { data?: { last_indexed_block?: number } } | null;
        const height = Number(json?.data?.last_indexed_block);
        return Number.isFinite(height) && height >= 0 ? height : null;
      } finally {
        if (timer) clearTimeout(timer);
      }
    } catch {
      return null;
    }
  }

  /**
   * Neofura has no peer concept — it consumes RPC + WebSocket from
   * exactly one upstream node. Return null so dashboards render
   * "—" rather than a misleading 0.
   */
  async getPeersCount(): Promise<number | null> {
    return null;
  }

  /**
   * Override the default log parser since neofura logs aren't
   * captured (we don't own its stdout). Return a synthetic info
   * line if anyone calls handleLogLine externally.
   */
  protected parseLogLine(line: string): LogEntry {
    return {
      timestamp: Date.now(),
      level: 'info',
      source: 'neofura',
      message: line,
    };
  }
}
