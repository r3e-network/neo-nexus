import { isPortAvailable } from '../utils/network';
import type { PortConfig } from '../types/index';

const BASE_RPC_PORT = 10332;
const BASE_P2P_PORT = 10333;
const BASE_WS_PORT = 10334;
const BASE_METRICS_PORT = 2112;
const PORT_INCREMENT = 10;

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
   * Allocate ports for a new node
   */
  async allocatePorts(nodeIndex: number): Promise<PortConfig> {
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
        throw new Error(`Port ${port} (${name}) is already in use by another node`);
      }
      
      const available = await isPortAvailable(port);
      if (!available) {
        throw new Error(`Port ${port} (${name}) is already in use by another process`);
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

  /**
   * Release ports when a node is removed
   */
  releasePorts(ports: PortConfig): void {
    this.usedPorts.delete(ports.rpc);
    this.usedPorts.delete(ports.p2p);
    if (ports.websocket) this.usedPorts.delete(ports.websocket);
    if (ports.metrics) this.usedPorts.delete(ports.metrics);
  }

  /**
   * Find next available node index
   */
  async findNextIndex(maxNodes = 100): Promise<number> {
    for (let i = 0; i < maxNodes; i++) {
      const baseOffset = i * PORT_INCREMENT;
      const rpcPort = BASE_RPC_PORT + baseOffset;
      
      if (!this.usedPorts.has(rpcPort) && await isPortAvailable(rpcPort)) {
        return i;
      }
    }
    throw new Error(`No available port range found (max ${maxNodes} nodes)`);
  }

  /**
   * Validate custom ports
   */
  async validateCustomPorts(ports: Partial<PortConfig>): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = [];

    if (ports.rpc) {
      if (this.usedPorts.has(ports.rpc)) {
        errors.push(`RPC port ${ports.rpc} is already in use`);
      } else if (!(await isPortAvailable(ports.rpc))) {
        errors.push(`RPC port ${ports.rpc} is unavailable`);
      }
    }

    if (ports.p2p) {
      if (this.usedPorts.has(ports.p2p)) {
        errors.push(`P2P port ${ports.p2p} is already in use`);
      } else if (!(await isPortAvailable(ports.p2p))) {
        errors.push(`P2P port ${ports.p2p} is unavailable`);
      }
    }

    if (ports.websocket) {
      if (this.usedPorts.has(ports.websocket)) {
        errors.push(`WebSocket port ${ports.websocket} is already in use`);
      } else if (!(await isPortAvailable(ports.websocket))) {
        errors.push(`WebSocket port ${ports.websocket} is unavailable`);
      }
    }

    if (ports.metrics) {
      if (this.usedPorts.has(ports.metrics)) {
        errors.push(`Metrics port ${ports.metrics} is already in use`);
      } else if (!(await isPortAvailable(ports.metrics))) {
        errors.push(`Metrics port ${ports.metrics} is unavailable`);
      }
    }

    return { valid: errors.length === 0, errors };
  }
}
