import { writeFileSync, mkdirSync, existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import YAML from 'js-yaml';
import type { NodeConfig, NodeNetwork, PluginId } from '../types/index';
import { getNetworkMagic, getSeedList } from '../utils/network';
import { NEO_GO_STANDBY_COMMITTEE, NEO_GO_HARDFORKS } from '../data/neo-committee';

export class ConfigManager {
  static getExpectedHardforks(network: NodeNetwork): Record<string, number> | undefined {
    if (network === 'private') return undefined;
    return NEO_GO_HARDFORKS[network];
  }

  /**
   * Generate neo-cli config.json
   */
  static async generateNeoCliConfig(node: NodeConfig, installedPlugins: PluginId[] = []): Promise<Record<string, unknown>> {
    const networkMagic = getNetworkMagic(node.network);

    // Try to load the official config from the downloaded neo-cli binary as a base.
    // neo-cli ships with config.testnet.json / config.mainnet.json that contain the
    // correct StandbyCommittee keys, NNS contract hash, and hardfork heights.
    let baseConfig: Record<string, unknown> = {};
    try {
      const { DownloadManager } = await import('./DownloadManager');
      const binaryDir = DownloadManager.getNodeBinaryPath('neo-cli', node.version);
      if (binaryDir) {
        const networkSuffix = node.network === 'private' ? 'mainnet' : node.network;
        const officialConfigPath = join(binaryDir, `config.${networkSuffix}.json`);
        if (existsSync(officialConfigPath)) {
          baseConfig = JSON.parse(readFileSync(officialConfigPath, 'utf-8'));
        }
      }
    } catch {
      // Fall back to generated config
    }

    const baseApp = (baseConfig as any)?.ApplicationConfiguration || {};

    const config: Record<string, unknown> = {
      ApplicationConfiguration: {
        ...baseApp,
        Logger: {
          Path: 'Logs',
          ConsoleOutput: true,
          Active: true,
        },
        Storage: {
          Engine: installedPlugins.includes('RocksDBStore') ? 'RocksDBStore' : 'LevelDBStore',
          Path: 'Data',
        },
        P2P: {
          ...(baseApp.P2P || {}),
          Port: node.ports.p2p,
          MinDesiredConnections: node.settings.minPeers ?? 10,
          MaxConnections: node.settings.maxPeers ?? 40,
          MaxConnectionsPerAddress: 3,
        },
        UnlockWallet: {
          Path: '',
          Password: '',
          IsActive: false,
        },
        Contracts: baseApp.Contracts || {
          NeoNameService: '0x50ac1c37690cc2cfc594472833cf57505d5f46de',
        },
        Plugins: {
          Enabled: installedPlugins,
        },
      },
      ProtocolConfiguration: (baseConfig as any)?.ProtocolConfiguration || {
        Network: networkMagic,
        AddressVersion: 53,
        MillisecondsPerBlock: 15000,
        MaxTransactionsPerBlock: 512,
        MemoryPoolMaxTransactions: 50000,
        MaxTraceableBlocks: 2102400,
        ValidatorsCount: 7,
        StandbyCommittee: [
          '03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c',
          '02df48f60e8f3e01c48ff40b9b7f1310d7a8b2a193188befe1c2e3df740e895093',
          '03b8d9d5771d8f513aa0869b9cc8d50986403b78c6da36890638c3d46a5adce04c',
          '02ca0e27697b9c248f6f16e085fd0061e26f44da85b58ee835c110caa5ec3ba554',
          '02cd5a553437e8dd650b566d5dd07cffe5dfad38e2d2fb42d71a2c6c86fedf3204',
          '02635c9f5b5b23730f9ff3f83b0da4592a74b5f44f3ce47a50f95c8ad950cf2150',
          '038b3d1535c76c6f6d24c0d1dd871b05b5274ec08f7840b8b222b50e3e6724dd20',
        ],
        SeedList: node.network === 'private' ? [] : getSeedList(node.network),
      },
    };

    // Remove undefined values
    return this.removeUndefined(config) as Record<string, unknown>;
  }

  /**
   * Generate neo-go protocol.yml
   */
  static generateNeoGoConfig(node: NodeConfig): Record<string, unknown> {
    const networkMagic = getNetworkMagic(node.network);
    const standbyCommittee =
      node.network === 'private' ? [] : NEO_GO_STANDBY_COMMITTEE[node.network];
    const hardforks = node.network === 'private' ? undefined : NEO_GO_HARDFORKS[node.network];

    const config: Record<string, unknown> = {
      ProtocolConfiguration: {
        Magic: networkMagic,
        MaxTraceableBlocks: 2102400,
        InitialGASSupply: 52000000,
        MaxBlockSystemFee: 2000000000,
        TimePerBlock: '15s',
        Genesis: {
          TimePerBlock: '15s',
        },
        MemPoolSize: 50000,
        ValidatorsCount: 7,
        StandbyCommittee: standbyCommittee,
        SeedList: node.network === 'private' ? [] : getSeedList(node.network),
        VerifyTransactions: false,
        P2PSigExtensions: false,
        Hardforks: hardforks,
      },
      ApplicationConfiguration: {
        SkipBlockVerification: false,
        DBConfiguration: {
          Type: 'leveldb',
          LevelDBOptions: {
            DataDirectoryPath: './data',
          },
        },
        P2P: {
          Addresses: [`:${node.ports.p2p}`],
          DialTimeout: '3s',
          ProtoTickInterval: '2s',
          PingInterval: '30s',
          PingTimeout: '90s',
          MaxPeers: node.settings.maxPeers ?? 100,
          AttemptConnPeers: node.settings.minPeers ?? 20,
          MinPeers: node.settings.minPeers ?? 10,
        },
        Relay: node.settings.relay ?? true,
        Consensus: {
          Enabled: false,
          UnlockWallet: {
            Path: '/cn_wallet.json',
            Password: 'pass',
          },
        },
        Oracle: {
          Enabled: false,
          AllowedContentTypes: ['application/json'],
        },
        P2PNotary: {
          Enabled: false,
          UnlockWallet: {
            Path: '/notary_wallet.json',
            Password: 'pass',
          },
        },
        RPC: {
          Enabled: true,
          Addresses: [`:${node.ports.rpc}`],
          MaxGasInvoke: 15,
          EnableCORSWorkaround: false,
          TLSConfig: {
            Enabled: false,
            CertFile: 'serv.crt',
            KeyFile: 'serv.key',
          },
        },
        Prometheus: node.ports.metrics
          ? {
              Enabled: true,
              Addresses: [`:${node.ports.metrics}`],
            }
          : undefined,
      },
    };

    // Remove undefined values
    return this.removeUndefined(config) as Record<string, unknown>;
  }

  /**
   * Generate plugin config.json for neo-cli
   */
  static generatePluginConfig(
    pluginId: PluginId,
    node: NodeConfig,
    customConfig: Record<string, unknown> = {},
  ): object {
    const networkMagic = getNetworkMagic(node.network);

    const configs: Record<PluginId, object> = {
      ApplicationLogs: {
        Path: 'ApplicationLogs_{0}',
        Network: networkMagic,
        MaxStackSize: 65535,
        Debug: false,
        UnhandledExceptionPolicy: 'StopPlugin',
      },
      DBFTPlugin: {
        RecoveryLogs: 'ConsensusState',
        IgnoreRecoveryLogs: false,
        AutoStart: false,
        Network: networkMagic,
        MaxBlockSize: 2097152,
        MaxBlockSystemFee: 150000000000,
        UnhandledExceptionPolicy: 'StopNode',
      },
      LevelDBStore: {},
      OracleService: {
        Network: networkMagic,
        AutoStart: false,
        UnhandledExceptionPolicy: 'StopPlugin',
      },
      RestServer: {
        Network: networkMagic,
        BindAddress: '0.0.0.0',
        Port: node.ports.websocket ?? 10334,
        KeepAliveTimeout: 120,
        EnableCors: true,
        AllowOrigins: [],
        MaxConcurrentConnections: 40,
        MaxGasInvoke: 200000000,
        EnableSwagger: true,
        MaxPageSize: 50,
        EnableCompression: true,
        UnhandledExceptionPolicy: 'StopPlugin',
      },
      RocksDBStore: {},
      RpcServer: {
        UnhandledExceptionPolicy: 'Ignore',
        Servers: [
          {
            Network: networkMagic,
            BindAddress: '0.0.0.0',
            Port: node.ports.rpc,
            MaxConcurrentConnections: node.settings.maxConnections ?? 40,
            KeepAliveTimeout: 60,
            RequestHeadersTimeout: 15,
            MaxGasInvoke: 20,
            MaxFee: 0.1,
            MaxIteratorResultItems: 100,
            MaxStackSize: 65535,
            EnableCors: true,
            AllowOrigins: [],
            DisabledMethods: [],
            SessionEnabled: false,
            SessionExpirationTime: 60,
            FindStoragePageSize: 50,
          },
        ],
      },
      SignClient: {
        PluginConfiguration: {
          Name:
            (customConfig.PluginConfiguration as Record<string, unknown> | undefined)?.Name ??
            customConfig.Name ??
            'SignClient',
          Endpoint:
            (customConfig.PluginConfiguration as Record<string, unknown> | undefined)?.Endpoint ??
            customConfig.Endpoint ??
            'http://127.0.0.1:9991',
        },
      },
      SQLiteWallet: {},
      StateService: {
        Path: 'Data_MPT_{0}',
        FullState: false,
        Network: networkMagic,
        AutoVerify: false,
        MaxFindResultItems: 100,
        UnhandledExceptionPolicy: 'StopPlugin',
      },
      StorageDumper: {},
      TokensTracker: {
        DBPath: 'TokenBalanceData',
        TrackHistory: true,
        MaxResults: 1000,
        Network: networkMagic,
        EnabledTrackers: ['NEP-11', 'NEP-17'],
        UnhandledExceptionPolicy: 'StopPlugin',
      },
    };

    if (pluginId === 'SignClient') {
      return configs[pluginId];
    }

    return this.mergeObjects(configs[pluginId] ?? {}, customConfig);
  }

  /**
   * Write node configuration files
   */
  static async writeNodeConfig(node: NodeConfig, installedPlugins: PluginId[] = []): Promise<void> {
    // Ensure config directory exists
    mkdirSync(node.paths.config, { recursive: true });

    if (node.type === 'neo-cli') {
      const config = await this.generateNeoCliConfig(node, installedPlugins);
      const configJson = JSON.stringify(config, null, 2);
      writeFileSync(join(node.paths.config, 'config.json'), configJson);
      // neo-cli looks for config.json in its working directory (the node base path)
      writeFileSync(join(node.paths.base, 'config.json'), configJson);
    } else {
      const config = this.generateNeoGoConfig(node);
      writeFileSync(
        join(node.paths.config, 'protocol.yml'),
        YAML.dump(config, { lineWidth: -1 })
      );
    }
  }

  /**
   * Write plugin configuration
   */
  static writePluginConfig(
    pluginId: PluginId,
    node: NodeConfig,
    customConfig: Record<string, unknown> = {},
  ): void {
    if (node.type !== 'neo-cli') return;

    const config = this.generatePluginConfig(pluginId, node, customConfig);
    const pluginDir = join(node.paths.base, 'Plugins', pluginId);

    mkdirSync(pluginDir, { recursive: true });

    // neo-cli plugins expect {PluginName}.json with PluginConfiguration wrapper
    const wrappedConfig = pluginId === 'SignClient' ? config : { PluginConfiguration: config };

    writeFileSync(
      join(pluginDir, `${pluginId}.json`),
      JSON.stringify(wrappedConfig, null, 2)
    );
  }

  /**
   * Remove undefined values from object recursively
   */
  private static removeUndefined(obj: unknown): Record<string, unknown> | unknown {
    if (Array.isArray(obj)) {
      return obj.map(item => this.removeUndefined(item));
    }
    
    if (obj && typeof obj === 'object') {
      const result: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(obj)) {
        if (value !== undefined) {
          result[key] = this.removeUndefined(value);
        }
      }
      return result;
    }
    
    return obj;
  }

  private static mergeObjects(base: object, override: Record<string, unknown>): object {
    const result: Record<string, unknown> = { ...base } as Record<string, unknown>;

    for (const [key, value] of Object.entries(override)) {
      if (value && typeof value === 'object' && !Array.isArray(value)) {
        const current = result[key];
        if (current && typeof current === 'object' && !Array.isArray(current)) {
          result[key] = this.mergeObjects(
            current as Record<string, unknown>,
            value as Record<string, unknown>,
          );
        } else {
          result[key] = this.mergeObjects({}, value as Record<string, unknown>);
        }
      } else {
        result[key] = value;
      }
    }

    return result;
  }

  /**
   * Audit a node's on-disk config against the expected generated config.
   * Returns a list of differences so users know what changed, what might
   * conflict, and what needs attention after a version upgrade.
   */
  static async auditNodeConfig(
    node: NodeConfig,
    installedPlugins: PluginId[] = [],
  ): Promise<ConfigAuditResult> {
    const issues: ConfigAuditIssue[] = [];

    if (node.type === 'neo-cli') {
      const expectedConfig = await this.generateNeoCliConfig(node, installedPlugins);
      const onDiskPath = join(node.paths.base, 'config.json');
      if (existsSync(onDiskPath)) {
        const onDisk = JSON.parse(readFileSync(onDiskPath, 'utf-8'));
        this.diffConfigs(expectedConfig, onDisk, '', issues);

        // Check Plugins.Enabled matches installed
        const diskPlugins: string[] = onDisk?.ApplicationConfiguration?.Plugins?.Enabled || [];
        const expectedPluginIds = installedPlugins as string[];
        for (const p of diskPlugins) {
          if (!expectedPluginIds.includes(p)) {
            issues.push({ path: `Plugins.Enabled.${p}`, severity: 'warning', message: `Plugin "${p}" is in config but not registered as installed` });
          }
        }
        for (const p of expectedPluginIds) {
          if (!diskPlugins.includes(p)) {
            issues.push({ path: `Plugins.Enabled.${p}`, severity: 'warning', message: `Plugin "${p}" is installed but missing from config` });
          }
        }
      } else {
        issues.push({ path: 'config.json', severity: 'error', message: 'Node config.json does not exist on disk' });
      }

      // Check each plugin config
      for (const pluginId of installedPlugins) {
        const pluginConfigPath = join(node.paths.base, 'Plugins', pluginId, `${pluginId}.json`);
        if (!existsSync(pluginConfigPath)) {
          issues.push({ path: `Plugins/${pluginId}/${pluginId}.json`, severity: 'warning', message: `Plugin config file missing — node may use defaults or fail to load` });
        }
        const pluginDllPath = join(node.paths.base, 'Plugins', pluginId, `${pluginId}.dll`);
        if (!existsSync(pluginDllPath)) {
          issues.push({ path: `Plugins/${pluginId}/${pluginId}.dll`, severity: 'error', message: `Plugin DLL missing — plugin will not load` });
        }
      }
    } else {
      // neo-go
      const expectedConfig = this.generateNeoGoConfig(node);
      const onDiskPath = join(node.paths.config, 'protocol.yml');
      if (existsSync(onDiskPath)) {
        const YAML = (await import('js-yaml')).default;
        const onDisk = YAML.load(readFileSync(onDiskPath, 'utf-8')) as Record<string, unknown>;
        this.diffConfigs(expectedConfig, onDisk, '', issues);

        if (node.type === 'neo-go' && node.network !== 'private') {
          const expectedForks = this.getExpectedHardforks(node.network);
          const onDiskForks = (onDisk as any)?.ProtocolConfiguration?.Hardforks;
          if (expectedForks && onDiskForks) {
            for (const [name, height] of Object.entries(expectedForks)) {
              if (onDiskForks[name] !== height) {
                issues.push({ path: `ProtocolConfiguration.Hardforks.${name}`, severity: 'warning', message: `Hardfork "${name}" height differs: expected ${height}, found ${onDiskForks[name] ?? 'missing'}` });
              }
            }
          }
        }
      } else {
        issues.push({ path: 'protocol.yml', severity: 'error', message: 'Neo-go protocol.yml does not exist on disk' });
      }
    }

    // Check binary availability
    const { DownloadManager } = await import('./DownloadManager');
    const binaryPath = DownloadManager.getNodeBinaryPath(node.type, node.version);
    if (!binaryPath) {
      issues.push({ path: 'binary', severity: 'error', message: `${node.type} ${node.version} binary not found in downloads` });
    }

    // Check port conflicts across all config
    const portPaths: Record<number, string> = {};
    const allPorts = [
      { port: node.ports.rpc, label: 'RPC' },
      { port: node.ports.p2p, label: 'P2P' },
      { port: node.ports.websocket, label: 'WebSocket' },
      { port: node.ports.metrics, label: 'Metrics' },
    ];
    for (const { port, label } of allPorts) {
      if (port) {
        if (portPaths[port]) {
          issues.push({ path: `ports.${label}`, severity: 'error', message: `Port ${port} conflicts with ${portPaths[port]}` });
        }
        portPaths[port] = label;
      }
    }

    return {
      nodeId: node.id,
      nodeName: node.name,
      nodeType: node.type,
      version: node.version,
      network: node.network,
      issueCount: issues.length,
      errors: issues.filter(i => i.severity === 'error').length,
      warnings: issues.filter(i => i.severity === 'warning').length,
      info: issues.filter(i => i.severity === 'info').length,
      issues,
    };
  }

  /**
   * Diff two config objects and report meaningful differences.
   */
  private static diffConfigs(
    expected: Record<string, unknown>,
    actual: Record<string, unknown>,
    prefix: string,
    issues: ConfigAuditIssue[],
  ): void {
    // Important keys to check — skip noise like timestamps
    const skip = new Set(['createdAt', 'updatedAt', 'lastUpdate', 'lastStarted', 'lastStopped']);

    for (const key of Object.keys(expected)) {
      if (skip.has(key)) continue;
      const path = prefix ? `${prefix}.${key}` : key;
      const exp = expected[key];
      const act = actual?.[key];

      if (act === undefined) {
        // Only report missing critical keys
        if (['Network', 'Magic', 'StandbyCommittee', 'ValidatorsCount', 'SeedList', 'Hardforks', 'Port', 'Engine'].includes(key)) {
          issues.push({ path, severity: 'warning', message: `Expected key "${key}" is missing from on-disk config` });
        }
      } else if (typeof exp === 'object' && exp !== null && !Array.isArray(exp)) {
        if (typeof act === 'object' && act !== null && !Array.isArray(act)) {
          this.diffConfigs(exp as Record<string, unknown>, act as Record<string, unknown>, path, issues);
        }
      } else if (Array.isArray(exp) && Array.isArray(act)) {
        if (key === 'StandbyCommittee' && exp.length !== act.length) {
          issues.push({ path, severity: 'error', message: `StandbyCommittee size mismatch: expected ${exp.length}, found ${act.length}` });
        }
        if (key === 'SeedList' && exp.length !== act.length) {
          issues.push({ path, severity: 'info', message: `SeedList differs: expected ${exp.length} seeds, found ${act.length}` });
        }
      } else if (exp !== act) {
        // Report mismatches on critical fields
        if (['Network', 'Magic', 'ValidatorsCount', 'Engine'].includes(key)) {
          issues.push({ path, severity: 'error', message: `Value mismatch: expected ${JSON.stringify(exp)}, found ${JSON.stringify(act)}` });
        } else if (['Port', 'MaxPeers', 'MinPeers', 'MaxConnections', 'Relay'].includes(key)) {
          issues.push({ path, severity: 'info', message: `Value differs from default: expected ${JSON.stringify(exp)}, found ${JSON.stringify(act)} (may be intentional)` });
        }
      }
    }

    // Check for unexpected keys in actual that aren't in expected
    for (const key of Object.keys(actual || {})) {
      if (skip.has(key)) continue;
      if (expected[key] === undefined && typeof actual[key] !== 'object') {
        const path = prefix ? `${prefix}.${key}` : key;
        if (!['DownloadUrl', 'MaxKnownHashes', 'EnableCompression'].includes(key)) {
          issues.push({ path, severity: 'info', message: `Extra key "${key}" found in on-disk config (value: ${JSON.stringify(actual[key])})` });
        }
      }
    }
  }
}

export interface ConfigAuditIssue {
  path: string;
  severity: 'error' | 'warning' | 'info';
  message: string;
}

export interface ConfigAuditResult {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  version: string;
  network: string;
  issueCount: number;
  errors: number;
  warnings: number;
  info: number;
  issues: ConfigAuditIssue[];
}
