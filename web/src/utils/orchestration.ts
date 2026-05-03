import type {
  FastSyncSnapshot,
  NodeRoleApplicationPlan,
  NodeRoleProfile,
  PrivateNetworkTemplate,
  RoleSyncStrategy,
  StorageEngine,
} from "../hooks/useNodeOrchestration";
import type { Node } from "../hooks/useNodes";

export const STORAGE_ENGINE_OPTIONS: Array<{
  value: StorageEngine;
  label: string;
  description: string;
}> = [
  { value: "leveldb", label: "LevelDB", description: "General-purpose default with broad compatibility." },
  { value: "rocksdb", label: "RocksDB", description: "Higher write throughput for state and indexer workloads." },
];

export const SYNC_STRATEGY_OPTIONS: Array<{
  value: RoleSyncStrategy;
  label: string;
  description: string;
}> = [
  { value: "full", label: "Full", description: "Verify from genesis with no checkpoint shortcuts." },
  { value: "light", label: "Light", description: "Use the light synchronization mode where supported." },
  { value: "fast-sync", label: "Fast sync", description: "Prefer a registered snapshot or checkpoint-backed context." },
];

export const PRIVATE_NETWORK_TEMPLATE_OPTIONS: Array<{
  value: PrivateNetworkTemplate;
  label: string;
  nodeCount: number;
  description: string;
}> = [
  { value: "single", label: "Single node", nodeCount: 1, description: "Fast local devnet with one validator." },
  { value: "four", label: "4 nodes", nodeCount: 4, description: "Small consensus network for plugin and failover tests." },
  { value: "seven", label: "7 nodes", nodeCount: 7, description: "Production-shaped validator set for rehearsals." },
];

export const ROLE_PLUGIN_OPTIONS = [
  { id: "RpcServer", label: "RPC server" },
  { id: "StateService", label: "State service" },
  { id: "OracleService", label: "Oracle service" },
  { id: "DBFTPlugin", label: "Consensus" },
  { id: "ApplicationLogs", label: "Application logs" },
  { id: "TokensTracker", label: "Tokens tracker" },
] as const;

export function roleSupportsNode(role: NodeRoleProfile, nodeType: Node["type"]) {
  return role.nodeTypes.includes(nodeType);
}

export function roleStorageEngine(role: NodeRoleProfile, fallback: StorageEngine): StorageEngine {
  return role.profile.storageEngine ?? fallback;
}

export function currentStorageEngine(node: Node): StorageEngine {
  return node.settings?.storageEngine ?? "leveldb";
}

export function currentSyncStrategy(node: Node): RoleSyncStrategy {
  return node.settings?.syncStrategy ?? node.syncMode ?? "full";
}

export function formatStorageEngine(engine: StorageEngine) {
  return engine === "rocksdb" ? "RocksDB" : "LevelDB";
}

export function formatSyncStrategy(strategy: RoleSyncStrategy) {
  if (strategy === "fast-sync") return "Fast sync";
  return strategy === "light" ? "Light" : "Full";
}

export function summarizeRole(role: NodeRoleProfile) {
  const parts = [
    role.profile.storageEngine ? formatStorageEngine(role.profile.storageEngine) : null,
    role.profile.sync?.strategy ? formatSyncStrategy(role.profile.sync.strategy) : null,
    role.profile.plugins?.length ? `${role.profile.plugins.length} plugin${role.profile.plugins.length === 1 ? "" : "s"}` : null,
    role.profile.dataContext ? "isolated context" : null,
  ].filter(Boolean);

  return parts.length > 0 ? parts.join(" · ") : "Settings-only role";
}

export function planTone(plan?: NodeRoleApplicationPlan) {
  if (!plan) return "idle";
  if (plan.warnings.length > 0) return "warning";
  if (plan.changes.length === 0) return "success";
  return "change";
}

export function compatibleSnapshots(
  snapshots: FastSyncSnapshot[],
  node: Node,
  storageEngine: StorageEngine = currentStorageEngine(node),
) {
  const chain = node.chain ?? (node.type === "neox-go" ? "x" : "n3");
  return snapshots.filter((snapshot) =>
    snapshot.chain === chain
    && snapshot.network === node.network
    && snapshot.nodeType === node.type
    && snapshot.storageEngine === storageEngine);
}

export function defaultRoleLabelTemplate(role: NodeRoleProfile) {
  return role.profile.dataContext?.labelTemplate ?? `${role.id.replace(/^builtin-/, "")}-{network}-{storageEngine}`;
}

export function privateNetworkTemplateNodeCount(template: PrivateNetworkTemplate) {
  return PRIVATE_NETWORK_TEMPLATE_OPTIONS.find((option) => option.value === template)?.nodeCount ?? 1;
}

export function defaultPrivateNetworkName(template: PrivateNetworkTemplate) {
  const count = privateNetworkTemplateNodeCount(template);
  return count === 1 ? "Local single-node private network" : `Local ${count}-node private network`;
}
