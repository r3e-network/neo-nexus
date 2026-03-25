import { mkdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import type Database from 'better-sqlite3';
import { randomUUID } from 'node:crypto';
import type { 
  NodeConfig, 
  NodeInstance, 
  CreateNodeRequest, 
  UpdateNodeRequest,
  ImportNodeRequest,
  NodeStatus,
  NodeMetrics,
  PluginId,
} from '../types/index';
import { paths, getNodePath, getNodeDataPath, getNodeLogsPath, getNodeConfigPath, getNodeWalletPath } from '../utils/paths';
import { PortManager } from './PortManager';
import { ConfigManager } from './ConfigManager';
import { StorageManager } from './StorageManager';
import { DownloadManager } from './DownloadManager';
import { PluginManager } from './PluginManager';
import { NeoCliNode } from '../nodes/NeoCliNode';
import { NeoGoNode } from '../nodes/NeoGoNode';
import { BaseNode } from '../nodes/BaseNode';

export class NodeManager {
  private nodes: Map<string, BaseNode> = new Map();
  private portManager: PortManager;
  private pluginManager: PluginManager;

  constructor(private db: Database.Database) {
    // Load existing nodes for port tracking
    const existingNodes = this.getAllNodes();
    const existingPorts = existingNodes.map(n => n.ports);
    this.portManager = new PortManager(existingPorts);
    this.pluginManager = new PluginManager(db);
  }

  /**
   * Import an existing node installation
   */
  async importExistingNode(request: ImportNodeRequest): Promise<NodeInstance> {
    const { NodeDetector } = await import('./NodeDetector');
    
    // Detect node configuration from existing path
    const detected = NodeDetector.detect(request.existingPath);
    if (!detected) {
      throw new Error(`Could not detect valid node installation at ${request.existingPath}. Make sure the path contains a valid neo-cli or neo-go installation.`);
    }

    // Validate the detected configuration
    const validation = NodeDetector.validateImport(detected);
    if (!validation.valid) {
      throw new Error(`Invalid node configuration: ${validation.errors.join(', ')}`);
    }

    // Use detected values or override with user-provided values
    const type = request.type || detected.type;
    const network = request.network || detected.network;
    const version = request.version || detected.version;
    const ports = { ...detected.ports, ...request.ports };

    // Check for port conflicts
    const nodeIndex = await this.portManager.findNextIndex();
    const allocatedPorts = await this.portManager.allocatePorts(nodeIndex);
    
    // Use detected ports if available, otherwise allocate new ones
    const finalPorts = {
      rpc: ports.rpc || allocatedPorts.rpc,
      p2p: ports.p2p || allocatedPorts.p2p,
      websocket: ports.websocket || allocatedPorts.websocket,
      metrics: ports.metrics || allocatedPorts.metrics,
    };

    // Create node configuration
    const nodeId = `node-${Date.now().toString(36)}-${Math.random().toString(36).substr(2, 5)}`;
    const now = Date.now();

    // Determine paths - use detected paths
    const existingPath = request.existingPath;
    const dataPath = detected.dataPath;
    const logsPath = join(existingPath, 'Logs');
    const configPath = detected.configPath;
    const walletPath = join(existingPath, 'wallets');

    const config: NodeConfig = {
      id: nodeId,
      name: request.name,
      type,
      network,
      syncMode: 'full', // Default for imported nodes
      version,
      ports: finalPorts,
      paths: {
        base: existingPath,
        data: dataPath,
        logs: logsPath,
        config: configPath,
        wallet: existsSync(walletPath) ? walletPath : undefined,
      },
      settings: {},
      createdAt: now,
      updatedAt: now,
    };

    // Save to database
    this.saveNodeToDb(config);

    // If a PID was provided and process is running, attach to it
    if (request.pid) {
      await this.attachToExistingProcess(nodeId, request.pid);
    } else if (detected.isRunning) {
      // Try to find and attach to running process
      await this.attachToRunningProcess(nodeId, type);
    }

    console.log(`Imported ${type} node from ${existingPath} with ID ${nodeId}`);
    
    return this.getNode(nodeId)!;
  }

  /**
   * Attach to an existing process by PID
   */
  private async attachToExistingProcess(nodeId: string, pid: number): Promise<void> {
    try {
      // Check if process exists
      process.kill(pid, 0); // Signal 0 checks if process exists
      
      // Update database to reflect running status
      this.updateNodeStatus(nodeId, 'running', pid);
      
      console.log(`Attached to existing process ${pid} for node ${nodeId}`);
    } catch {
      console.warn(`Process ${pid} not found for node ${nodeId}`);
    }
  }

  /**
   * Try to find and attach to a running node process
   */
  private async attachToRunningProcess(nodeId: string, type: string): Promise<void> {
    try {
      const { execSync } = require('node:child_process');
      
      // Try to find the process
      const processName = type === 'neo-cli' ? 'neo-cli' : 'neo-go';
      const result = execSync(`pgrep -f "${processName}" || true`, { encoding: 'utf8' });
      
      if (result.trim()) {
        const pid = parseInt(result.trim().split('\n')[0], 10);
        if (!isNaN(pid)) {
          await this.attachToExistingProcess(nodeId, pid);
        }
      }
    } catch {
      // Ignore errors - process not found is okay
    }
  }

  /**
   * Create a new node
   */
  async createNode(request: CreateNodeRequest): Promise<NodeInstance> {
    // Validate version or get latest
    let version = request.version;
    if (!version) {
      const release = await DownloadManager.getLatestRelease(request.type);
      if (!release) {
        throw new Error(`Could not determine latest version for ${request.type}`);
      }
      version = release.version;
    }

    // Ensure binary is downloaded
    if (!DownloadManager.hasNodeBinary(request.type, version)) {
      console.log(`Downloading ${request.type} ${version}...`);
      if (request.type === 'neo-cli') {
        await DownloadManager.downloadNeoCli(version);
      } else {
        await DownloadManager.downloadNeoGo(version);
      }
    }

    // Allocate ports
    const nodeIndex = await this.portManager.findNextIndex();
    const ports = request.customPorts 
      ? { ...await this.portManager.allocatePorts(nodeIndex), ...request.customPorts }
      : await this.portManager.allocatePorts(nodeIndex);

    // Create node configuration
    const nodeId = `node-${Date.now().toString(36)}-${Math.random().toString(36).substr(2, 5)}`;
    const now = Date.now();

    const config: NodeConfig = {
      id: nodeId,
      name: request.name,
      type: request.type,
      network: request.network,
      syncMode: request.syncMode || 'full',
      version,
      ports,
      paths: {
        base: getNodePath(nodeId),
        data: getNodeDataPath(nodeId),
        logs: getNodeLogsPath(nodeId),
        config: getNodeConfigPath(nodeId),
        wallet: getNodeWalletPath(nodeId),
      },
      settings: request.settings || {},
      createdAt: now,
      updatedAt: now,
    };

    // Create directories
    StorageManager.ensureNodeDirectories(config.paths);

    // Write initial config
    ConfigManager.writeNodeConfig(config);

    // Save to database
    this.saveNodeToDb(config);

    return this.getNode(nodeId)!;
  }

  /**
   * Get a node by ID
   */
  getNode(nodeId: string): NodeInstance | null {
    const config = this.getNodeConfigFromDb(nodeId);
    if (!config) return null;

    const process = this.getProcessFromDb(nodeId);
    const metrics = this.getMetricsFromDb(nodeId);
    const plugins = this.pluginManager.getInstalledPlugins(nodeId);

    return {
      ...config,
      process,
      metrics,
      plugins,
    };
  }

  /**
   * Get all nodes
   */
  getAllNodes(): NodeInstance[] {
    const stmt = this.db.prepare('SELECT id FROM nodes');
    const rows = stmt.all() as Array<{ id: string }>;
    
    return rows
      .map(row => this.getNode(row.id))
      .filter((node): node is NodeInstance => node !== null);
  }

  /**
   * Update node configuration
   */
  updateNode(nodeId: string, request: UpdateNodeRequest): NodeInstance {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    if (node.process.status === 'running') {
      throw new Error('Cannot update configuration while node is running');
    }

    const updates: string[] = [];
    const values: (string | number)[] = [];

    if (request.name) {
      updates.push('name = ?');
      values.push(request.name);
    }

    if (request.settings) {
      const currentSettings = node.settings || {};
      const newSettings = { ...currentSettings, ...request.settings };
      updates.push('settings = ?');
      values.push(JSON.stringify(newSettings));
    }

    updates.push('updated_at = ?');
    values.push(Date.now());
    values.push(nodeId);

    const stmt = this.db.prepare(`
      UPDATE nodes 
      SET ${updates.join(', ')}
      WHERE id = ?
    `);
    stmt.run(...values);

    // Update config files
    const updatedNode = this.getNode(nodeId)!;
    ConfigManager.writeNodeConfig(updatedNode, updatedNode.plugins?.map(p => p.id));

    return updatedNode;
  }

  /**
   * Delete a node
   */
  async deleteNode(nodeId: string): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    // Stop if running
    if (node.process.status === 'running') {
      await this.stopNode(nodeId);
    }

    // Release ports
    this.portManager.releasePorts(node.ports);

    // Remove from database
    const stmt = this.db.prepare('DELETE FROM nodes WHERE id = ?');
    stmt.run(nodeId);

    // Remove from memory
    const runningNode = this.nodes.get(nodeId);
    if (runningNode) {
      runningNode.destroy();
      this.nodes.delete(nodeId);
    }
  }

  /**
   * Start a node
   */
  async startNode(nodeId: string): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    // Check if already running
    const existingNode = this.nodes.get(nodeId);
    if (existingNode?.isRunning()) {
      throw new Error('Node is already running');
    }

    // Create appropriate node instance
    const nodeInstance = node.type === 'neo-cli' 
      ? new NeoCliNode(node)
      : new NeoGoNode(node);

    // Set up event handlers
    nodeInstance.on('status', (status, previous) => {
      this.updateNodeStatus(nodeId, status);
    });

    nodeInstance.on('log', (entry) => {
      this.saveLogEntry(nodeId, entry);
    });

    nodeInstance.on('error', (error) => {
      console.error(`Node ${nodeId} error:`, error);
    });

    // Start the node
    await nodeInstance.start();
    this.nodes.set(nodeId, nodeInstance);

    // Update database
    this.updateNodeStatus(nodeId, 'running', nodeInstance.getStatus().pid);
  }

  /**
   * Stop a node
   */
  async stopNode(nodeId: string, force = false): Promise<void> {
    const nodeInstance = this.nodes.get(nodeId);
    if (!nodeInstance) {
      throw new Error('Node is not running');
    }

    await nodeInstance.stop(force);
    this.nodes.delete(nodeId);

    // Update database
    this.updateNodeStatus(nodeId, 'stopped');
  }

  /**
   * Restart a node
   */
  async restartNode(nodeId: string): Promise<void> {
    await this.stopNode(nodeId);
    await this.startNode(nodeId);
  }

  /**
   * Get node logs
   */
  getNodeLogs(nodeId: string, count = 100): Array<{ timestamp: number; level: string; message: string }> {
    const stmt = this.db.prepare(`
      SELECT timestamp, level, message 
      FROM logs 
      WHERE node_id = ? 
      ORDER BY timestamp DESC 
      LIMIT ?
    `);
    return stmt.all(nodeId, count) as Array<{ timestamp: number; level: string; message: string }>;
  }

  /**
   * Get storage info for a node
   */
  async getStorageInfo(nodeId: string) {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    return StorageManager.getNodeStorageInfo(nodeId, node.paths);
  }

  /**
   * Install a plugin on a node
   */
  async installPlugin(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    if (node.type !== 'neo-cli') {
      throw new Error('Plugins are only supported for neo-cli nodes');
    }

    if (node.process.status === 'running') {
      throw new Error('Cannot install plugins while node is running');
    }

    await this.pluginManager.installPlugin(nodeId, pluginId, node.version, config);

    // Update node config
    const plugins = this.pluginManager.getInstalledPlugins(nodeId).map(p => p.id);
    ConfigManager.writeNodeConfig(node, plugins);
  }

  /**
   * Get the plugin manager
   */
  getPluginManager(): PluginManager {
    return this.pluginManager;
  }

  /**
   * Update node metrics
   */
  async updateMetrics(nodeId: string): Promise<void> {
    const nodeInstance = this.nodes.get(nodeId);
    if (!nodeInstance?.isRunning()) return;

    try {
      const [blockHeight, peersCount] = await Promise.all([
        nodeInstance.getBlockHeight(),
        nodeInstance.getPeersCount(),
      ]);

      const resources = await nodeInstance.getResourceUsage();

      const metrics: NodeMetrics = {
        blockHeight: blockHeight ?? 0,
        headerHeight: blockHeight ?? 0, // Simplified
        connectedPeers: peersCount ?? 0,
        unconnectedPeers: 0,
        syncProgress: 0, // Would need to calculate based on network height
        memoryUsage: resources?.memory ? resources.memory * 1024 * 1024 : 0, // Convert to bytes
        cpuUsage: resources?.cpu ?? 0,
        lastUpdate: Date.now(),
      };

      this.saveMetrics(nodeId, metrics);
    } catch (error) {
      console.error(`Failed to update metrics for ${nodeId}:`, error);
    }
  }

  // Database operations

  private saveNodeToDb(config: NodeConfig): void {
    const stmt = this.db.prepare(`
      INSERT INTO nodes (
        id, name, type, network, sync_mode, version,
        rpc_port, p2p_port, websocket_port, metrics_port,
        base_path, data_path, logs_path, config_path, wallet_path,
        settings, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    stmt.run(
      config.id,
      config.name,
      config.type,
      config.network,
      config.syncMode,
      config.version,
      config.ports.rpc,
      config.ports.p2p,
      config.ports.websocket ?? null,
      config.ports.metrics ?? null,
      config.paths.base,
      config.paths.data,
      config.paths.logs,
      config.paths.config,
      config.paths.wallet ?? null,
      JSON.stringify(config.settings),
      config.createdAt,
      config.updatedAt
    );

    // Initialize process record
    const processStmt = this.db.prepare(`
      INSERT INTO node_processes (node_id, status) VALUES (?, 'stopped')
    `);
    processStmt.run(config.id);

    // Initialize metrics record
    const metricsStmt = this.db.prepare(`
      INSERT INTO node_metrics (node_id) VALUES (?)
    `);
    metricsStmt.run(config.id);
  }

  private getNodeConfigFromDb(nodeId: string): NodeConfig | null {
    const stmt = this.db.prepare('SELECT * FROM nodes WHERE id = ?');
    const row = stmt.get(nodeId) as {
      id: string;
      name: string;
      type: 'neo-cli' | 'neo-go';
      network: 'mainnet' | 'testnet' | 'private';
      sync_mode: 'full' | 'light';
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
    } | undefined;

    if (!row) return null;

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

  private getProcessFromDb(nodeId: string): { status: NodeStatus; pid?: number; errorMessage?: string } {
    const stmt = this.db.prepare('SELECT * FROM node_processes WHERE node_id = ?');
    const row = stmt.get(nodeId) as {
      status: NodeStatus;
      pid: number | null;
      error_message: string | null;
    } | undefined;

    return {
      status: row?.status ?? 'stopped',
      pid: row?.pid ?? undefined,
      errorMessage: row?.error_message ?? undefined,
    };
  }

  private getMetricsFromDb(nodeId: string): NodeMetrics | undefined {
    const stmt = this.db.prepare('SELECT * FROM node_metrics WHERE node_id = ?');
    const row = stmt.get(nodeId) as {
      block_height: number;
      header_height: number;
      connected_peers: number;
      unconnected_peers: number;
      sync_progress: number;
      memory_usage: number;
      cpu_usage: number;
      last_update: number;
    } | undefined;

    if (!row) return undefined;

    return {
      blockHeight: row.block_height,
      headerHeight: row.header_height,
      connectedPeers: row.connected_peers,
      unconnectedPeers: row.unconnected_peers,
      syncProgress: row.sync_progress,
      memoryUsage: row.memory_usage,
      cpuUsage: row.cpu_usage,
      lastUpdate: row.last_update,
    };
  }

  private updateNodeStatus(nodeId: string, status: NodeStatus, pid?: number): void {
    const stmt = this.db.prepare(`
      UPDATE node_processes 
      SET status = ?, pid = ?, last_started = ?
      WHERE node_id = ?
    `);
    stmt.run(status, pid ?? null, status === 'running' ? Date.now() : null, nodeId);
  }

  private saveLogEntry(nodeId: string, entry: { timestamp: number; level: string; source: string; message: string }): void {
    const stmt = this.db.prepare(`
      INSERT INTO logs (node_id, timestamp, level, source, message)
      VALUES (?, ?, ?, ?, ?)
    `);
    stmt.run(nodeId, entry.timestamp, entry.level, entry.source, entry.message);
  }

  private saveMetrics(nodeId: string, metrics: NodeMetrics): void {
    const stmt = this.db.prepare(`
      UPDATE node_metrics 
      SET block_height = ?, header_height = ?, connected_peers = ?, 
          unconnected_peers = ?, sync_progress = ?, memory_usage = ?, 
          cpu_usage = ?, last_update = ?
      WHERE node_id = ?
    `);
    stmt.run(
      metrics.blockHeight,
      metrics.headerHeight,
      metrics.connectedPeers,
      metrics.unconnectedPeers,
      metrics.syncProgress,
      metrics.memoryUsage,
      metrics.cpuUsage,
      metrics.lastUpdate,
      nodeId
    );
  }
}
