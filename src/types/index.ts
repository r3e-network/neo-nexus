// Node Types
export type NodeChain = 'n3' | 'x';
export type N3NodeType = 'neo-cli' | 'neo-go';
export type XNodeType = 'neox-go';
export type NodeType = N3NodeType | XNodeType;
export type N3NodeNetwork = 'mainnet' | 'testnet' | 'private';
export type XNodeNetwork = 'neox-mainnet' | 'neox-testnet';
export type NodeNetwork = N3NodeNetwork | XNodeNetwork;
export type NodeStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'error' | 'syncing';
export type SyncMode = 'full' | 'light';
export type ImportedNodeOwnershipMode = 'observe-only' | 'managed-config' | 'managed-process';
export type StorageEngine = 'leveldb' | 'rocksdb';
export type RoleSyncStrategy = 'full' | 'light' | 'fast-sync';
export type NodeRoleKind = 'builtin' | 'custom';
export type RoleApplicationStatus = 'planned' | 'applied' | 'failed';
export type FastSyncSourceType = 'local' | 'url' | 'catalog';
export type PrivateNetworkTemplate = 'single' | 'four' | 'seven';
export type PrivateNetworkPlanStatus = 'draft' | 'applied' | 'failed';

// Node Configuration
export interface NodeConfig {
  id: string;
  name: string;
  chain: NodeChain;
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

export function chainOf(type: NodeType): NodeChain {
  return type === 'neox-go' ? 'x' : 'n3';
}

export function defaultNetworkForChain(chain: NodeChain): NodeNetwork {
  return chain === 'x' ? 'neox-mainnet' : 'mainnet';
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
  storageEngine?: StorageEngine;
  syncStrategy?: RoleSyncStrategy;
  activeDataContextId?: string;
  role?: {
    id: string;
    name: string;
    appliedAt: number;
  };
  customConfig?: Record<string, unknown>;
  keyProtection?: NodeKeyProtectionSettings;
  import?: ImportedNodeSettings;
  resourceLimits?: { maxMemoryMB?: number };
}

export interface ImportedNodeSettings {
  imported: boolean;
  ownershipMode: ImportedNodeOwnershipMode;
  existingPath?: string;
  importedAt?: number;
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
export type BlockHeightSyncStatus = 'synced' | 'syncing' | 'unknown' | 'private';

export interface BlockHeightStatus {
  status: BlockHeightSyncStatus;
  localHeight: number;
  networkHeight: number | null;
  remainingBlocks: number | null;
  progressPercent: number;
  stale: boolean;
  safeToUseAsLatest: boolean;
  checkedAt: number;
  message: string;
}

export interface NodeMetrics {
  blockHeight: number;
  headerHeight: number;
  connectedPeers: number;
  unconnectedPeers: number;
  syncProgress: number;
  memoryUsage: number;
  cpuUsage: number;
  lastUpdate: number;
  blockHeightStatus?: BlockHeightStatus;
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

export interface NodeRolePluginDesiredState {
  id: PluginId;
  enabled: boolean;
  config?: Record<string, unknown>;
}

export interface NodeRoleProfileBody {
  storageEngine?: StorageEngine;
  settings?: Partial<NodeSettings>;
  plugins?: NodeRolePluginDesiredState[];
  dataContext?: {
    mode: 'reuse-or-create' | 'always-create';
    labelTemplate: string;
  };
  sync?: {
    strategy: RoleSyncStrategy;
    allowCheckpoint?: boolean;
  };
  warnings?: string[];
  prerequisites?: string[];
}

export interface NodeRoleProfile {
  id: string;
  name: string;
  description?: string;
  kind: NodeRoleKind;
  nodeTypes: NodeType[];
  profile: NodeRoleProfileBody;
  createdBy?: string;
  createdAt: number;
  updatedAt: number;
}

export interface NodeRoleApplication {
  id: string;
  nodeId: string;
  roleId: string;
  roleName: string;
  applicationPlan: NodeRoleApplicationPlan;
  previousState?: Record<string, unknown>;
  appliedAt: number;
  appliedBy?: string;
  status: RoleApplicationStatus;
  errorMessage?: string;
}

export interface NodeRoleApplicationPlan {
  nodeId: string;
  roleId: string;
  roleName: string;
  requiresRestart: boolean;
  changes: Array<{
    type: 'settings' | 'plugin' | 'storage' | 'data-context' | 'fast-sync';
    summary: string;
  }>;
  warnings: string[];
}

export interface NodeDataContext {
  id: string;
  nodeId: string;
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
  active: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface FastSyncSnapshot {
  id: string;
  name: string;
  sourceType: FastSyncSourceType;
  source: string;
  chain: NodeChain;
  network: NodeNetwork;
  nodeType: NodeType;
  storageEngine: StorageEngine;
  height: number;
  blockHash?: string;
  sha256: string;
  sizeBytes?: number;
  signature?: string;
  trusted: boolean;
  createdAt: number;
  lastVerifiedAt?: number;
}

export interface PrivateNetworkPlan {
  id: string;
  name: string;
  template: PrivateNetworkTemplate;
  networkMagic: number;
  plan: {
    nodes: Array<{
      name: string;
      type: N3NodeType;
      roleIds: string[];
      storageEngine: StorageEngine;
      ports: Partial<PortConfig>;
      publicKey: string;
      address: string;
    }>;
    seedList: string[];
    validatorsCount: number;
    standbyCommittee: string[];
  };
  status: PrivateNetworkPlanStatus;
  createdAt: number;
  appliedAt?: number;
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
  ownershipMode?: ImportedNodeOwnershipMode;
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

export type UpdateSecureSignerRequest = Partial<CreateSecureSignerRequest>;

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

export interface SecureSignerCapabilityDeclaration {
  profileId: string;
  mode: SecureSignerMode;
  isolation: 'software-fallback' | 'sgx-enclave' | 'nitro-enclave' | 'external-hsm' | 'custom-remote-signer';
  privateKeyExportable: false;
  hardwareBacked: boolean;
  remoteAttestation: boolean;
  policyEnforcedBy: 'signer-service' | 'external-provider';
  experimental: boolean;
  notes: string[];
}

export interface SignClientPolicyRequest {
  allowedOperations?: string[];
  allowedContracts?: string[];
  allowedRecipients?: string[];
  requireHardwareProtection?: boolean;
}

export interface SignClientPolicyConfig {
  AllowedOperations?: string[];
  AllowedContracts?: string[];
  AllowedRecipients?: string[];
  RequireHardwareProtection: boolean;
  DenyByDefault: true;
}

export interface SignClientConfig extends Record<string, unknown> {
  Name: string;
  Endpoint: string;
  Policy?: SignClientPolicyConfig;
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
  policy?: SignClientPolicyRequest;
}

// WebSocket Events
export interface LogEntry {
  timestamp: number;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: string;
  message: string;
}

export type WebSocketMessage =
  | { type: 'log'; nodeId: string; data: LogEntry; timestamp: number }
  | { type: 'metrics'; nodeId: string; data: NodeMetrics; timestamp: number }
  | { type: 'status'; nodeId: string; data: { status: NodeStatus; previousStatus?: NodeStatus }; timestamp: number }
  | { type: 'system'; nodeId?: undefined; data: SystemMetrics; timestamp: number };

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
