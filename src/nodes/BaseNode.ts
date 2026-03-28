import type { ChildProcess } from 'node:child_process';
import { EventEmitter } from 'node:events';
import type { NodeConfig, NodeStatus, ProcessStatus, NodeMetrics, LogEntry } from '../types/index';
import { spawnProcess, killProcess, getProcessInfo } from '../utils/exec';
import { buildResourceEnv } from '../utils/resourceLimits';

export interface NodeEvents {
  status: (status: NodeStatus, previousStatus: NodeStatus) => void;
  log: (entry: LogEntry) => void;
  metrics: (metrics: NodeMetrics) => void;
  error: (error: Error) => void;
  exit: (code: number | null) => void;
}

export declare interface BaseNode {
  on<K extends keyof NodeEvents>(event: K, listener: NodeEvents[K]): this;
  emit<K extends keyof NodeEvents>(event: K, ...args: Parameters<NodeEvents[K]>): boolean;
}

export abstract class BaseNode extends EventEmitter {
  protected process: ChildProcess | null = null;
  protected processStatus: ProcessStatus = {
    status: 'stopped',
    errorMessage: undefined,
  };
  protected startTime: number | null = null;
  protected logBuffer: LogEntry[] = [];
  protected readonly maxLogBufferSize = 10000;

  constructor(public readonly config: NodeConfig) {
    super();
    this.setMaxListeners(20);
  }

  /**
   * Get the binary path for this node
   */
  abstract getBinaryPath(): Promise<string>;

  /**
   * Get the command line arguments for starting this node
   */
  abstract getStartArgs(): string[];

  /**
   * Get the working directory for the node process
   */
  abstract getWorkingDirectory(): string;

  /**
   * Check if the node is currently running
   */
  isRunning(): boolean {
    return this.process !== null && this.process.exitCode === null && this.processStatus.status === 'running';
  }

  /**
   * Get current process status
   */
  getStatus(): ProcessStatus {
    return { ...this.processStatus };
  }

  /**
   * Start the node process
   */
  async start(): Promise<void> {
    if (this.isRunning()) {
      throw new Error('Node is already running');
    }

    const binaryPath = await this.getBinaryPath();
    const args = this.getStartArgs();
    const cwd = this.getWorkingDirectory();

    const previousStatus = this.processStatus.status;
    this.processStatus.status = 'starting';
    this.emit('status', 'starting', previousStatus);

    try {
      const resourceEnv = buildResourceEnv(this.config.type, this.config.settings.resourceLimits ?? {});
      this.process = spawnProcess(binaryPath, args, {
        cwd,
        env: resourceEnv,
        logCallback: (line) => this.handleLogLine(line),
      });

      this.process.on('exit', (code) => {
        this.handleExit(code);
      });

      this.process.on('error', (error) => {
        this.handleError(error);
      });

      // Wait a moment to check if process started successfully
      await this.delay(2000);

      if (!this.process || this.process.exitCode !== null) {
        throw new Error(`Process exited immediately with code ${this.process?.exitCode}`);
      }

      this.startTime = Date.now();
      this.processStatus = {
        status: 'running',
        pid: this.process.pid,
        lastStarted: this.startTime,
      };
      
      this.emit('status', 'running', 'starting');
    } catch (error) {
      this.processStatus = {
        status: 'error',
        errorMessage: error instanceof Error ? error.message : String(error),
      };
      this.emit('status', 'error', 'starting');
      this.emit('error', error instanceof Error ? error : new Error(String(error)));
      throw error;
    }
  }

  /**
   * Stop the node process
   */
  async stop(force = false): Promise<void> {
    if (!this.isRunning() || !this.process) {
      this.processStatus.status = 'stopped';
      return;
    }

    const previousStatus = this.processStatus.status;
    this.processStatus.status = 'stopping';
    this.emit('status', 'stopping', previousStatus);

    const pid = this.process.pid;
    if (!pid) {
      throw new Error('Cannot stop process: no PID available');
    }

    // Try graceful shutdown first
    const killed = await killProcess(pid, force);
    
    if (!killed) {
      throw new Error('Failed to stop process');
    }

    // Wait for process to exit
    let attempts = 0;
    const maxAttempts = force ? 5 : 30;
    
    while (attempts < maxAttempts) {
      const info = await getProcessInfo(pid);
      if (!info) {
        // Process is gone
        break;
      }
      await this.delay(1000);
      attempts++;
    }

    const stoppedTime = Date.now();
    this.processStatus = {
      status: 'stopped',
      lastStopped: stoppedTime,
      uptime: this.startTime ? stoppedTime - this.startTime : undefined,
    };
    this.startTime = null;
    this.process = null;
    
    this.emit('status', 'stopped', 'stopping');
  }

  /**
   * Restart the node
   */
  async restart(): Promise<void> {
    await this.stop();
    await this.delay(1000);
    await this.start();
  }

  /**
   * Get recent logs
   */
  getLogs(count = 100): LogEntry[] {
    return this.logBuffer.slice(-count);
  }

  /**
   * Get process resource usage
   */
  async getResourceUsage(): Promise<{ cpu: number; memory: number } | null> {
    if (!this.process?.pid) return null;
    
    const info = await getProcessInfo(this.process.pid);
    if (!info) return null;

    return {
      cpu: info.cpu,
      memory: info.memory,
    };
  }

  /**
   * Execute a JSON-RPC command on the running node
   */
  async executeRpc(method: string, ...params: string[]): Promise<string> {
    if (!this.isRunning()) {
      throw new Error('Node is not running');
    }

    const rpcUrl = `http://127.0.0.1:${this.config.ports.rpc}`;

    const response = await fetch(rpcUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method,
        params,
        id: 1,
      }),
    });

    if (!response.ok) {
      throw new Error(`RPC call failed: ${response.statusText}`);
    }

    const data = await response.json() as { error?: { message: string }; result: unknown };
    if (data.error) {
      throw new Error(`RPC error: ${data.error.message}`);
    }

    return JSON.stringify(data.result);
  }

  /**
   * Handle log line from process
   */
  protected handleLogLine(line: string): void {
    const entry = this.parseLogLine(line);
    this.logBuffer.push(entry);
    
    // Trim buffer if it gets too large
    if (this.logBuffer.length > this.maxLogBufferSize) {
      this.logBuffer = this.logBuffer.slice(-this.maxLogBufferSize / 2);
    }

    this.emit('log', entry);
  }

  /**
   * Parse a log line into a structured entry
   */
  protected parseLogLine(line: string): LogEntry {
    // Default parsing - subclasses can override for specific formats
    let level: LogEntry['level'] = 'info';
    
    const lowerLine = line.toLowerCase();
    if (lowerLine.includes('error') || lowerLine.includes('exception') || lowerLine.includes('fail')) {
      level = 'error';
    } else if (lowerLine.includes('warn')) {
      level = 'warn';
    } else if (lowerLine.includes('debug')) {
      level = 'debug';
    }

    return {
      timestamp: Date.now(),
      level,
      source: 'node',
      message: line,
    };
  }

  /**
   * Handle process exit
   */
  protected handleExit(code: number | null): void {
    const previousStatus = this.processStatus.status;
    
    if (previousStatus !== 'stopping') {
      // Process exited unexpectedly
      this.processStatus = {
        status: code === 0 ? 'stopped' : 'error',
        exitCode: code ?? undefined,
        errorMessage: code !== 0 && code !== null ? `Process exited with code ${code}` : undefined,
        lastStopped: Date.now(),
        uptime: this.startTime ? Date.now() - this.startTime : undefined,
      };
      
      this.emit('status', this.processStatus.status, previousStatus);
      this.emit('exit', code);
    }

    this.process = null;
    this.startTime = null;
  }

  /**
   * Handle process error
   */
  protected handleError(error: Error): void {
    const previousStatus = this.processStatus.status;
    this.processStatus = {
      status: 'error',
      errorMessage: error.message,
    };
    this.emit('status', 'error', previousStatus);
    this.emit('error', error);
  }

  /**
   * Delay helper
   */
  protected delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get current block height
   */
  abstract getBlockHeight(): Promise<number | null>;

  /**
   * Get connected peers count
   */
  abstract getPeersCount(): Promise<number | null>;

  /**
   * Clean up resources
   */
  destroy(): void {
    if (this.isRunning()) {
      this.stop(true).catch(console.error);
    }
    this.removeAllListeners();
  }
}
