// Node Types
export type NodeType = 'neo-cli' | 'neo-go';
export type NodeNetwork = 'mainnet' | 'testnet' | 'private';
export type NodeStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'error' | 'syncing';
export type SyncMode = 'full' | 'light';

// Node Configuration
export interface NodeConfig {
  id: string;
  name: string;
  type: NodeType;
  network: NodeNetwork;
  syncMode: SyncMode;
  version: string;
  ports: PortConfig;
  paths: PathConfig;
  settings: NodeSettings;
  createdAt: number;
  updatedAt: number;
}

export interface PortConfig {
  rpc: number;
  p2p: number;
  websocket?: number;
  metrics?: number;
}

export interface PathConfig {
  base: string;
  data: string;
  logs: string;
  config: string;
  wallet?: string;
}

export interface NodeSettings {
  maxConnections?: number;
  minPeers?: number;
  maxPeers?: number;
  relay?: boolean;
  debugMode?: boolean;
  customConfig?: Record<string, unknown>;
  keyProtection?: NodeKeyProtectionSettings;
}

// Process Status
export interface ProcessStatus {
  pid?: number;
  status: NodeStatus;
  uptime?: number;
  exitCode?: number;
  errorMessage?: string;
  lastStarted?: number;
  lastStopped?: number;
}

// Node Instance
export interface NodeInstance extends NodeConfig {
  process: ProcessStatus;
  metrics?: NodeMetrics;
  plugins?: InstalledPlugin[];
}

// Metrics
export interface NodeMetrics {
  blockHeight: number;
  headerHeight: number;
  connectedPeers: number;
  unconnectedPeers: number;
  syncProgress: number;
  memoryUsage: number;
  cpuUsage: number;
  lastUpdate: number;
}

export interface SystemMetrics {
  cpu: {
    usage: number;
    cores: number;
  };
  memory: {
    total: number;
    used: number;
    free: number;
    percentage: number;
  };
  disk: {
    total: number;
    used: number;
    free: number;
    percentage: number;
  };
  network: {
    rx: number;
    tx: number;
  };
}

// Plugins
export type PluginId = 
  | 'ApplicationLogs'
  | 'DBFTPlugin'
  | 'LevelDBStore'
  | 'OracleService'
  | 'RestServer'
  | 'RocksDBStore'
  | 'RpcServer'
  | 'SignClient'
  | 'SQLiteWallet'
  | 'StateService'
  | 'StorageDumper'
  | 'TokensTracker';

export interface PluginDefinition {
  id: PluginId;
  name: string;
  description: string;
  category: 'Core' | 'Storage' | 'API' | 'Tooling';
  requiresConfig: boolean;
  dependencies?: PluginId[];
  defaultConfig?: Record<string, unknown>;
}

export interface InstalledPlugin {
  id: PluginId;
  version: string;
  config: Record<string, unknown>;
  installedAt: number;
  enabled: boolean;
}

// API Types
export interface CreateNodeRequest {
  name: string;
  type: NodeType;
  network: NodeNetwork;
  syncMode?: SyncMode;
  version?: string;
  customPorts?: Partial<PortConfig>;
  settings?: Partial<NodeSettings>;
}

export interface ImportNodeRequest {
  name: string;
  type: NodeType;
  existingPath: string;
  pid?: number;
  network?: NodeNetwork;
  version?: string;
  ports?: Partial<PortConfig>;
}

export interface UpdateNodeRequest {
  name?: string;
  settings?: Partial<NodeSettings>;
}

export interface InstallPluginRequest {
  pluginId: PluginId;
  version?: string;
  config?: Record<string, unknown>;
}

export interface ConfigurationSnapshotNode {
  name: string;
  type: NodeType;
  network: NodeNetwork;
  syncMode?: SyncMode;
  version?: string;
  ports?: Partial<PortConfig>;
  settings?: Partial<NodeSettings>;
  plugins?: Array<{
    id: PluginId;
    config?: Record<string, unknown>;
    enabled?: boolean;
  }>;
}

export interface ConfigurationSnapshot {
  generatedAt?: number;
  version: string;
  nodes: ConfigurationSnapshotNode[];
}

export interface RemoteServerProfile {
  id: string;
  name: string;
  baseUrl: string;
  description?: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface CreateRemoteServerRequest {
  name: string;
  baseUrl: string;
  description?: string;
  enabled?: boolean;
}

export interface UpdateRemoteServerRequest {
  name?: string;
  baseUrl?: string;
  description?: string;
  enabled?: boolean;
}

export interface RemoteServerStatusSummary {
  totalNodes: number;
  runningNodes: number;
  syncingNodes: number;
  errorNodes: number;
  totalBlocks: number;
  totalPeers: number;
  timestamp: number;
}

export interface RemoteServerNodeSummary {
  id: string;
  name: string;
  type: NodeType;
  network: NodeNetwork;
  status: NodeStatus | string;
  version: string;
  metrics: {
    blockHeight: number;
    headerHeight?: number;
    connectedPeers: number;
    syncProgress?: number;
  } | null;
  uptime?: number;
  lastUpdate?: number;
}

export interface RemoteServerSystemMetrics {
  cpu: {
    usage: number;
    cores: number;
  };
  memory: {
    percentage: number;
    used: number;
    total: number;
  };
  disk: {
    percentage: number;
    used: number;
    total: number;
  };
  timestamp?: number;
}

export interface RemoteServerSummary {
  profile: RemoteServerProfile;
  reachable: boolean;
  status?: RemoteServerStatusSummary;
  nodes?: RemoteServerNodeSummary[];
  systemMetrics?: RemoteServerSystemMetrics;
  error?: string;
}

export type SecureSignerMode = "software" | "sgx" | "nitro" | "custom";
export type SecureSignerUnlockMode = "manual" | "interactive-passphrase" | "recipient-attestation";
export type SecureSignerTestStatus = "reachable" | "unreachable" | "warning";
export type NodeKeyProtectionMode = "standard" | "secure-signer";

export interface SecureSignerProfile {
  id: string;
  name: string;
  mode: SecureSignerMode;
  endpoint: string;
  publicKey?: string;
  accountAddress?: string;
  walletPath?: string;
  unlockMode: SecureSignerUnlockMode;
  notes?: string;
  enabled: boolean;
  workspacePath?: string;
  startupPort?: number;
  awsRegion?: string;
  kmsKeyId?: string;
  kmsCiphertextBlobPath?: string;
  lastTestStatus?: SecureSignerTestStatus;
  lastTestMessage?: string;
  lastTestedAt?: number;
  createdAt: number;
  updatedAt: number;
}

export interface CreateSecureSignerRequest {
  name: string;
  mode: SecureSignerMode;
  endpoint: string;
  publicKey?: string;
  accountAddress?: string;
  walletPath?: string;
  unlockMode?: SecureSignerUnlockMode;
  notes?: string;
  enabled?: boolean;
  workspacePath?: string;
  startupPort?: number;
  awsRegion?: string;
  kmsKeyId?: string;
  kmsCiphertextBlobPath?: string;
}

export interface UpdateSecureSignerRequest {
  name?: string;
  mode?: SecureSignerMode;
  endpoint?: string;
  publicKey?: string;
  accountAddress?: string;
  walletPath?: string;
  unlockMode?: SecureSignerUnlockMode;
  notes?: string;
  enabled?: boolean;
  workspacePath?: string;
  startupPort?: number;
  awsRegion?: string;
  kmsKeyId?: string;
  kmsCiphertextBlobPath?: string;
}

export interface SecureSignerTestResult {
  ok: boolean;
  status: SecureSignerTestStatus;
  message: string;
  latencyMs?: number;
  checkedAt: number;
}

export interface SecureSignerConnectionInfo {
  scheme: "http" | "https" | "vsock";
  host: string;
  servicePort: number;
  startupPort: number;
  cid?: number;
  localToolingCompatible: boolean;
}

export interface SecureSignerReadinessResult extends SecureSignerTestResult {
  source: "probe" | "secure-sign-tools" | "vsock-format";
  accountStatus?: string;
}

export interface SecureSignerOrchestrationPlan {
  connection: SecureSignerConnectionInfo;
  commands: {
    deploy: string[];
    unlock: string[];
    status: string[];
    attestation: string[];
    startRecipient: string[];
  };
  warnings: string[];
}

export interface SecureSignerAttestationResult {
  attestationBase64: string;
  checkedAt: number;
}

export interface SecureSignerCommandResult {
  ok: boolean;
  message: string;
  checkedAt: number;
}

export interface NodeKeyProtectionSettings {
  mode: NodeKeyProtectionMode;
  signerProfileId?: string;
  signerName?: string;
  signerMode?: SecureSignerMode;
  signerEndpoint?: string;
  accountPublicKey?: string;
  accountAddress?: string;
  walletPath?: string;
  unlockMode?: SecureSignerUnlockMode;
}

// WebSocket Events
export interface LogEntry {
  timestamp: number;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: string;
  message: string;
}

export interface WebSocketMessage {
  type: 'log' | 'metrics' | 'status' | 'system';
  nodeId?: string;
  data: unknown;
  timestamp: number;
}

// Storage
export interface StorageInfo {
  chain: {
    size: number;
    path: string;
  };
  logs: {
    size: number;
    files: number;
  };
  wallets: {
    count: number;
    path: string;
  };
}

// Downloads
export interface ReleaseInfo {
  version: string;
  url: string;
  publishedAt: string;
}

// Node Import
export interface DetectedNodeConfig {
  type: NodeType;
  network: NodeNetwork;
  version: string;
  ports: Partial<PortConfig>;
  dataPath: string;
  configPath: string;
  isRunning: boolean;
}
