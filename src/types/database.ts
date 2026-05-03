import type { NodeChain, NodeType, NodeNetwork, SyncMode, NodeStatus } from './index';
import { chainOf } from './index';

export interface NodeRow {
  id: string;
  name: string;
  chain: NodeChain | null;
  type: NodeType;
  network: NodeNetwork;
  sync_mode: SyncMode;
  version: string;
  rpc_port: number;
  p2p_port: number;
  websocket_port: number | null;
  metrics_port: number | null;
  base_path: string;
  data_path: string;
  logs_path: string;
  config_path: string;
  wallet_path: string | null;
  settings: string;
  created_at: number;
  updated_at: number;
}

export interface ProcessRow {
  status: NodeStatus;
  pid: number | null;
  error_message: string | null;
}

export interface MetricsRow {
  block_height: number;
  header_height: number;
  connected_peers: number;
  unconnected_peers: number;
  sync_progress: number;
  memory_usage: number;
  cpu_usage: number;
  last_update: number;
}

export interface PluginRow {
  id: string;
  name: string;
  description: string;
  category: string;
  requires_config: number;
  dependencies: string | null;
  default_config: string | null;
}

export interface InstalledPluginRow {
  plugin_id: string;
  version: string;
  config: string | null;
  installed_at: number;
  enabled: number;
}

export interface NodeRoleProfileRow {
  id: string;
  name: string;
  description: string | null;
  kind: string;
  node_types: string;
  profile: string;
  created_by: string | null;
  created_at: number;
  updated_at: number;
}

export interface NodeRoleApplicationRow {
  id: string;
  node_id: string;
  role_id: string;
  role_name: string;
  application_plan: string;
  previous_state: string | null;
  applied_at: number;
  applied_by: string | null;
  status: string;
  error_message: string | null;
}

export interface NodeDataContextRow {
  id: string;
  node_id: string;
  label: string;
  storage_engine: string;
  sync_strategy: string;
  checkpoint_height: number | null;
  checkpoint_hash: string | null;
  snapshot_id: string | null;
  active: number;
  created_at: number;
  updated_at: number;
}

export interface FastSyncSnapshotRow {
  id: string;
  name: string;
  source_type: string;
  source: string;
  chain: string;
  network: string;
  node_type: string;
  storage_engine: string;
  height: number;
  block_hash: string | null;
  sha256: string;
  size_bytes: number | null;
  signature: string | null;
  trusted: number;
  created_at: number;
  last_verified_at: number | null;
}

export interface PrivateNetworkPlanRow {
  id: string;
  name: string;
  template: string;
  network_magic: number;
  plan: string;
  status: string;
  created_at: number;
  applied_at: number | null;
}

export function nodeRowToConfig(row: NodeRow) {
  return {
    id: row.id,
    name: row.name,
    chain: row.chain ?? chainOf(row.type),
    type: row.type,
    network: row.network,
    syncMode: row.sync_mode,
    version: row.version,
    ports: {
      rpc: row.rpc_port,
      p2p: row.p2p_port,
      websocket: row.websocket_port ?? undefined,
      metrics: row.metrics_port ?? undefined,
    },
    paths: {
      base: row.base_path,
      data: row.data_path,
      logs: row.logs_path,
      config: row.config_path,
      wallet: row.wallet_path ?? undefined,
    },
    settings: row.settings ? (() => { try { return JSON.parse(row.settings!); } catch { return {}; } })() : {},
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}
