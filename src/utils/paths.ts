import { homedir } from 'node:os';
import { join } from 'node:path';

const configuredBaseDir = process.env.NEONEXUS_DATA_DIR || process.env.DATA_DIR;
const BASE_DIR = configuredBaseDir?.trim() || join(homedir(), '.neonexus');

export const paths = {
  base: BASE_DIR,
  nodes: join(BASE_DIR, 'nodes'),
  plugins: join(BASE_DIR, 'plugins'),
  downloads: join(BASE_DIR, 'downloads'),
  database: join(BASE_DIR, 'neonexus.db'),
  logs: join(BASE_DIR, 'logs'),
  config: join(BASE_DIR, 'config'),
} as const;

export function getNodePath(nodeId: string): string {
  return join(paths.nodes, nodeId);
}

export function getNodeDataPath(nodeId: string): string {
  return join(getNodePath(nodeId), 'data');
}

export function getNodeDataContextsRoot(nodeId: string): string {
  return join(getNodePath(nodeId), 'data-contexts');
}

export function getNodeDataContextPath(nodeId: string, contextId: string): string {
  return join(getNodeDataContextsRoot(nodeId), contextId);
}

export function getNodeLogsPath(nodeId: string): string {
  return join(getNodePath(nodeId), 'logs');
}

export function getNodeConfigPath(nodeId: string): string {
  return join(getNodePath(nodeId), 'config');
}

export function getNodeWalletPath(nodeId: string): string {
  return join(getNodePath(nodeId), 'wallets');
}

export function getPluginPath(pluginId: string): string {
  return join(paths.plugins, pluginId);
}

export function getDownloadPath(filename: string): string {
  return join(paths.downloads, filename);
}

export function getFastSyncRoot(): string {
  return join(paths.base, 'fast-sync');
}

export function getFastSyncStagingPath(snapshotId: string): string {
  return join(getFastSyncRoot(), 'staging', snapshotId);
}
