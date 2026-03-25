import { writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import YAML from 'js-yaml';
import type { NodeConfig, NodeType, NodeNetwork, PluginId } from '../types/index';
import { getNetworkMagic, getSeedList } from '../utils/network';

export class ConfigManager {
  /**
   * Generate neo-cli config.json
   */
  static generateNeoCliConfig(node: NodeConfig, installedPlugins: PluginId[] = []): Record<string, unknown> {
    const networkMagic = getNetworkMagic(node.network);

    const config: Record<string, unknown> = {
      ApplicationConfiguration: {
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
          Port: node.ports.p2p,
          MinDesiredConnections: node.settings.minPeers ?? 10,
          MaxConnections: node.settings.maxPeers ?? 40,
          MaxConnectionsPerAddress: 3,
        },
        RpcServer: installedPlugins.includes('RpcServer')
          ? {
              Enabled: true,
              BindAddress: '0.0.0.0',
              Port: node.ports.rpc,
              MaxConcurrentConnections: node.settings.maxConnections ?? 40,
              KeepAliveTimeout: 60,
            }
          : undefined,
        UnlockWallet: {
          Path: '',
          Password: '',
          IsActive: false,
        },
        Contracts: {
          NeoName: 'NeoToken',
          GasName: 'GasToken',
        },
        Plugins: {
          Enabled: installedPlugins,
        },
      },
      ProtocolConfiguration: {
        Network: networkMagic,
        AddressVersion: 53,
        SecondsPerBlock: 15,
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
        MillisecondsPerBlock: 15000,
        MaxCommitteeChangeBlockHeight: 0,
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

    const config: Record<string, unknown> = {
      ProtocolConfiguration: {
        Magic: networkMagic,
        AddressVersion: 53,
        SecondsPerBlock: 15,
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
      ApplicationConfiguration: {
        DataDirectoryPath: 'Data',
        SkipBlockVerification: false,
        MaxPeers: node.settings.maxPeers ?? 100,
        AttemptConnPeers: node.settings.minPeers ?? 20,
        MinPeers: node.settings.minPeers ?? 5,
        RPC: {
          Enabled: true,
          Address: '0.0.0.0',
          Port: node.ports.rpc,
          MaxGasInvoke: 10,
          MaxIteratorResultItems: 100,
          MaxFindResultItems: 100,
          MaxConcurrentRequests: 40,
          KeepAliveTimeout: 90,
          TLSConfig: null,
        },
        P2P: {
          Address: '0.0.0.0',
          Port: node.ports.p2p,
          MinPeers: node.settings.minPeers ?? 5,
          MaxPeers: node.settings.maxPeers ?? 100,
          AttemptConnPeers: node.settings.minPeers ?? 20,
          PingInterval: 30,
          PingTimeout: 90,
        },
        Prometheus: node.ports.metrics
          ? {
              Enabled: true,
              Address: '0.0.0.0',
              Port: node.ports.metrics,
            }
          : undefined,
        UnlockWallet: null,
      },
    };

    // Remove undefined values
    return this.removeUndefined(config) as Record<string, unknown>;
  }

  /**
   * Generate plugin config.json for neo-cli
   */
  static generatePluginConfig(pluginId: PluginId, node: NodeConfig): object {
    const networkMagic = getNetworkMagic(node.network);

    const configs: Record<PluginId, object> = {
      ApplicationLogs: {
        Network: networkMagic,
        MaxLogSize: 2147483647,
      },
      DBFTPlugin: {
        Network: networkMagic,
        AutoStart: true,
        BlockTxNumber: 512,
        MaxBlockSize: 262144,
      },
      LevelDBStore: {},
      OracleService: {
        Network: networkMagic,
        AutoStart: true,
      },
      RestServer: {
        Network: networkMagic,
        BindAddress: '0.0.0.0',
        Port: node.ports.websocket ?? 10334,
        KeepAliveTimeout: 60,
      },
      RocksDBStore: {},
      RpcServer: {
        Network: networkMagic,
        BindAddress: '0.0.0.0',
        Port: node.ports.rpc,
        MaxConcurrentConnections: node.settings.maxConnections ?? 40,
        KeepAliveTimeout: 60,
        DisabledMethods: [],
      },
      SignClient: {},
      SQLiteWallet: {},
      StateService: {
        Network: networkMagic,
        AutoStart: true,
        FullState: false,
      },
      StorageDumper: {},
      TokensTracker: {
        Network: networkMagic,
      },
    };

    return configs[pluginId] ?? {};
  }

  /**
   * Write node configuration files
   */
  static writeNodeConfig(node: NodeConfig, installedPlugins: PluginId[] = []): void {
    // Ensure config directory exists
    mkdirSync(node.paths.config, { recursive: true });

    if (node.type === 'neo-cli') {
      const config = this.generateNeoCliConfig(node, installedPlugins);
      writeFileSync(
        join(node.paths.config, 'config.json'),
        JSON.stringify(config, null, 2)
      );
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
  static writePluginConfig(pluginId: PluginId, node: NodeConfig): void {
    if (node.type !== 'neo-cli') return;

    const config = this.generatePluginConfig(pluginId, node);
    const pluginDir = join(node.paths.base, 'Plugins', pluginId);
    
    mkdirSync(pluginDir, { recursive: true });
    
    writeFileSync(
      join(pluginDir, 'config.json'),
      JSON.stringify(config, null, 2)
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
}
