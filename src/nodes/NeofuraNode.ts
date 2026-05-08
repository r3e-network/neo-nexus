import { existsSync, statSync } from 'node:fs';
import { BaseNode } from './BaseNode';
import type { LogEntry, NodeConfig } from '../types/index';

/**
 * NeofuraNode — adapter for the neo3fura indexer sidecar.
 *
 * Two operating modes, selected by config:
 *
 *   1. **managed-process** — NeoNexus owns the neofura process
 *      lifecycle. Activated when `settings.customConfig.binaryPath`
 *      is set. Falls through to BaseNode's start()/stop()/spawn
 *      machinery, so crash recovery, log capture, and resource
 *      limits all apply for free.
 *
 *   2. **observe-only** — the operator manages neofura externally
 *      (docker-compose / systemd) and just wants NeoNexus to
 *      monitor it. Activated when `binaryPath` is unset and
 *      `settings.customConfig.endpoint` is set. start()/stop()
 *      flip in-memory status without spawning or killing anything.
 *
 * Health (in both modes):
 *   - getBlockHeight() polls the indexer's /summary endpoint and
 *     reads `last_indexed_block` — the same number the explorer's
 *     home-page summary card displays. In managed mode the endpoint
 *     defaults to http://127.0.0.1:<config.ports.rpc>; in observe
 *     mode the operator supplies an absolute URL via
 *     customConfig.endpoint.
 *   - getPeersCount() returns null (neo3fura has no peer concept;
 *     it consumes RPC + WebSocket from exactly one upstream node).
 *
 * Required config (managed-process):
 *   settings.customConfig.binaryPath = absolute path to the
 *     neo3fura_http binary built from `go build` against the
 *     upstream source.
 *   settings.customConfig.workingDir = directory containing the
 *     neo3fura config.yml (defaults to paths.base if unset).
 *   ports.rpc = the HTTP port neo3fura will listen on (NeoNexus
 *     allocates one automatically; the operator can override via
 *     customPorts at create time).
 *
 * Required config (observe-only):
 *   settings.customConfig.endpoint = absolute URL of the neofura
 *     HTTP API root, e.g. "https://api.n3index.dev/mainnet".
 */
export class NeofuraNode extends BaseNode {
  constructor(config: NodeConfig) {
    super(config);
  }

  /**
   * Returns the absolute neo3fura binary path the operator
   * registered, or null when running in observe-only mode.
   */
  private getConfiguredBinaryPath(): string | null {
    const raw = this.config.settings.customConfig?.binaryPath;
    if (typeof raw !== 'string') return null;
    const trimmed = raw.trim();
    return trimmed || null;
  }

  /**
   * Decide between managed-process and observe-only mode based
   * solely on whether a binary path was supplied. Operators who
   * want NeoNexus to take over lifecycle set binaryPath; those
   * who just want monitoring set endpoint instead.
   */
  private getMode(): 'managed' | 'observe' {
    return this.getConfiguredBinaryPath() ? 'managed' : 'observe';
  }

  /** Endpoint URL for /summary polling — managed mode falls back to localhost:rpc. */
  private getEndpoint(): string | null {
    const raw = this.config.settings.customConfig?.endpoint;
    if (typeof raw === 'string' && raw.trim()) {
      return raw.trim().replace(/\/+$/, '');
    }
    if (this.getMode() === 'managed' && this.config.ports?.rpc) {
      return `http://127.0.0.1:${this.config.ports.rpc}`;
    }
    return null;
  }

  // BaseNode abstract methods — used by the BaseNode.start() spawn
  // path in managed mode. In observe mode the overridden start()
  // never reaches these.

  async getBinaryPath(): Promise<string> {
    const binaryPath = this.getConfiguredBinaryPath();
    if (!binaryPath) {
      throw new Error(
        'NeofuraNode is in observe-only mode; no binary configured. ' +
        'Set settings.customConfig.binaryPath to switch to managed-process mode.',
      );
    }
    if (!existsSync(binaryPath)) {
      throw new Error(`Configured neo3fura binary not found: ${binaryPath}`);
    }
    const stat = statSync(binaryPath);
    if (!stat.isFile()) {
      throw new Error(`Configured neo3fura binary is not a regular file: ${binaryPath}`);
    }
    return binaryPath;
  }

  getStartArgs(): string[] {
    // neo3fura_http reads ./config.yml from cwd; the working dir
    // (set below) puts that file in the right place. Pass no
    // extra args by default — operators tune behavior via the
    // YAML config, not flags.
    return [];
  }

  getWorkingDirectory(): string {
    const configured = this.config.settings.customConfig?.workingDir;
    if (typeof configured === 'string' && configured.trim()) {
      return configured.trim();
    }
    return this.config.paths.base;
  }

  /**
   * Override BaseNode.isRunning() so callers like NodeManager
   * .updateMetrics() see observe-only neofura rows as running
   * when their status field says so. The base implementation
   * requires `this.process !== null`, which is always null for
   * an observe-only target — without this override, metric polling
   * (block height, peer count) silently no-ops for the entire
   * observe-mode lifetime. In managed mode, fall through to
   * BaseNode's stricter check so we don't report running while a
   * spawned subprocess is actually dead.
   */
  isRunning(): boolean {
    if (this.getMode() === 'managed') {
      return super.isRunning();
    }
    return this.processStatus.status === 'running';
  }

  /**
   * In managed mode: delegate to BaseNode.start() so we get the
   * spawn + crash-recovery + log-capture pipeline for free.
   * In observe mode: flip in-memory status without touching any
   * process.
   */
  async start(): Promise<void> {
    if (this.getMode() === 'managed') {
      await super.start();
      return;
    }
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
   * In managed mode: delegate to BaseNode.stop() so the spawned
   * process is signaled and reaped. In observe mode: flip the
   * status field — NeoNexus doesn't own the process so nothing
   * to kill.
   */
  async stop(force = false): Promise<void> {
    if (this.getMode() === 'managed') {
      await super.stop(force);
      return;
    }
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
   *
   * In managed mode, calling this before the process is fully
   * listening will return null — the dashboard's existing stalled-
   * sync detection will surface that as a transient condition.
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
