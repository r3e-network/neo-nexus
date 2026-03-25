import type Database from "better-sqlite3";
import { findAvailablePort } from "../utils/ports";

export interface PortAllocation {
  rpcPort: number;
  p2pPort: number;
}

export class PortAllocator {
  private baseRpcPort: number;
  private baseP2pPort: number;

  constructor(
    private db: Database.Database,
    baseRpcPort = 10333,
    baseP2pPort = 10334,
  ) {
    this.baseRpcPort = baseRpcPort;
    this.baseP2pPort = baseP2pPort;
  }

  async allocatePorts(): Promise<PortAllocation> {
    const usedPorts = this.getUsedPorts();

    const rpcPort = await findAvailablePort(this.findNextPort(this.baseRpcPort, usedPorts));

    usedPorts.add(rpcPort);

    const p2pPort = await findAvailablePort(this.findNextPort(this.baseP2pPort, usedPorts));

    return { rpcPort, p2pPort };
  }

  private getUsedPorts(): Set<number> {
    const ports = this.db.prepare("SELECT rpc_port, p2p_port FROM nodes").all() as Array<{
      rpc_port: number;
      p2p_port: number;
    }>;

    const used = new Set<number>();
    for (const { rpc_port, p2p_port } of ports) {
      used.add(rpc_port);
      used.add(p2p_port);
    }
    return used;
  }

  private findNextPort(basePort: number, usedPorts: Set<number>): number {
    let port = basePort;
    while (usedPorts.has(port)) {
      port++;
    }
    return port;
  }
}
