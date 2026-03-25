import { existsSync, statSync, readdirSync, unlinkSync, mkdirSync } from 'node:fs';
import { stat, readdir, unlink, rmdir } from 'node:fs/promises';
import { join } from 'node:path';
import type { StorageInfo } from '../types/index';

export class StorageManager {
  /**
   * Get storage information for a node
   */
  static async getNodeStorageInfo(nodeId: string, paths: { data: string; logs: string; wallet?: string }): Promise<StorageInfo> {
    const [chainSize, logsInfo, walletCount] = await Promise.all([
      this.getDirectorySize(paths.data),
      this.getLogsInfo(paths.logs),
      paths.wallet ? this.countWallets(paths.wallet) : Promise.resolve(0),
    ]);

    return {
      chain: {
        size: chainSize,
        path: paths.data,
      },
      logs: logsInfo,
      wallets: {
        count: walletCount,
        path: paths.wallet || '',
      },
    };
  }

  /**
   * Clean old log files
   */
  static async cleanOldLogs(logsPath: string, maxAgeDays = 30): Promise<number> {
    if (!existsSync(logsPath)) return 0;

    const cutoffTime = Date.now() - (maxAgeDays * 24 * 60 * 60 * 1000);
    let cleanedCount = 0;

    const files = await readdir(logsPath);
    
    for (const file of files) {
      const filePath = join(logsPath, file);
      const stats = await stat(filePath);
      
      if (stats.isFile() && stats.mtime.getTime() < cutoffTime) {
        await unlink(filePath);
        cleanedCount++;
      }
    }

    return cleanedCount;
  }

  /**
   * Clean chain data (full reset)
   */
  static async cleanChainData(dataPath: string): Promise<void> {
    if (!existsSync(dataPath)) return;

    const entries = await readdir(dataPath);
    
    for (const entry of entries) {
      const entryPath = join(dataPath, entry);
      
      try {
        const stats = await stat(entryPath);
        if (stats.isDirectory()) {
          await this.removeDirectory(entryPath);
        } else {
          await unlink(entryPath);
        }
      } catch {
        // Ignore errors for individual files
      }
    }
  }

  /**
   * Get directory size recursively
   */
  static async getDirectorySize(dirPath: string): Promise<number> {
    if (!existsSync(dirPath)) return 0;

    let totalSize = 0;

    try {
      const entries = await readdir(dirPath);
      
      for (const entry of entries) {
        const entryPath = join(dirPath, entry);
        
        try {
          const stats = await stat(entryPath);
          if (stats.isDirectory()) {
            totalSize += await this.getDirectorySize(entryPath);
          } else {
            totalSize += stats.size;
          }
        } catch {
          // Ignore errors for individual entries
        }
      }
    } catch {
      return 0;
    }

    return totalSize;
  }

  /**
   * Get storage info for logs
   */
  private static async getLogsInfo(logsPath: string): Promise<{ size: number; files: number }> {
    if (!existsSync(logsPath)) {
      return { size: 0, files: 0 };
    }

    let totalSize = 0;
    let fileCount = 0;

    try {
      const files = await readdir(logsPath);
      
      for (const file of files) {
        const filePath = join(logsPath, file);
        
        try {
          const stats = await stat(filePath);
          if (stats.isFile()) {
            totalSize += stats.size;
            fileCount++;
          }
        } catch {
          // Ignore errors
        }
      }
    } catch {
      return { size: 0, files: 0 };
    }

    return { size: totalSize, files: fileCount };
  }

  /**
   * Count wallet files
   */
  private static async countWallets(walletPath: string): Promise<number> {
    if (!existsSync(walletPath)) return 0;

    try {
      const files = await readdir(walletPath);
      return files.filter(f => f.endsWith('.json') || f.endsWith('.db3')).length;
    } catch {
      return 0;
    }
  }

  /**
   * Remove directory recursively
   */
  private static async removeDirectory(dirPath: string): Promise<void> {
    const entries = await readdir(dirPath);
    
    for (const entry of entries) {
      const entryPath = join(dirPath, entry);
      
      const stats = await stat(entryPath);
      if (stats.isDirectory()) {
        await this.removeDirectory(entryPath);
      } else {
        await unlink(entryPath);
      }
    }

    await rmdir(dirPath);
  }

  /**
   * Format bytes to human readable string
   */
  static formatBytes(bytes: number, decimals = 2): string {
    if (bytes === 0) return '0 B';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
  }

  /**
   * Ensure all node directories exist
   */
  static ensureNodeDirectories(paths: { base: string; data: string; logs: string; config: string; wallet?: string }): void {
    mkdirSync(paths.base, { recursive: true });
    mkdirSync(paths.data, { recursive: true });
    mkdirSync(paths.logs, { recursive: true });
    mkdirSync(paths.config, { recursive: true });
    if (paths.wallet) {
      mkdirSync(paths.wallet, { recursive: true });
    }
  }

  /**
   * Check available disk space (Linux only)
   */
  static async getDiskSpace(path: string): Promise<{ total: number; free: number; used: number } | null> {
    try {
      const { execShell } = await import('../utils/exec');
      const { stdout } = await execShell(`df -B1 "${path}" | tail -1`);
      const parts = stdout.trim().split(/\s+/);
      
      if (parts.length >= 4) {
        return {
          total: parseInt(parts[1], 10),
          used: parseInt(parts[2], 10),
          free: parseInt(parts[3], 10),
        };
      }
    } catch {
      // Ignore errors
    }
    return null;
  }
}
