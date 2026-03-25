import { existsSync, mkdirSync, copyFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import type Database from 'better-sqlite3';
import type { PluginId, PluginDefinition, InstalledPlugin } from '../types/index';
import { DownloadManager } from './DownloadManager';
import { ConfigManager } from './ConfigManager';

// Plugin version mapping (simplified - in production, fetch from releases)
const PLUGIN_VERSIONS: Record<string, string> = {
  '3.9.2': 'v3.9.2',
  '3.9.1': 'v3.9.1',
  '3.9.0': 'v3.9.0',
  '3.8.0': 'v3.8.0',
  '0.106.0': 'v3.9.0', // neo-go uses different versioning
  '0.105.0': 'v3.8.0',
};

export class PluginManager {
  constructor(private db: Database.Database) {}

  /**
   * Get all available plugins
   */
  getAvailablePlugins(): PluginDefinition[] {
    const stmt = this.db.prepare('SELECT * FROM plugins ORDER BY category, name');
    const rows = stmt.all() as Array<{
      id: string;
      name: string;
      description: string;
      category: string;
      requires_config: number;
      dependencies: string | null;
      default_config: string | null;
    }>;

    return rows.map(row => ({
      id: row.id as PluginId,
      name: row.name,
      description: row.description,
      category: row.category as PluginDefinition['category'],
      requiresConfig: row.requires_config === 1,
      dependencies: row.dependencies ? JSON.parse(row.dependencies) : undefined,
      defaultConfig: row.default_config ? JSON.parse(row.default_config) : undefined,
    }));
  }

  /**
   * Get plugin definition by ID
   */
  getPlugin(id: PluginId): PluginDefinition | null {
    const stmt = this.db.prepare('SELECT * FROM plugins WHERE id = ?');
    const row = stmt.get(id) as {
      id: string;
      name: string;
      description: string;
      category: string;
      requires_config: number;
      dependencies: string | null;
      default_config: string | null;
    } | undefined;

    if (!row) return null;

    return {
      id: row.id as PluginId,
      name: row.name,
      description: row.description,
      category: row.category as PluginDefinition['category'],
      requiresConfig: row.requires_config === 1,
      dependencies: row.dependencies ? JSON.parse(row.dependencies) : undefined,
      defaultConfig: row.default_config ? JSON.parse(row.default_config) : undefined,
    };
  }

  /**
   * Get installed plugins for a node
   */
  getInstalledPlugins(nodeId: string): InstalledPlugin[] {
    const stmt = this.db.prepare(`
      SELECT np.*, p.name, p.category 
      FROM node_plugins np 
      JOIN plugins p ON np.plugin_id = p.id 
      WHERE np.node_id = ?
    `);
    const rows = stmt.all(nodeId) as Array<{
      plugin_id: string;
      version: string;
      config: string | null;
      installed_at: number;
      enabled: number;
    }>;

    return rows.map(row => ({
      id: row.plugin_id as PluginId,
      version: row.version,
      config: row.config ? JSON.parse(row.config) : {},
      installedAt: row.installed_at,
      enabled: row.enabled === 1,
    }));
  }

  /**
   * Install a plugin to a node
   */
  async installPlugin(
    nodeId: string,
    pluginId: PluginId,
    nodeVersion: string,
    customConfig?: Record<string, unknown>
  ): Promise<void> {
    const plugin = this.getPlugin(pluginId);
    if (!plugin) {
      throw new Error(`Unknown plugin: ${pluginId}`);
    }

    // Check if already installed
    const existing = this.getInstalledPlugins(nodeId).find(p => p.id === pluginId);
    if (existing) {
      throw new Error(`Plugin ${pluginId} is already installed on this node`);
    }

    // Get plugin version matching node version
    const pluginVersion = PLUGIN_VERSIONS[nodeVersion] || 'v3.9.2';

    // Download plugin
    const pluginSourceDir = await DownloadManager.downloadPlugin(pluginId, pluginVersion);

    // Install to node
    const nodePluginDir = join(this.getNodeBasePath(nodeId), 'Plugins', pluginId);
    mkdirSync(nodePluginDir, { recursive: true });

    // Copy plugin files
    this.copyPluginFiles(pluginSourceDir, nodePluginDir);

    // Create/update config
    const config = customConfig || plugin.defaultConfig || {};
    const stmt = this.db.prepare(`
      INSERT INTO node_plugins (node_id, plugin_id, version, config, installed_at, enabled)
      VALUES (?, ?, ?, ?, ?, 1)
    `);
    stmt.run(nodeId, pluginId, pluginVersion, JSON.stringify(config), Date.now());

    // Write plugin config file
    ConfigManager.writePluginConfig(pluginId, {
      id: nodeId,
      type: 'neo-cli',
      network: 'mainnet',
    } as unknown as Parameters<typeof ConfigManager.writePluginConfig>[1]);
  }

  /**
   * Uninstall a plugin from a node
   */
  async uninstallPlugin(nodeId: string, pluginId: PluginId): Promise<void> {
    // Remove from database
    const stmt = this.db.prepare('DELETE FROM node_plugins WHERE node_id = ? AND plugin_id = ?');
    stmt.run(nodeId, pluginId);

    // Remove plugin directory
    const pluginDir = join(this.getNodeBasePath(nodeId), 'Plugins', pluginId);
    if (existsSync(pluginDir)) {
      // In a real implementation, use fs.rm for recursive deletion
      // For now, we'll just mark it as disabled in the DB
    }
  }

  /**
   * Update plugin configuration
   */
  updatePluginConfig(
    nodeId: string,
    pluginId: PluginId,
    config: Record<string, unknown>
  ): void {
    const stmt = this.db.prepare(`
      UPDATE node_plugins 
      SET config = ?
      WHERE node_id = ? AND plugin_id = ?
    `);
    stmt.run(JSON.stringify(config), nodeId, pluginId);

    // Update config file
    ConfigManager.writePluginConfig(pluginId, {
      id: nodeId,
      type: 'neo-cli',
      network: 'mainnet',
    } as unknown as Parameters<typeof ConfigManager.writePluginConfig>[1]);
  }

  /**
   * Enable/disable a plugin
   */
  setPluginEnabled(nodeId: string, pluginId: PluginId, enabled: boolean): void {
    const stmt = this.db.prepare(`
      UPDATE node_plugins 
      SET enabled = ?
      WHERE node_id = ? AND plugin_id = ?
    `);
    stmt.run(enabled ? 1 : 0, nodeId, pluginId);
  }

  /**
   * Copy plugin files from source to destination
   */
  private copyPluginFiles(sourceDir: string, destDir: string): void {
    const files = readdirSync(sourceDir);
    
    for (const file of files) {
      const sourcePath = join(sourceDir, file);
      const destPath = join(destDir, file);
      
      const stat = statSync(sourcePath);
      if (stat.isDirectory()) {
        mkdirSync(destPath, { recursive: true });
        this.copyPluginFiles(sourcePath, destPath);
      } else {
        copyFileSync(sourcePath, destPath);
      }
    }
  }

  /**
   * Get node base path from database
   */
  private getNodeBasePath(nodeId: string): string {
    const stmt = this.db.prepare('SELECT base_path FROM nodes WHERE id = ?');
    const row = stmt.get(nodeId) as { base_path: string } | undefined;
    if (!row) {
      throw new Error(`Node ${nodeId} not found`);
    }
    return row.base_path;
  }

  /**
   * Get storage plugin for a node
   */
  getStoragePlugin(nodeId: string): 'LevelDBStore' | 'RocksDBStore' {
    const plugins = this.getInstalledPlugins(nodeId);
    if (plugins.some(p => p.id === 'RocksDBStore')) {
      return 'RocksDBStore';
    }
    return 'LevelDBStore';
  }

  /**
   * Get plugins that require restart when modified
   */
  getRestartRequiredPlugins(): PluginId[] {
    return ['RpcServer', 'RestServer', 'OracleService', 'DBFTPlugin'];
  }

  /**
   * Check if a plugin requires restart when modified
   */
  requiresRestart(pluginId: PluginId): boolean {
    return this.getRestartRequiredPlugins().includes(pluginId);
  }
}
