import { existsSync } from "node:fs";
import { join } from "node:path";
import { BaseNode } from "./BaseNode";
import { DownloadManager } from "../core/DownloadManager";
import type { NodeConfig, LogEntry } from "../types/index";

export class NeoCliNode extends BaseNode {
  constructor(config: NodeConfig) {
    super(config);
  }

  async getBinaryPath(): Promise<string> {
    // neo-cli requires a pseudo-TTY because it uses Console.ReadKey().
    // Use 'script' to provide one when running non-interactively.
    return "script";
  }

  getStartArgs(): string[] {
    // Prefer neo-cli.dll in the node's own directory (supports per-node binary installs
    // where plugins are resolved from the same directory as the assembly).
    const localDll = join(this.config.paths.base, "neo-cli.dll");
    let dllPath: string;

    if (existsSync(localDll)) {
      dllPath = localDll;
    } else {
      const downloadPath = DownloadManager.getNodeBinaryPath("neo-cli", this.config.version);
      if (!downloadPath) {
        throw new Error(`neo-cli ${this.config.version} is not downloaded`);
      }
      dllPath = join(downloadPath, "neo-cli.dll");
    }

    const dotnetArgs: string[] = ["dotnet", dllPath];

    // Add RPC port if configured
    if (this.config.ports.rpc) {
      dotnetArgs.push("--rpc");
    }

    // Add log level
    if (this.config.settings.debugMode) {
      dotnetArgs.push("--log-level", "Debug");
    } else {
      dotnetArgs.push("--log-level", "Info");
    }

    // `script -qfc "command" /dev/null` provides a pseudo-TTY
    return ["-qfc", dotnetArgs.join(" "), "/dev/null"];
  }

  getWorkingDirectory(): string {
    return this.config.paths.base;
  }

  /**
   * Parse neo-cli specific log format
   */
  protected parseLogLine(line: string): LogEntry {
    // neo-cli log format: [TIMESTAMP] [LEVEL] MESSAGE
    // Example: [11:23:45] [INFO] Block 123456 persisted

    const match = line.match(/^\[(\d{2}:\d{2}:\d{2})\]\s*\[(\w+)\]\s*(.+)$/);

    if (match) {
      const [, timeStr, levelStr, message] = match;

      let level: LogEntry["level"] = "info";
      const lowerLevel = levelStr.toLowerCase();

      if (lowerLevel === "error" || lowerLevel === "fatal") {
        level = "error";
      } else if (lowerLevel === "warn" || lowerLevel === "warning") {
        level = "warn";
      } else if (lowerLevel === "debug") {
        level = "debug";
      }

      // Parse time (assume today)
      const [hours, minutes, seconds] = timeStr.split(":").map(Number);
      const timestamp = new Date();
      timestamp.setHours(hours, minutes, seconds, 0);

      return {
        timestamp: timestamp.getTime(),
        level,
        source: "neo-cli",
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
      const result = await this.executeRpc("getblockcount");
      const parsed = JSON.parse(result) as number;
      return typeof parsed === "number" ? parsed : null;
    } catch {
      return null;
    }
  }

  /**
   * Get connected peers via RPC
   */
  async getPeersCount(): Promise<number | null> {
    try {
      const result = await this.executeRpc("getpeers");
      const peers = JSON.parse(result);
      return peers.connected?.length ?? 0;
    } catch {
      return null;
    }
  }

  /**
   * Check if dotnet runtime is installed
   */
  static async checkDotnetRuntime(): Promise<{ installed: boolean; version?: string }> {
    try {
      const { execShell } = await import("../utils/exec");
      const { stdout } = await execShell("dotnet --version");
      return { installed: true, version: stdout.trim() };
    } catch {
      return { installed: false };
    }
  }

  /**
   * Install dotnet runtime (Ubuntu/Debian)
   */
  static async installDotnetRuntime(): Promise<void> {
    const { execShell } = await import("../utils/exec");

    // Add Microsoft package repository
    await execShell(
      "wget https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb",
    );
    await execShell("sudo dpkg -i packages-microsoft-prod.deb");
    await execShell("rm packages-microsoft-prod.deb");

    // Install .NET 8.0
    await execShell("sudo apt-get update");
    await execShell("sudo apt-get install -y dotnet-runtime-8.0");
  }
}
