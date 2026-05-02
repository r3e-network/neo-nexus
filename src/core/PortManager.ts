import { isPortAvailable } from '../utils/ports';
import type { NodeChain, PortConfig } from '../types/index';
import { Errors } from '../api/errors';

const BASE_RPC_PORT = 10332;
const BASE_P2P_PORT = 10333;
const BASE_WS_PORT = 10334;
const BASE_METRICS_PORT = 2112;
const PORT_INCREMENT = 10;

// Neo X (geth) defaults to a separate port range so N3 + X can coexist on the
// same host without bumping each other.
const NEOX_BASE_RPC_PORT = 8551;
const NEOX_BASE_P2P_PORT = 30303;
const NEOX_BASE_WS_PORT = 8571;
const NEOX_BASE_AUTHRPC_PORT = 8561;
const NEOX_PORT_INCREMENT = 10;

export class PortManager {
  private usedPorts: Set<number> = new Set();

  constructor(existingPorts: PortConfig[] = []) {
    // Track existing ports as used
    for (const ports of existingPorts) {
      this.usedPorts.add(ports.rpc);
      this.usedPorts.add(ports.p2p);
      if (ports.websocket) this.usedPorts.add(ports.websocket);
      if (ports.metrics) this.usedPorts.add(ports.metrics);
    }
  }

  /**
   * Allocate ports for a new node. Pass `chain` so Neo X nodes get the
   * geth-style 8551/30303 range instead of the Neo N3 10332/10333 range.
   */
  async allocatePorts(nodeIndex: number, chain: NodeChain = 'n3'): Promise<PortConfig> {
    if (chain === 'x') return this.allocateNeoXPorts(nodeIndex);

    const baseOffset = nodeIndex * PORT_INCREMENT;

    const rpc = BASE_RPC_PORT + baseOffset;
    const p2p = BASE_P2P_PORT + baseOffset;
    const websocket = BASE_WS_PORT + baseOffset;
    const metrics = BASE_METRICS_PORT + baseOffset;

    // Verify ports are available
    const portsToCheck = [
      { name: 'RPC', port: rpc },
      { name: 'P2P', port: p2p },
      { name: 'WebSocket', port: websocket },
      { name: 'Metrics', port: metrics },
    ];

    for (const { name, port } of portsToCheck) {
      if (this.usedPorts.has(port)) {
        throw Errors.portConflictNode(port, name);
      }

      const available = await isPortAvailable(port);
      if (!available) {
        throw Errors.portConflictSystem(port, name);
      }
    }

    // Mark ports as used
    this.usedPorts.add(rpc);
    this.usedPorts.add(p2p);
    this.usedPorts.add(websocket);
    this.usedPorts.add(metrics);

    return {
      rpc,
      p2p,
      websocket,
      metrics,
    };
  }

  private async allocateNeoXPorts(nodeIndex: number): Promise<PortConfig> {
    const offset = nodeIndex * NEOX_PORT_INCREMENT;
    const rpc = NEOX_BASE_RPC_PORT + offset;
    const p2p = NEOX_BASE_P2P_PORT + offset;
    const websocket = NEOX_BASE_WS_PORT + offset;
    const metrics = NEOX_BASE_AUTHRPC_PORT + offset;
    const portsToCheck = [
      { name: 'RPC', port: rpc },
      { name: 'P2P', port: p2p },
      { name: 'WebSocket', port: websocket },
      { name: 'AuthRPC', port: metrics },
    ];
    for (const { name, port } of portsToCheck) {
      if (this.usedPorts.has(port)) throw Errors.portConflictNode(port, name);
      if (!(await isPortAvailable(port))) throw Errors.portConflictSystem(port, name);
    }
    this.usedPorts.add(rpc);
    this.usedPorts.add(p2p);
    this.usedPorts.add(websocket);
    this.usedPorts.add(metrics);
    return { rpc, p2p, websocket, metrics };
  }

  /**
   * Release ports when a node is removed
   */
  releasePorts(ports: PortConfig): void {
    this.usedPorts.delete(ports.rpc);
    this.usedPorts.delete(ports.p2p);
    if (ports.websocket) this.usedPorts.delete(ports.websocket);
    if (ports.metrics) this.usedPorts.delete(ports.metrics);
  }

  async reservePorts(ports: PortConfig): Promise<void> {
    const validation = await this.validateCustomPorts(ports);
    if (!validation.valid) {
      throw Errors.invalidPortConfig(validation.errors);
    }

    this.usedPorts.add(ports.rpc);
    this.usedPorts.add(ports.p2p);
    if (ports.websocket) this.usedPorts.add(ports.websocket);
    if (ports.metrics) this.usedPorts.add(ports.metrics);
  }

  /**
   * Find next available node index
   */
  async findNextIndex(maxNodes = 100, chain: NodeChain = 'n3'): Promise<number> {
    const base = chain === 'x' ? NEOX_BASE_RPC_PORT : BASE_RPC_PORT;
    const inc = chain === 'x' ? NEOX_PORT_INCREMENT : PORT_INCREMENT;
    for (let i = 0; i < maxNodes; i++) {
      const rpcPort = base + i * inc;
      if (!this.usedPorts.has(rpcPort) && await isPortAvailable(rpcPort)) {
        return i;
      }
    }
    throw Errors.noPortRange();
  }

  /**
   * Validate custom ports
   */
  async validateCustomPorts(ports: Partial<PortConfig>): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = [];
    const entries = [
      { name: 'RPC', port: ports.rpc },
      { name: 'P2P', port: ports.p2p },
      { name: 'WebSocket', port: ports.websocket },
      { name: 'Metrics', port: ports.metrics },
    ].filter((entry): entry is { name: string; port: number } => entry.port !== undefined);
    const seen = new Map<number, string>();

    for (const { name, port } of entries) {
      if (!Number.isInteger(port) || port < 1 || port > 65535) {
        errors.push(`${name} port ${port} is invalid`);
        continue;
      }
      const existingName = seen.get(port);
      if (existingName) {
        errors.push(`${name} port ${port} duplicates ${existingName} port`);
        continue;
      }
      seen.set(port, name);
    }

    for (const { name, port } of entries) {
      if (this.usedPorts.has(port)) {
        errors.push(`${name} port ${port} is already in use`);
      } else if (!(await isPortAvailable(port))) {
        errors.push(`${name} port ${port} is unavailable`);
      }
    }

    return { valid: errors.length === 0, errors };
  }
}
