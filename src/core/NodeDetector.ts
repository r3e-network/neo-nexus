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
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import YAML from 'js-yaml';
import type { NodeType, NodeNetwork, PortConfig } from '../types';
import { Errors } from '../api/errors';

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
      throw Errors.pathNotFound(basePath);
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

    const configCandidates = [configPath, configTestnetPath, configMainnetPath];

    // Check if this looks like a neo-cli installation
    const hasExecutable = existsSync(neoCliDll) || existsSync(neoCliExe);

    // Single-pass: find and parse the first readable config file
    let network: NodeNetwork = 'testnet';
    const ports: Partial<PortConfig> = {};
    let dataPath = join(basePath, 'Data');
    let detectedConfigPath: string | undefined;

    for (const configFile of configCandidates) {
      if (!existsSync(configFile)) {
        continue;
      }
      detectedConfigPath ??= configFile;

      try {
        const config = JSON.parse(readFileSync(configFile, 'utf8'));

        if (config.ProtocolConfiguration?.Network !== undefined) {
          network = this.resolveNetworkFromMagic(config.ProtocolConfiguration.Network);
        }

        if (config.ApplicationConfiguration?.P2P?.Port) {
          ports.p2p = config.ApplicationConfiguration.P2P.Port;
        }
        if (config.ApplicationConfiguration?.RPC?.Port) {
          ports.rpc = config.ApplicationConfiguration.RPC.Port;
        }

        if (config.ApplicationConfiguration?.Storage?.Path) {
          dataPath = join(basePath, config.ApplicationConfiguration.Storage.Path);
        }

        break;
      } catch {
        // Continue to next config file
      }
    }

    if (!hasExecutable && !detectedConfigPath) {
      return null;
    }

    const version = this.detectNeoCliVersion(basePath);
    this.applyDefaultPorts(ports, network);

    return {
      type: 'neo-cli',
      network,
      version,
      ports,
      dataPath,
      configPath: detectedConfigPath ?? configPath,
      isRunning: this.isProcessRunning('neo-cli'),
    };
  }

  /**
   * Detect neo-go installation
   */
  private static detectNeoGo(basePath: string): DetectedNodeConfig | null {
    // Check for neo-go specific files
    const configPath = join(basePath, 'config.yaml');
    const protocolYmlPath = join(basePath, 'protocol.yml');
    const protocolYamlPath = join(basePath, 'protocol.yaml');
    const neoGoBinary = join(basePath, 'neo-go');
    const configCandidates = [configPath, protocolYmlPath, protocolYamlPath];

    // Check if this looks like a neo-go installation
    const hasBinary = existsSync(neoGoBinary);

    // Single-pass: find and parse the first readable config file
    let network: NodeNetwork = 'testnet';
    const ports: Partial<PortConfig> = {};
    let dataPath = join(basePath, 'data');
    let detectedConfigPath: string | undefined;

    for (const candidate of configCandidates) {
      if (!existsSync(candidate)) {
        continue;
      }
      detectedConfigPath ??= candidate;

      try {
        const parsedConfig = YAML.load(readFileSync(candidate, 'utf8'));
        const config = this.asRecord(parsedConfig);
        const protocolConfig = this.asRecord(config?.ProtocolConfiguration);
        const applicationConfig = this.asRecord(config?.ApplicationConfiguration);

        const networkMagic =
          this.asNumber(protocolConfig?.Magic) ?? this.asNumber(protocolConfig?.Network);
        if (networkMagic !== undefined) {
          network = this.resolveNetworkFromMagic(networkMagic);
        }

        const p2pConfig = this.asRecord(applicationConfig?.P2P);
        const rpcConfig = this.asRecord(applicationConfig?.RPC);
        const dbConfig = this.asRecord(applicationConfig?.DBConfiguration);
        const levelDbOptions = this.asRecord(dbConfig?.LevelDBOptions);

        const detectedP2pPort = this.extractAddressPort(p2pConfig?.Addresses);
        if (detectedP2pPort !== undefined) {
          ports.p2p = detectedP2pPort;
        }

        const detectedRpcPort = this.extractAddressPort(rpcConfig?.Addresses);
        if (detectedRpcPort !== undefined) {
          ports.rpc = detectedRpcPort;
        }

        const configuredDataDirectory = typeof levelDbOptions?.DataDirectoryPath === 'string'
          ? levelDbOptions.DataDirectoryPath
          : undefined;
        if (configuredDataDirectory) {
          dataPath = join(basePath, configuredDataDirectory);
        }

        break;
      } catch {
        // Use defaults if the config cannot be parsed.
      }
    }

    if (!hasBinary && !detectedConfigPath) {
      return null;
    }

    const version = this.detectNeoGoVersion(basePath);
    this.applyDefaultPorts(ports, network);

    return {
      type: 'neo-go',
      network,
      version,
      ports,
      dataPath,
      configPath: detectedConfigPath ?? protocolYmlPath,
      isRunning: this.isProcessRunning('neo-go'),
    };
  }

  private static detectNeoCliVersion(basePath: string): string {
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
        // Fall through to default
      }
    }

    return 'v3.6.0';
  }

  private static detectNeoGoVersion(basePath: string): string {
    try {
      const binary = join(basePath, 'neo-go');
      const output = execFileSync(binary, ['--version'], { encoding: 'utf8', timeout: 5000, stdio: ['ignore', 'pipe', 'pipe'] });
      const match = output.match(/v?(\d+\.\d+\.\d+)/);
      if (match) return match[1];
    } catch {
      // Fall through to default
    }
    return '0.104.0';
  }

  /**
   * Check if a process is running for this node
   */
  private static isProcessRunning(processName: string): boolean {
    try {
      const result = execFileSync('pgrep', ['-f', processName], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
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

  private static applyDefaultPorts(ports: Partial<PortConfig>, network: NodeNetwork): void {
    if (!ports.p2p) {
      ports.p2p = network === 'mainnet' ? 10333 : 20333;
    }
    if (!ports.rpc) {
      ports.rpc = network === 'mainnet' ? 10332 : 20332;
    }
  }

  private static resolveNetworkFromMagic(networkMagic: number): NodeNetwork {
    if (networkMagic === 860833102) {
      return 'mainnet';
    }
    if (networkMagic === 894710606) {
      return 'testnet';
    }
    return 'private';
  }

  private static extractAddressPort(addresses: unknown): number | undefined {
    if (!Array.isArray(addresses) || addresses.length === 0 || typeof addresses[0] !== 'string') {
      return undefined;
    }

    const match = addresses[0].match(/:(\d+)$/);
    if (!match) {
      return undefined;
    }

    const port = Number.parseInt(match[1], 10);
    return Number.isNaN(port) ? undefined : port;
  }

  private static asRecord(value: unknown): Record<string, unknown> | undefined {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return undefined;
    }

    return value as Record<string, unknown>;
  }

  private static asNumber(value: unknown): number | undefined {
    return typeof value === 'number' ? value : undefined;
  }
}
