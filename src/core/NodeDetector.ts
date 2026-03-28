/**
 * NodeDetector - Detects node configuration from existing installations
 * 
 * This utility analyzes existing neo-cli or neo-go installations to extract:
 * - Node type (neo-cli vs neo-go)
 * - Network (mainnet vs testnet)
 * - Version
 * - Port configuration
 * - Data paths
 */

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import type { NodeType, NodeNetwork, PortConfig } from '../types';

export interface DetectedNodeConfig {
  type: NodeType;
  network: NodeNetwork;
  version: string;
  ports: Partial<PortConfig>;
  dataPath: string;
  configPath: string;
  isRunning: boolean;
}

export class NodeDetector {
  /**
   * Detect node configuration from a path
   */
  static detect(basePath: string): DetectedNodeConfig | null {
    // Validate path exists
    if (!existsSync(basePath)) {
      throw new Error(`Path does not exist: ${basePath}`);
    }

    // Try to detect as neo-cli
    const neoCliConfig = this.detectNeoCli(basePath);
    if (neoCliConfig) {
      return neoCliConfig;
    }

    // Try to detect as neo-go
    const neoGoConfig = this.detectNeoGo(basePath);
    if (neoGoConfig) {
      return neoGoConfig;
    }

    return null;
  }

  /**
   * Detect neo-cli installation
   */
  private static detectNeoCli(basePath: string): DetectedNodeConfig | null {
    // Check for neo-cli specific files
    const configPath = join(basePath, 'config.json');
    const configMainnetPath = join(basePath, 'config.mainnet.json');
    const configTestnetPath = join(basePath, 'config.testnet.json');
    const neoCliDll = join(basePath, 'neo-cli.dll');
    const neoCliExe = join(basePath, 'neo-cli');

    // Check if this looks like a neo-cli installation
    const hasExecutable = existsSync(neoCliDll) || existsSync(neoCliExe);
    const hasConfig = existsSync(configPath) || existsSync(configMainnetPath) || existsSync(configTestnetPath);

    if (!hasExecutable && !hasConfig) {
      return null;
    }

    // Detect network and ports from config
    let network: NodeNetwork = 'testnet';
    const ports: Partial<PortConfig> = {};
    let dataPath = join(basePath, 'Data');

    // Try to read and parse config
    const configFiles = [
      configPath,
      configTestnetPath,
      configMainnetPath,
    ];

    for (const configFile of configFiles) {
      if (existsSync(configFile)) {
        try {
          const config = JSON.parse(readFileSync(configFile, 'utf8'));
          
          // Detect network from ProtocolConfiguration.Network
          if (config.ProtocolConfiguration?.Network !== undefined) {
            const networkMagic = config.ProtocolConfiguration.Network;
            // Neo N3 network magics
            if (networkMagic === 860833102) {
              network = 'mainnet';
            } else if (networkMagic === 894710606) {
              network = 'testnet';
            } else {
              network = 'private';
            }
          }

          // Detect ports from ApplicationConfiguration
          if (config.ApplicationConfiguration?.P2P?.Port) {
            ports.p2p = config.ApplicationConfiguration.P2P.Port;
          }
          if (config.ApplicationConfiguration?.RPC?.Port) {
            ports.rpc = config.ApplicationConfiguration.RPC.Port;
          }

          // Detect data path
          if (config.ApplicationConfiguration?.Storage?.Path) {
            dataPath = join(basePath, config.ApplicationConfiguration.Storage.Path);
          }

          break; // Use first valid config
        } catch {
          // Continue to next config file
        }
      }
    }

    // Detect version from neo-cli.dll or other files
    const version = this.detectNeoCliVersion(basePath);

    // Default ports if not detected
    if (!ports.p2p) {
      ports.p2p = network === 'mainnet' ? 10333 : 20333;
    }
    if (!ports.rpc) {
      ports.rpc = network === 'mainnet' ? 10332 : 20332;
    }

    return {
      type: 'neo-cli',
      network,
      version,
      ports,
      dataPath,
      configPath: existsSync(configPath) ? configPath : configTestnetPath,
      isRunning: this.isProcessRunning(basePath, 'neo-cli'),
    };
  }

  /**
   * Detect neo-go installation
   */
  private static detectNeoGo(basePath: string): DetectedNodeConfig | null {
    // Check for neo-go specific files
    const configPath = join(basePath, 'config.yaml');
    const configProtocolPath = join(basePath, 'protocol.yaml');
    const neoGoBinary = join(basePath, 'neo-go');

    // Check if this looks like a neo-go installation
    const hasBinary = existsSync(neoGoBinary);
    const hasConfig = existsSync(configPath) || existsSync(configProtocolPath);

    if (!hasBinary && !hasConfig) {
      return null;
    }

    // Detect network and ports from config
    let network: NodeNetwork = 'testnet';
    const ports: Partial<PortConfig> = {};
    const dataPath = join(basePath, 'data');

    // Try to read and parse config.yaml
    if (existsSync(configPath)) {
      try {
        const configContent = readFileSync(configPath, 'utf8');
        
        // Parse YAML-like content (simplified)
        const networkMatch = configContent.match(/Network:\s*(\d+)/);
        if (networkMatch) {
          const networkMagic = parseInt(networkMatch[1], 10);
          if (networkMagic === 860833102) {
            network = 'mainnet';
          } else if (networkMagic === 894710606) {
            network = 'testnet';
          } else {
            network = 'private';
          }
        }

        // Extract ports
        const p2pMatch = configContent.match(/Port:\s*(\d+)/);
        if (p2pMatch) {
          ports.p2p = parseInt(p2pMatch[1], 10);
        }

        const rpcMatch = configContent.match(/RPC:\s*[\s\S]*?Port:\s*(\d+)/);
        if (rpcMatch) {
          ports.rpc = parseInt(rpcMatch[1], 10);
        }

      } catch {
        // Use defaults
      }
    }

    // Detect version
    const version = this.detectNeoGoVersion(basePath);

    // Default ports if not detected
    if (!ports.p2p) {
      ports.p2p = network === 'mainnet' ? 10333 : 20333;
    }
    if (!ports.rpc) {
      ports.rpc = network === 'mainnet' ? 10332 : 20332;
    }

    return {
      type: 'neo-go',
      network,
      version,
      ports,
      dataPath,
      configPath: existsSync(configPath) ? configPath : configProtocolPath,
      isRunning: this.isProcessRunning(basePath, 'neo-go'),
    };
  }

  /**
   * Detect neo-cli version from installation
   */
  private static detectNeoCliVersion(basePath: string): string {
    // Try to get version from neo-cli.deps.json
    const depsPath = join(basePath, 'neo-cli.deps.json');
    if (existsSync(depsPath)) {
      try {
        const deps = JSON.parse(readFileSync(depsPath, 'utf8'));
        if (deps?.runtimeTarget?.name) {
          const match = deps.runtimeTarget.name.match(/neo-cli\/(\d+\.\d+\.\d+)/);
          if (match) {
            return `v${match[1]}`;
          }
        }
      } catch {
        // Fall through to next method
      }
    }

    // Try to infer from Neo.dll version
    const neoDllPath = join(basePath, 'Neo.dll');
    if (existsSync(neoDllPath)) {
      // Version detection would require assembly reading
      // For now, return a default
      return 'v3.6.0';
    }

    return 'v3.6.0';
  }

  /**
   * Detect neo-go version from installation
   */
  private static detectNeoGoVersion(basePath: string): string {
    // Try to get version from binary
    const neoGoBinary = join(basePath, 'neo-go');
    if (existsSync(neoGoBinary)) {
      // In real implementation, would run: ./neo-go --version
      // For now, return a default
      return '0.104.0';
    }

    return '0.104.0';
  }

  /**
   * Check if a process is running for this node
   */
  private static isProcessRunning(basePath: string, processName: string): boolean {
    // This is a simplified check - in production would use ps-list or similar
    try {
      const { execSync } = require('node:child_process');
      const result = execSync(`pgrep -f "${processName}" || true`, { encoding: 'utf8' });
      return result.trim().length > 0;
    } catch {
      return false;
    }
  }

  /**
   * Scan a directory for potential node installations
   */
  static scanDirectory(basePath: string): Array<{ path: string; type: NodeType | null }> {
    const results: Array<{ path: string; type: NodeType | null }> = [];

    if (!existsSync(basePath)) {
      return results;
    }

    try {
      const entries = readdirSync(basePath, { withFileTypes: true });
      
      for (const entry of entries) {
        if (entry.isDirectory()) {
          const fullPath = join(basePath, entry.name);
          const detected = this.detect(fullPath);
          
          if (detected) {
            results.push({ path: fullPath, type: detected.type });
          }
        }
      }
    } catch {
      // Directory not readable
    }

    return results;
  }

  /**
   * Validate that an imported node configuration is usable
   */
  static validateImport(config: DetectedNodeConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    // Check if data path exists
    if (!existsSync(config.dataPath)) {
      errors.push(`Data path does not exist: ${config.dataPath}`);
    }

    // Check if config file exists
    if (!existsSync(config.configPath)) {
      errors.push(`Config file does not exist: ${config.configPath}`);
    }

    // Validate ports are in valid range
    if (config.ports.p2p && (config.ports.p2p < 1 || config.ports.p2p > 65535)) {
      errors.push(`Invalid P2P port: ${config.ports.p2p}`);
    }
    if (config.ports.rpc && (config.ports.rpc < 1 || config.ports.rpc > 65535)) {
      errors.push(`Invalid RPC port: ${config.ports.rpc}`);
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }
}
