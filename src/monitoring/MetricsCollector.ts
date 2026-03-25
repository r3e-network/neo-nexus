import si from 'systeminformation';
import type { SystemMetrics } from '../types/index';

export class MetricsCollector {
  private lastNetworkStats: { rx: number; tx: number; time: number } | null = null;

  /**
   * Collect system-wide metrics
   */
  async collectSystemMetrics(): Promise<SystemMetrics> {
    const [cpu, mem, disk, network] = await Promise.all([
      this.getCpuMetrics(),
      this.getMemoryMetrics(),
      this.getDiskMetrics(),
      this.getNetworkMetrics(),
    ]);

    return {
      cpu,
      memory: mem,
      disk,
      network,
    };
  }

  /**
   * Get CPU metrics
   */
  private async getCpuMetrics(): Promise<{ usage: number; cores: number }> {
    const [currentLoad, cpuInfo] = await Promise.all([
      si.currentLoad(),
      si.cpu(),
    ]);

    return {
      usage: Math.round(currentLoad.currentLoad * 10) / 10,
      cores: cpuInfo.cores,
    };
  }

  /**
   * Get memory metrics
   */
  private async getMemoryMetrics(): Promise<SystemMetrics['memory']> {
    const mem = await si.mem();

    return {
      total: mem.total,
      used: mem.used,
      free: mem.free,
      percentage: Math.round((mem.used / mem.total) * 100 * 10) / 10,
    };
  }

  /**
   * Get disk metrics
   */
  private async getDiskMetrics(): Promise<SystemMetrics['disk']> {
    const fs = await si.fsSize();
    const mainFs = fs[0]; // Use first filesystem

    if (!mainFs) {
      return { total: 0, used: 0, free: 0, percentage: 0 };
    }

    return {
      total: mainFs.size,
      used: mainFs.used,
      free: mainFs.size - mainFs.used,
      percentage: Math.round(mainFs.use * 10) / 10,
    };
  }

  /**
   * Get network metrics (delta since last call)
   */
  private async getNetworkMetrics(): Promise<{ rx: number; tx: number }> {
    const network = await si.networkStats();
    const mainInterface = network[0];

    if (!mainInterface) {
      return { rx: 0, tx: 0 };
    }

    const now = Date.now();
    const currentStats = {
      rx: mainInterface.rx_bytes,
      tx: mainInterface.tx_bytes,
      time: now,
    };

    let result = { rx: 0, tx: 0 };

    if (this.lastNetworkStats) {
      const timeDiff = (now - this.lastNetworkStats.time) / 1000; // seconds
      if (timeDiff > 0) {
        result = {
          rx: Math.round((currentStats.rx - this.lastNetworkStats.rx) / timeDiff),
          tx: Math.round((currentStats.tx - this.lastNetworkStats.tx) / timeDiff),
        };
      }
    }

    this.lastNetworkStats = currentStats;
    return result;
  }

  /**
   * Get process-specific metrics
   */
  async getProcessMetrics(pid: number): Promise<{ cpu: number; memory: number; ppid?: number } | null> {
    try {
      const processes = await si.processes();
      const process = processes.list.find(p => p.pid === pid);

      if (!process) return null;

      return {
        cpu: process.cpu,
        memory: process.memRss,
        // ppid not available in this version
      };
    } catch {
      return null;
    }
  }

  /**
   * Get list of all processes
   */
  async getProcessList(): Promise<Array<{
    pid: number;
    name: string;
    cpu: number;
    memory: number;
    command?: string;
  }>> {
    try {
      const processes = await si.processes();
      return processes.list
        .filter(p => p.cpu > 0 || p.memRss > 0)
        .map(p => ({
          pid: p.pid,
          name: p.name || p.command || 'unknown',
          cpu: p.cpu,
          memory: p.memRss,
          command: p.command,
        }))
        .sort((a, b) => b.cpu - a.cpu)
        .slice(0, 50);
    } catch {
      return [];
    }
  }
}
