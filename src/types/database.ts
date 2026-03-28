import type { NodeType, NodeNetwork, SyncMode, NodeStatus } from './index';

export interface NodeRow {
  id: string;
  name: string;
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

export function nodeRowToConfig(row: NodeRow) {
  return {
    id: row.id,
    name: row.name,
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
    settings: row.settings ? JSON.parse(row.settings) : {},
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}
