import { join } from 'node:path';
import { BaseNode } from './BaseNode';
import { DownloadManager } from '../core/DownloadManager';
import type { NodeConfig, LogEntry } from '../types/index';

export class NeoGoNode extends BaseNode {
  constructor(config: NodeConfig) {
    super(config);
  }

  async getBinaryPath(): Promise<string> {
    const path = DownloadManager.getNodeBinaryPath('neo-go', this.config.version);
    if (!path) {
      throw new Error(`neo-go ${this.config.version} is not downloaded`);
    }
    return path;
  }

  getStartArgs(): string[] {
    const args: string[] = [
      'node',
      '--config-file', join(this.config.paths.config, 'protocol.yml'),
      '--relative-path',
      this.config.paths.base,
    ];

    // neo-go uses short flags for network selection
    if (this.config.network === 'testnet') {
      args.push('--testnet');
    } else if (this.config.network === 'private') {
      args.push('--privnet');
    }
    // mainnet is default — no flag needed

    return args;
  }

  getWorkingDirectory(): string {
    return this.config.paths.base;
  }

  /**
   * Parse neo-go specific log format
   */
  protected parseLogLine(line: string): LogEntry {
    // neo-go log format: TIMESTAMP LEVEL COMPONENT MESSAGE
    // Example: 2024-01-15T11:23:45.123Z	INFO	server	Block 123456 persisted
    
    const match = line.match(/^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(\w+)\s+(\S+)\s+(.+)$/);
    
    if (match) {
      const [, timeStr, levelStr, component, message] = match;
      
      let level: LogEntry['level'] = 'info';
      const lowerLevel = levelStr.toLowerCase();
      
      if (lowerLevel === 'error' || lowerLevel === 'fatal') {
        level = 'error';
      } else if (lowerLevel === 'warn' || lowerLevel === 'warning') {
        level = 'warn';
      } else if (lowerLevel === 'debug') {
        level = 'debug';
      }

      return {
        timestamp: new Date(timeStr).getTime(),
        level,
        source: component,
        message: message.trim(),
      };
    }

    // Fallback to default parsing
    return super.parseLogLine(line);
  }

  /**
   * Get node height via RPC
   */
  async getBlockHeight(): Promise<number | null> {
    try {
      const result = await this.executeRpc('getblockcount');
      const parsed = JSON.parse(result) as number;
      return typeof parsed === 'number' ? parsed : null;
    } catch {
      return null;
    }
  }

  /**
   * Get connected peers via RPC
   */
  async getPeersCount(): Promise<number | null> {
    try {
      const result = await this.executeRpc('getpeers');
      const peers = JSON.parse(result);
      return peers.connected?.length ?? 0;
    } catch {
      return null;
    }
  }

  /**
   * Get neo-go version
   */
  static async getVersion(binaryPath: string): Promise<string | null> {
    try {
      const { exec } = await import('../utils/exec');
      const result = await exec(binaryPath, ['--version']);
      return result.stdout.trim();
    } catch {
      return null;
    }
  }
}
