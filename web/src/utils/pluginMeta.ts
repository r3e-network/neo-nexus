export interface PluginConfigField {
  key: string;
  label: string;
  type: "number" | "text" | "boolean" | "select";
  help: string;
  defaultValue?: unknown;
  placeholder?: string;
  options?: Array<{ value: string; label: string }>;
  advanced?: boolean;
}

export interface PluginMeta {
  id: string;
  featureName: string;
  icon: string;
  summary: string;
  installNote: string;
  configFields: PluginConfigField[];
}

/**
 * Human-readable plugin metadata with config field schemas.
 * Each plugin maps from its technical ID to a user-friendly feature description.
 */
export const PLUGIN_META: Record<string, PluginMeta> = {
  RpcServer: {
    id: "RpcServer",
    featureName: "JSON-RPC API",
    icon: "api",
    summary: "Expose the standard Neo JSON-RPC interface so wallets, dApps, and tools can query your node.",
    installNote: "Installs the RpcServer plugin and opens an HTTP endpoint on the configured port.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "The protocol network magic number. Must match the network your node is connected to (e.g., 860833102 for N3 mainnet).",
        defaultValue: 860833102,
      },
      {
        key: "BindAddress",
        label: "Bind Address",
        type: "text",
        help: "IP address the RPC server listens on. Use 0.0.0.0 to listen on all interfaces, or 127.0.0.1 for localhost only.",
        defaultValue: "0.0.0.0",
        placeholder: "0.0.0.0",
      },
      {
        key: "Port",
        label: "RPC Port",
        type: "number",
        help: "TCP port for JSON-RPC requests. Default is 10332. Ensure this port is not already in use.",
        defaultValue: 10332,
      },
      {
        key: "MaxConcurrentConnections",
        label: "Max Connections",
        type: "number",
        help: "Maximum number of simultaneous RPC connections. Higher values use more memory but support more clients.",
        defaultValue: 40,
        advanced: true,
      },
      {
        key: "KeepAliveTimeout",
        label: "Keep-Alive Timeout (s)",
        type: "number",
        help: "Seconds to keep idle HTTP connections alive before closing them. Helps reduce connection overhead for frequent callers.",
        defaultValue: 60,
        advanced: true,
      },
    ],
  },

  RestServer: {
    id: "RestServer",
    featureName: "REST API",
    icon: "api",
    summary: "Provide a RESTful HTTP interface for simpler integration with web services and monitoring tools.",
    installNote: "Installs the RestServer plugin and opens an HTTP REST endpoint.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network magic number.",
        defaultValue: 860833102,
      },
      {
        key: "BindAddress",
        label: "Bind Address",
        type: "text",
        help: "IP address the REST server listens on. Use 0.0.0.0 for all interfaces.",
        defaultValue: "0.0.0.0",
      },
      {
        key: "Port",
        label: "REST Port",
        type: "number",
        help: "TCP port for REST API requests. Choose a port that doesn't conflict with the RPC server.",
        defaultValue: 10334,
      },
      {
        key: "KeepAliveTimeout",
        label: "Keep-Alive Timeout (s)",
        type: "number",
        help: "Seconds to keep idle connections open.",
        defaultValue: 60,
        advanced: true,
      },
    ],
  },

  ApplicationLogs: {
    id: "ApplicationLogs",
    featureName: "Smart Contract Logs",
    icon: "logs",
    summary: "Record smart contract execution logs so you can query transaction results and event notifications.",
    installNote: "Installs the ApplicationLogs plugin. Increases storage usage as logs accumulate over time.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network.",
        defaultValue: 860833102,
      },
      {
        key: "MaxLogSize",
        label: "Max Log Size (bytes)",
        type: "number",
        help: "Maximum size of a single log entry in bytes. The default (2 GB) is suitable for most use cases.",
        defaultValue: 2147483647,
        advanced: true,
      },
    ],
  },

  DBFTPlugin: {
    id: "DBFTPlugin",
    featureName: "Consensus (dBFT)",
    icon: "consensus",
    summary: "Participate in the Neo dBFT consensus mechanism. Required only if your node is a registered consensus node.",
    installNote: "Installs the dBFT consensus plugin. Only enable this if your node holds a consensus key and is part of the validator set.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network.",
        defaultValue: 860833102,
      },
      {
        key: "AutoStart",
        label: "Auto-Start Consensus",
        type: "boolean",
        help: "Automatically begin participating in consensus when the node starts. Disable if you want to start consensus manually.",
        defaultValue: true,
      },
      {
        key: "BlockTxNumber",
        label: "Max Transactions per Block",
        type: "number",
        help: "Maximum number of transactions to include in each proposed block.",
        defaultValue: 512,
        advanced: true,
      },
      {
        key: "MaxBlockSize",
        label: "Max Block Size (bytes)",
        type: "number",
        help: "Maximum size of a proposed block in bytes.",
        defaultValue: 262144,
        advanced: true,
      },
    ],
  },

  StateService: {
    id: "StateService",
    featureName: "State Root Tracking",
    icon: "state",
    summary: "Track and validate MPT (Merkle Patricia Trie) state roots for cross-chain verification and state proofs.",
    installNote: "Installs the StateService plugin. When FullState is enabled, storage usage increases significantly.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network.",
        defaultValue: 860833102,
      },
      {
        key: "AutoStart",
        label: "Auto-Start",
        type: "boolean",
        help: "Automatically start state tracking when the node boots.",
        defaultValue: true,
      },
      {
        key: "FullState",
        label: "Full State Mode",
        type: "boolean",
        help: "Store the complete state trie for every block. Enables historical state queries but uses significantly more disk space.",
        defaultValue: false,
      },
    ],
  },

  OracleService: {
    id: "OracleService",
    featureName: "Oracle Network",
    icon: "oracle",
    summary: "Enable your node to participate in the Neo Oracle protocol, fetching off-chain data for smart contracts.",
    installNote: "Installs the OracleService plugin. Requires network connectivity to external data sources.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network.",
        defaultValue: 860833102,
      },
      {
        key: "AutoStart",
        label: "Auto-Start",
        type: "boolean",
        help: "Automatically start the Oracle service when the node boots.",
        defaultValue: true,
      },
    ],
  },

  TokensTracker: {
    id: "TokensTracker",
    featureName: "Token Transfer Tracking",
    icon: "tokens",
    summary: "Index NEP-11 (NFT) and NEP-17 (fungible token) transfers and balances for fast token queries.",
    installNote: "Installs the TokensTracker plugin. Adds a database index for all token transfers, increasing storage usage.",
    configFields: [
      {
        key: "Network",
        label: "Network Magic",
        type: "number",
        help: "Must match your node's network.",
        defaultValue: 860833102,
      },
    ],
  },

  LevelDBStore: {
    id: "LevelDBStore",
    featureName: "LevelDB Storage",
    icon: "storage",
    summary: "Use Google's LevelDB as the node's storage engine. The default, battle-tested choice for most deployments.",
    installNote: "LevelDB is the default storage engine. Only one storage plugin can be active at a time.",
    configFields: [],
  },

  RocksDBStore: {
    id: "RocksDBStore",
    featureName: "RocksDB Storage",
    icon: "storage",
    summary: "Use Facebook's RocksDB for higher throughput and better compression. Recommended for high-traffic nodes.",
    installNote: "Replaces LevelDB as the storage engine. Only one storage plugin can be active at a time. Requires RocksDB native libraries.",
    configFields: [],
  },

  SQLiteWallet: {
    id: "SQLiteWallet",
    featureName: "SQLite Wallet Support",
    icon: "wallet",
    summary: "Enable opening and managing SQLite-based NEP-6 wallets directly from the node CLI.",
    installNote: "Installs the SQLiteWallet plugin. No configuration required.",
    configFields: [],
  },

  StorageDumper: {
    id: "StorageDumper",
    featureName: "Storage Dump & Migration",
    icon: "tools",
    summary: "Export and import node storage state snapshots for backup, migration, or fast-sync setup.",
    installNote: "Installs the StorageDumper plugin. No configuration required.",
    configFields: [],
  },

  SignClient: {
    id: "SignClient",
    featureName: "Remote Signing Client",
    icon: "security",
    summary: "Connect to a remote secure signing service instead of using a local wallet for transaction signing.",
    installNote: "Installs the SignClient plugin. Configure via the Secure Signer section in Settings.",
    configFields: [],
  },
};

/**
 * Get metadata for a plugin, falling back to a generic entry if not defined.
 */
export function getPluginMeta(pluginId: string, fallbackName?: string): PluginMeta {
  return (
    PLUGIN_META[pluginId] || {
      id: pluginId,
      featureName: fallbackName || pluginId,
      icon: "plugin",
      summary: "",
      installNote: `Installs the ${pluginId} plugin.`,
      configFields: [],
    }
  );
}

/**
 * Icon component name mapping for lucide-react.
 */
export const PLUGIN_ICON_MAP: Record<string, string> = {
  api: "Globe",
  logs: "ScrollText",
  consensus: "Vote",
  state: "GitBranch",
  oracle: "CloudCog",
  tokens: "Coins",
  storage: "Database",
  wallet: "Wallet",
  tools: "Wrench",
  security: "ShieldCheck",
  plugin: "Puzzle",
};
