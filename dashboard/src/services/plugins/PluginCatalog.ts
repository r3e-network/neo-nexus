export type SupportedPluginId = 
  | 'tee-oracle' 
  | 'aa-bundler' 
  | 'tee-mempool'
  | 'ApplicationLogs'
  | 'DBFTPlugin'
  | 'LevelDBStore'
  | 'OracleService'
  | 'RestServer'
  | 'RocksDBStore'
  | 'RpcServer'
  | 'StateService'
  | 'TokensTracker'
  | 'SQLiteWallet'
  | 'StorageDumper'
  | 'SignClient';

export type PluginConfigSchemaField = {
  key: string;
  type: 'string' | 'number' | 'boolean' | 'stringArray' | 'multiselect';
  label: string;
  description: string;
  defaultValue?: string | number | boolean | string[];
  options?: string[]; // for multiselect
};

export type PluginDefinition = {
  id: SupportedPluginId;
  name: string;
  description: string;
  badge: string;
  requiresPrivateKey: boolean;
  category: 'Security' | 'Execution' | 'Core' | 'Storage' | 'API' | 'Tooling';
  defaultImage: string;
  schema?: PluginConfigSchemaField[];
};

const PLUGINS: PluginDefinition[] = [
  // Original NeoNexus Plugins
  {
    id: 'tee-oracle',
    name: 'TEE Privacy Oracle',
    description: 'Phala CVM instance securely bound to this node.',
    badge: 'Privacy',
    requiresPrivateKey: true,
    category: 'Security',
    defaultImage: 'ghcr.io/neonexus/tee-oracle:latest',
  },
  {
    id: 'aa-bundler',
    name: 'Account Abstraction Bundler',
    description: 'Native AA relay services for smart-account transaction bundling.',
    badge: 'AA',
    requiresPrivateKey: true,
    category: 'Execution',
    defaultImage: 'ghcr.io/neonexus/aa-bundler:latest',
  },
  {
    id: 'tee-mempool',
    name: 'TEE Protected Mempool',
    description: 'Protected pre-confirmation transaction routing for latency-sensitive workloads.',
    badge: 'TEE',
    requiresPrivateKey: true,
    category: 'Security',
    defaultImage: 'ghcr.io/neonexus/tee-mempool:latest',
  },
  
  // Official Neo-CLI Plugins
  {
    id: 'ApplicationLogs',
    name: 'Application Logs',
    description: 'Synchronizes the smart contract execution logs (ApplicationLogs) into the node storage.',
    badge: 'Core',
    requiresPrivateKey: false,
    category: 'Core',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'MaxLogSize', type: 'number', label: 'Max Log Size', description: 'Maximum size of the log file.', defaultValue: 2147483647 },
    ]
  },
  {
    id: 'DBFTPlugin',
    name: 'dBFT Consensus',
    description: 'Provides the dBFT consensus algorithm. Requires a consensus node private key.',
    badge: 'Consensus',
    requiresPrivateKey: true,
    category: 'Core',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'AutoStart', type: 'boolean', label: 'Auto Start', description: 'Automatically start consensus on boot.', defaultValue: true },
      { key: 'BlockTxNumber', type: 'number', label: 'Max Tx per Block', description: 'Maximum number of transactions per block.', defaultValue: 512 },
      { key: 'MaxBlockSize', type: 'number', label: 'Max Block Size', description: 'Maximum block size in bytes.', defaultValue: 262144 },
    ]
  },
  {
    id: 'LevelDBStore',
    name: 'LevelDB Store',
    description: 'Uses LevelDB for the underlying node storage engine.',
    badge: 'Storage',
    requiresPrivateKey: false,
    category: 'Storage',
    defaultImage: 'neo-cli-plugin',
  },
  {
    id: 'OracleService',
    name: 'Oracle Service',
    description: 'Enables the node to participate in the native Oracle network.',
    badge: 'Oracle',
    requiresPrivateKey: true, // Needs an oracle key
    category: 'Core',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'AutoStart', type: 'boolean', label: 'Auto Start', description: 'Automatically start the Oracle service on boot.', defaultValue: true },
    ]
  },
  {
    id: 'RestServer',
    name: 'REST Server',
    description: 'Provides a RESTful API interface for interacting with the Neo node.',
    badge: 'API',
    requiresPrivateKey: false,
    category: 'API',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'BindAddress', type: 'string', label: 'Bind Address', description: 'The IP address to bind to.', defaultValue: '0.0.0.0' },
      { key: 'Port', type: 'number', label: 'Port', description: 'The port to listen on.', defaultValue: 10334 },
      { key: 'KeepAliveTimeout', type: 'number', label: 'Keep Alive Timeout', description: 'HTTP keep-alive timeout.', defaultValue: 60 },
    ]
  },
  {
    id: 'RocksDBStore',
    name: 'RocksDB Store',
    description: 'Uses RocksDB for the underlying node storage engine (faster than LevelDB).',
    badge: 'Storage',
    requiresPrivateKey: false,
    category: 'Storage',
    defaultImage: 'neo-cli-plugin',
  },
  {
    id: 'RpcServer',
    name: 'RPC Server',
    description: 'Provides the standard JSON-RPC interface for the Neo node.',
    badge: 'API',
    requiresPrivateKey: false,
    category: 'API',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'DisabledMethods', type: 'multiselect', label: 'Disabled Methods', description: 'Specific JSON-RPC methods to disable.', defaultValue: [], options: ['getapplicationlog', 'getnep17balances', 'invokecontractverify', 'getversion', 'getblock', 'gettransaction', 'sendrawtransaction', 'getstorage'] },
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'BindAddress', type: 'string', label: 'Bind Address', description: 'The IP address to bind to.', defaultValue: '0.0.0.0' },
      { key: 'Port', type: 'number', label: 'Port', description: 'The port to listen on.', defaultValue: 10332 },
      { key: 'MaxConcurrentConnections', type: 'number', label: 'Max Connections', description: 'Maximum concurrent HTTP connections.', defaultValue: 40 },
      { key: 'KeepAliveTimeout', type: 'number', label: 'Keep Alive Timeout', description: 'HTTP keep-alive timeout.', defaultValue: 60 },
    ]
  },
  {
    id: 'SignClient',
    name: 'Sign Client',
    description: 'Allows node to securely communicate with a remote multi-sig wallet.',
    badge: 'Tooling',
    requiresPrivateKey: false,
    category: 'Tooling',
    defaultImage: 'neo-cli-plugin',
  },
  {
    id: 'SQLiteWallet',
    name: 'SQLite Wallet',
    description: 'Allows Neo-CLI to open and manage SQLite based NEP-6 wallets.',
    badge: 'Tooling',
    requiresPrivateKey: false,
    category: 'Tooling',
    defaultImage: 'neo-cli-plugin',
  },
  {
    id: 'StateService',
    name: 'State Service',
    description: 'Provides MPT state root tracking and validation.',
    badge: 'Core',
    requiresPrivateKey: false,
    category: 'Core',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
      { key: 'AutoStart', type: 'boolean', label: 'Auto Start', description: 'Automatically start the State service.', defaultValue: true },
      { key: 'FullState', type: 'boolean', label: 'Full State', description: 'Maintain full historic MPT state.', defaultValue: false },
    ]
  },
  {
    id: 'StorageDumper',
    name: 'Storage Dumper',
    description: 'Provides tools for dumping and migrating Neo node storage states.',
    badge: 'Tooling',
    requiresPrivateKey: false,
    category: 'Tooling',
    defaultImage: 'neo-cli-plugin',
  },
  {
    id: 'TokensTracker',
    name: 'Tokens Tracker',
    description: 'Tracks NEP-11 and NEP-17 token transfers and balances for fast querying.',
    badge: 'API',
    requiresPrivateKey: false,
    category: 'API',
    defaultImage: 'neo-cli-plugin',
    schema: [
      { key: 'Network', type: 'number', label: 'Network Magic', description: 'The network magic number.', defaultValue: 860833102 },
    ]
  },
];

export function listSupportedPlugins(): PluginDefinition[] {
  return PLUGINS;
}

export function isSupportedPlugin(pluginId: string): pluginId is SupportedPluginId {
  return PLUGINS.some((plugin) => plugin.id === pluginId);
}

export function getPluginDefinition(pluginId: string): PluginDefinition | null {
  return PLUGINS.find((plugin) => plugin.id === pluginId) ?? null;
}
