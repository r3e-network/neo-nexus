import type { SystemMetrics } from '../types/index';

interface SystemInformationFacade {
  currentLoad(): Promise<{ currentLoad: number }>;
  cpu(): Promise<{ cores: number }>;
  // `active` and `available` are reported by Linux/macOS/Windows; they
  // exclude buffers/cache so the dashboard doesn't read 96% on idle hosts.
  mem(): Promise<{
    total: number;
    used: number;
    free: number;
    active?: number;
    available?: number;
  }>;
  fsSize(): Promise<Array<{ size: number; used: number; use: number; mount?: string; type?: string; fs?: string }>>;
  networkStats(): Promise<Array<{ rx_bytes: number; tx_bytes: number }>>;
  processes(): Promise<{
    list: Array<{
      pid: number;
      name?: string;
      command?: string;
      cpu: number;
      memRss: number;
    }>;
  }>;
}

let systemInformationModule: Promise<SystemInformationFacade> | null = null;
const systemInformationPackageName: string = 'systeminformation';

async function getSystemInformation(): Promise<SystemInformationFacade> {
  systemInformationModule ??= import(systemInformationPackageName) as Promise<SystemInformationFacade>;
  return systemInformationModule;
}

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
    const si = await getSystemInformation();
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
   *
   * Linux's `mem.used` field counts buffers and disk cache against the
   * "used" total, so a healthy idle box reads as 90%+ utilized. That made
   * the dashboard's "Memory" tile alarm-looking even on hosts with plenty
   * of free RAM. Prefer `mem.active` (memory actually allocated to
   * processes) and fall back to `total - available` / `total - free` so
   * platforms that don't report active still get a reasonable number.
   */
  private async getMemoryMetrics(): Promise<SystemMetrics['memory']> {
    const si = await getSystemInformation();
    const mem = await si.mem();

    const active = typeof mem.active === 'number' && mem.active > 0 ? mem.active : null;
    const available = typeof mem.available === 'number' && mem.available > 0 ? mem.available : null;
    const used = active ?? (available !== null ? mem.total - available : mem.used);
    const free = available ?? mem.free;

    return {
      total: mem.total,
      used,
      free,
      percentage: Math.round((used / mem.total) * 100 * 10) / 10,
    };
  }

  /**
   * Get disk metrics
   *
   * fsSize() returns *every* mounted filesystem in implementation-defined
   * order — picking [0] gave whatever the kernel happened to enumerate
   * first, which on hosts with docker / overlay / tmpfs mounts could be
   * a 5MB tmpfs reading 0% used. Operationally the dashboard wants the
   * filesystem the operator cares about: the root volume. Prefer the
   * mount at "/", then fall back to the largest non-pseudo filesystem.
   */
  private async getDiskMetrics(): Promise<SystemMetrics['disk']> {
    const si = await getSystemInformation();
    const fs = await si.fsSize();

    // Skip pseudo / container-overlay filesystems; operators have no
    // signal-to-noise from those numbers on the dashboard.
    const PSEUDO_TYPES = new Set(['tmpfs', 'devtmpfs', 'overlay', 'squashfs', 'proc', 'sysfs', 'cgroup', 'cgroup2']);
    const real = fs.filter((entry) => !entry.type || !PSEUDO_TYPES.has(entry.type.toLowerCase()));

    const mainFs =
      real.find((entry) => entry.mount === '/') ??
      real.slice().sort((a, b) => (b.size || 0) - (a.size || 0))[0] ??
      fs[0];

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
    const si = await getSystemInformation();
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
      const si = await getSystemInformation();
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
      const si = await getSystemInformation();
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
