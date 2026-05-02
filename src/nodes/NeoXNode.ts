import { join } from 'node:path';
import { BaseNode } from './BaseNode';
import { DownloadManager } from '../core/DownloadManager';
import type { NodeConfig, LogEntry } from '../types/index';

const NETWORK_IDS: Record<string, number> = {
  'neox-mainnet': 47763,
  'neox-testnet': 12227332,
};

const BOOTNODES: Record<string, string> = {
  'neox-mainnet':
    'enode://92eec46dd8b67ea8d8999defe0bf2b43d4c4802ed42a430843fec97dafbdc9128849261bdf1a940d431fc61f06a1317f5fc7c0386e18a9bbf951d0ccd8bf4f98@34.42.6.58:30303,enode://f289fb5c83ed39cf7d7aff2727afe70bf7951222c4a9aaef7bcbceef9fd0b53e4b6c9c0e08a50774dfd50d93e83b977932e4780934d379a6a0ac10cc44c6cfdb@34.87.188.162:30303',
};

export class NeoXNode extends BaseNode {
  constructor(config: NodeConfig) {
    super(config);
  }

  async getBinaryPath(): Promise<string> {
    const path = DownloadManager.getNodeBinaryPath('neox-go', this.config.version);
    if (!path) {
      throw new Error(`neox-go (geth) ${this.config.version} is not downloaded`);
    }
    return path;
  }

  getStartArgs(): string[] {
    const networkId = NETWORK_IDS[this.config.network];
    if (!networkId) {
      throw new Error(`Unsupported Neo X network: ${this.config.network}`);
    }
    const bootnodes = BOOTNODES[this.config.network] ?? '';
    const dataDir = this.config.paths.data;
    const args: string[] = [
      '--networkid', String(networkId),
      '--datadir', dataDir,
      '--port', String(this.config.ports.p2p),
      '--discovery.port', String(this.config.ports.p2p),
      '--syncmode', this.config.syncMode === 'light' ? 'snap' : 'full',
      '--http',
      '--http.addr', '127.0.0.1',
      '--http.port', String(this.config.ports.rpc),
      '--http.api', 'eth,net,txpool,web3,dbft',
      '--http.vhosts', '*',
    ];
    if (this.config.ports.websocket) {
      args.push(
        '--ws',
        '--ws.addr', '127.0.0.1',
        '--ws.port', String(this.config.ports.websocket),
        '--ws.api', 'eth,net,web3',
      );
    }
    if (bootnodes) {
      args.push('--bootnodes', bootnodes);
    }
    args.push('--verbosity', '3');
    return args;
  }

  getWorkingDirectory(): string {
    return this.config.paths.base;
  }

  /**
   * Get genesis path (used by `geth init` before first start). NodeManager
   * runs `geth init <genesis>` once on create — this method exposes the
   * expected location.
   */
  getGenesisPath(): string {
    return join(this.config.paths.config, `genesis_${this.config.network === 'neox-testnet' ? 'testnet' : 'mainnet'}.json`);
  }

  /**
   * geth log lines look like:
   *   INFO [05-02|11:22:33.456] Starting peer-to-peer node ...
   * with a leading single-character level token in some lines.
   */
  protected parseLogLine(line: string): LogEntry {
    const match = line.match(/^(?:\x1b\[\d+m)?(INFO|WARN|ERROR|DEBUG|TRACE|FATAL)?(?:\x1b\[0m)?\s*(?:\[([^\]]+)\])?\s*(.*)$/);
    if (match) {
      const [, levelStr, timeStr, message] = match;
      const level = mapGethLevel(levelStr);
      const ts = parseGethTimestamp(timeStr) ?? Date.now();
      return {
        timestamp: ts,
        level,
        source: 'geth',
        message: (message || line).trim(),
      };
    }
    return super.parseLogLine(line);
  }

  async getBlockHeight(): Promise<number | null> {
    try {
      const raw = await this.executeRpc('eth_blockNumber');
      const hex = JSON.parse(raw) as string;
      if (typeof hex !== 'string' || !hex.startsWith('0x')) return null;
      return Number.parseInt(hex.slice(2), 16);
    } catch {
      return null;
    }
  }

  async getPeersCount(): Promise<number | null> {
    try {
      const raw = await this.executeRpc('net_peerCount');
      const hex = JSON.parse(raw) as string;
      if (typeof hex !== 'string' || !hex.startsWith('0x')) return null;
      return Number.parseInt(hex.slice(2), 16);
    } catch {
      return null;
    }
  }

  async getChainId(): Promise<number | null> {
    try {
      const raw = await this.executeRpc('eth_chainId');
      const hex = JSON.parse(raw) as string;
      if (typeof hex !== 'string' || !hex.startsWith('0x')) return null;
      return Number.parseInt(hex.slice(2), 16);
    } catch {
      return null;
    }
  }
}

function mapGethLevel(level?: string): LogEntry['level'] {
  switch (level) {
    case 'ERROR':
    case 'FATAL':
      return 'error';
    case 'WARN':
      return 'warn';
    case 'DEBUG':
    case 'TRACE':
      return 'debug';
    default:
      return 'info';
  }
}

/**
 * geth uses `MM-DD|HH:MM:SS.sss` (no year). Synthesize a timestamp by combining
 * the current year with the parsed parts. If the parsed datetime is in the
 * future relative to now (clock skew at year boundaries), back off by a year.
 */
function parseGethTimestamp(input?: string): number | null {
  if (!input) return null;
  const match = input.match(/^(\d{2})-(\d{2})\|(\d{2}):(\d{2}):(\d{2})\.(\d+)$/);
  if (!match) return null;
  const [, mm, dd, hh, mi, ss, frac] = match;
  const now = new Date();
  let year = now.getUTCFullYear();
  const candidate = new Date(Date.UTC(
    year,
    Number(mm) - 1,
    Number(dd),
    Number(hh),
    Number(mi),
    Number(ss),
    Number(frac.padEnd(3, '0').slice(0, 3)),
  ));
  if (candidate.getTime() - now.getTime() > 24 * 60 * 60 * 1000) {
    year -= 1;
    return new Date(Date.UTC(
      year,
      Number(mm) - 1,
      Number(dd),
      Number(hh),
      Number(mi),
      Number(ss),
      Number(frac.padEnd(3, '0').slice(0, 3)),
    )).getTime();
  }
  return candidate.getTime();
}
