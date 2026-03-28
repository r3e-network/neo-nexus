import { existsSync, rmSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type Database from 'better-sqlite3';
import { EventEmitter } from 'node:events';
import type { 
  NodeConfig, 
  NodeInstance, 
  CreateNodeRequest, 
  UpdateNodeRequest,
  ImportNodeRequest,
  NodeStatus,
  NodeMetrics,
  PluginId,
  LogEntry,
  ConfigurationSnapshot,
  ConfigurationSnapshotNode,
} from '../types/index';
import { NodeRow, ProcessRow, MetricsRow, nodeRowToConfig } from '../types/database';
import { paths, getNodePath, getNodeDataPath, getNodeLogsPath, getNodeConfigPath, getNodeWalletPath } from '../utils/paths';
import { isProcessAlive, getProcessCommand } from '../utils/lifecycle';
import { PortManager } from './PortManager';
import { ConfigManager } from './ConfigManager';
import { StorageManager } from './StorageManager';
import { DownloadManager } from './DownloadManager';
import { PluginManager } from './PluginManager';
import { SecureSignerManager } from './SecureSignerManager';
import { NeoCliNode } from '../nodes/NeoCliNode';
import { NeoGoNode } from '../nodes/NeoGoNode';
import { BaseNode } from '../nodes/BaseNode';

export interface NodeManagerEvents {
  nodeStatus: (event: { nodeId: string; status: NodeStatus; previousStatus: NodeStatus }) => void;
  nodeLog: (event: { nodeId: string; entry: LogEntry }) => void;
  nodeMetrics: (event: { nodeId: string; metrics: NodeMetrics }) => void;
}

export declare interface NodeManager {
  on<K extends keyof NodeManagerEvents>(event: K, listener: NodeManagerEvents[K]): this;
  emit<K extends keyof NodeManagerEvents>(event: K, ...args: Parameters<NodeManagerEvents[K]>): boolean;
}

export class NodeManager extends EventEmitter {
  private nodes: Map<string, BaseNode> = new Map();
  private portManager: PortManager;
  private pluginManager: PluginManager;
  private secureSignerManager: SecureSignerManager;

  constructor(private db: Database.Database) {
    super();
    this.pluginManager = new PluginManager(db);
    this.secureSignerManager = new SecureSignerManager(db);
    // Load existing nodes for port tracking
    const existingNodes = this.getAllNodes();
    const existingPorts = existingNodes.map(n => n.ports);
    this.portManager = new PortManager(existingPorts);
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
    const secureSignerBinding = request.settings?.keyProtection;
    if (secureSignerBinding?.mode === "secure-signer") {
      this.assertSecureSignerCompatibility(request.type, secureSignerBinding.signerProfileId);
    }

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
    await ConfigManager.writeNodeConfig(config);

    try {
      // Save to database in a transaction
      const saveTransaction = this.db.transaction(() => {
        this.saveNodeToDb(config);
      });
      saveTransaction();

      if (secureSignerBinding?.mode === "secure-signer") {
        await this.syncNodeSecureSigner(nodeId);
      }

      return this.getNode(nodeId)!;
    } catch (error) {
      // Clean up all related tables (CASCADE handles node_processes and node_metrics)
      this.db.prepare("DELETE FROM nodes WHERE id = ?").run(nodeId);
      rmSync(config.paths.base, { recursive: true, force: true });
      throw error;
    }
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
   * Reconcile DB process states against reality on startup.
   * Nodes recorded as 'running' or 'starting' with a PID are checked:
   *   - If the PID is alive and the command contains 'neo-cli' or 'neo-go', keep as running.
   *   - If the PID is alive but belongs to a different process, mark as stopped (stale PID).
   *   - If the PID is dead, mark as stopped.
   */
  reconcileProcessStates(): void {
    const nodes = this.getAllNodes();
    for (const node of nodes) {
      const { status, pid } = node.process;
      if ((status === 'running' || status === 'starting') && pid != null) {
        if (isProcessAlive(pid)) {
          const cmd = getProcessCommand(pid);
          if (cmd && (cmd.includes('neo-cli') || cmd.includes('neo-go'))) {
            console.log(`♻️ Node ${node.name} (PID ${pid}) still running`);
            this.updateNodeStatus(node.id, 'running', pid);
          } else {
            console.warn(`⚠️ Node ${node.name} stale PID`);
            this.updateNodeStatus(node.id, 'stopped');
          }
        } else {
          console.warn(`⚠️ Node ${node.name} PID dead`);
          this.updateNodeStatus(node.id, 'stopped');
        }
      }
    }
  }

  /**
   * Stop all running nodes
   */
  async stopAllNodes(): Promise<{ stoppedCount: number; alreadyStoppedCount: number }> {
    const nodes = this.getAllNodes();
    let stoppedCount = 0;
    let alreadyStoppedCount = 0;

    for (const node of nodes) {
      if (node.process.status === 'running') {
        await this.stopNode(node.id);
        stoppedCount++;
      } else {
        alreadyStoppedCount++;
      }
    }

    return { stoppedCount, alreadyStoppedCount };
  }

  /**
   * Clean old log files across all nodes
   */
  async cleanOldLogs(maxAgeDays = 30): Promise<{ cleanedFiles: number; nodesAffected: number; maxAgeDays: number }> {
    const nodes = this.getAllNodes();
    let cleanedFiles = 0;
    let nodesAffected = 0;

    for (const node of nodes) {
      const cleanedForNode = await StorageManager.cleanOldLogs(node.paths.logs, maxAgeDays);
      cleanedFiles += cleanedForNode;
      if (cleanedForNode > 0) {
        nodesAffected++;
      }
    }

    return {
      cleanedFiles,
      nodesAffected,
      maxAgeDays,
    };
  }

  /**
   * Export a configuration snapshot for all managed nodes
   */
  exportConfiguration(): {
    generatedAt: number;
    version: string;
    nodes: Array<Omit<NodeInstance, 'process' | 'metrics'>>;
  } {
    const nodes = this.getAllNodes().map(({ process, metrics, ...config }) => config);

    let version = "2.0.0";
    try {
      const pkg = JSON.parse(readFileSync(join(import.meta.dirname ?? '.', '..', 'package.json'), 'utf-8'));
      version = pkg.version || version;
    } catch { /* use default */ }

    return {
      generatedAt: Date.now(),
      version,
      nodes,
    };
  }

  /**
   * Reset all node data managed by NeoNexus
   */
  async resetAllNodeData(): Promise<{
    deletedNodeCount: number;
    removedDirectoryCount: number;
    stoppedCount: number;
    alreadyStoppedCount: number;
  }> {
    const nodes = this.getAllNodes();
    const { stoppedCount, alreadyStoppedCount } = await this.stopAllNodes();
    let deletedNodeCount = 0;
    let removedDirectoryCount = 0;

    for (const node of nodes) {
      await this.deleteNode(node.id);
      deletedNodeCount++;

      if (node.paths.base.startsWith(paths.nodes)) {
        rmSync(node.paths.base, { recursive: true, force: true });
        removedDirectoryCount++;
      }
    }

    return {
      deletedNodeCount,
      removedDirectoryCount,
      stoppedCount,
      alreadyStoppedCount,
    };
  }

  /**
   * Restore nodes from an exported configuration snapshot
   */
  async restoreConfiguration(
    snapshot: ConfigurationSnapshot,
    options: { replaceExisting?: boolean } = {},
  ): Promise<{ restoredCount: number; skippedCount: number; failedCount: number }> {
    if (options.replaceExisting) {
      await this.resetAllNodeData();
    }

    let restoredCount = 0;
    let skippedCount = 0;
    let failedCount = 0;

    for (const node of snapshot.nodes) {
      if (!this.isRestorableSnapshotNode(node)) {
        skippedCount++;
        continue;
      }

      try {
        const restoredNode = await this.createNode({
          name: node.name,
          type: node.type,
          network: node.network,
          syncMode: node.syncMode,
          version: node.version,
          customPorts: node.ports,
          settings: node.settings,
        });

        for (const plugin of node.plugins ?? []) {
          await this.installPlugin(restoredNode.id, plugin.id, plugin.config);
          if (plugin.enabled === false) {
            this.pluginManager.setPluginEnabled(restoredNode.id, plugin.id, false);
          }
        }

        restoredCount++;
      } catch (error) {
        failedCount++;
      }
    }

    return {
      restoredCount,
      skippedCount,
      failedCount,
    };
  }

  /**
   * Update node configuration
   */
  async updateNode(nodeId: string, request: UpdateNodeRequest): Promise<NodeInstance> {
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
    await ConfigManager.writeNodeConfig(updatedNode, updatedNode.plugins?.map(p => p.id));

    return updatedNode;
  }

  /**
   * Delete a node and optionally clean up filesystem data
   */
  async deleteNode(nodeId: string, removeFiles = true): Promise<void> {
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

    // Remove from database (CASCADE handles node_processes, node_metrics, node_plugins, logs)
    const stmt = this.db.prepare('DELETE FROM nodes WHERE id = ?');
    stmt.run(nodeId);

    // Remove from memory
    const runningNode = this.nodes.get(nodeId);
    if (runningNode) {
      runningNode.destroy();
      this.nodes.delete(nodeId);
    }

    // Clean up filesystem data for NeoNexus-managed nodes
    if (removeFiles && node.paths.base.startsWith(paths.nodes)) {
      rmSync(node.paths.base, { recursive: true, force: true });
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
      this.emit('nodeStatus', { nodeId, status, previousStatus: previous });
    });

    nodeInstance.on('log', (entry) => {
      this.saveLogEntry(nodeId, entry);
      this.emit('nodeLog', { nodeId, entry });
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
    await ConfigManager.writeNodeConfig(node, plugins);
  }

  /**
   * Get the plugin manager
   */
  getPluginManager(): PluginManager {
    return this.pluginManager;
  }

  getSecureSignerManager(): SecureSignerManager {
    return this.secureSignerManager;
  }

  async getNodeSecureSignerHealth(nodeId: string): Promise<{
    nodeId: string;
    profile: {
      id: string;
      name: string;
      mode: string;
      endpoint: string;
    };
    readiness: Awaited<ReturnType<SecureSignerManager["getReadiness"]>>;
  } | null> {
    const node = this.getNode(nodeId);
    const signerProfileId = node?.settings?.keyProtection?.signerProfileId;
    if (!node || node.settings?.keyProtection?.mode !== "secure-signer" || !signerProfileId) {
      return null;
    }

    const profile = this.secureSignerManager.getProfile(signerProfileId);
    if (!profile) {
      return {
        nodeId,
        profile: {
          id: signerProfileId,
          name: "Missing signer profile",
          mode: "unknown",
          endpoint: "unavailable",
        },
        readiness: {
          ok: false,
          status: "unreachable",
          source: "probe",
          message: `Secure signer profile ${signerProfileId} is not available`,
          checkedAt: Date.now(),
        },
      };
    }

    return {
      nodeId,
      profile: {
        id: profile.id,
        name: profile.name,
        mode: profile.mode,
        endpoint: profile.endpoint,
      },
      readiness: await this.secureSignerManager.getReadiness(profile.id),
    };
  }

  async syncNodeSecureSigner(nodeId: string): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    const keyProtection = node.settings?.keyProtection;
    if (!keyProtection || keyProtection.mode !== "secure-signer") {
      return;
    }

    const signerProfileId = keyProtection.signerProfileId;
    this.assertSecureSignerCompatibility(node.type, signerProfileId);

    const profile = this.secureSignerManager.getProfile(signerProfileId!);
    if (!profile || !profile.enabled) {
      throw new Error(`Secure signer profile ${signerProfileId} is not available`);
    }

    const config = this.secureSignerManager.buildSignClientConfig(profile);
    const installedPlugins = this.pluginManager.getInstalledPlugins(nodeId);
    const existingSignClient = installedPlugins.find((plugin) => plugin.id === "SignClient");

    if (existingSignClient) {
      this.pluginManager.updatePluginConfig(nodeId, "SignClient", config);
      this.pluginManager.setPluginEnabled(nodeId, "SignClient", true);
      return;
    }

    await this.pluginManager.installPlugin(nodeId, "SignClient", node.version, config);
  }

  /**
   * Update sync progress for the current metrics record.
   * Updates the node_metrics table which tracks current metrics per node.
   */
  updateSyncProgress(nodeId: string, syncProgress: number): void {
    const stmt = this.db.prepare('UPDATE node_metrics SET sync_progress = ? WHERE node_id = ?');
    stmt.run(syncProgress, nodeId);
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
      this.emit('nodeMetrics', { nodeId, metrics });
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
    const row = stmt.get(nodeId) as NodeRow | undefined;

    if (!row) return null;

    return nodeRowToConfig(row);
  }

  private getProcessFromDb(nodeId: string): { status: NodeStatus; pid?: number; errorMessage?: string } {
    const stmt = this.db.prepare('SELECT * FROM node_processes WHERE node_id = ?');
    const row = stmt.get(nodeId) as ProcessRow | undefined;

    return {
      status: row?.status ?? 'stopped',
      pid: row?.pid ?? undefined,
      errorMessage: row?.error_message ?? undefined,
    };
  }

  private getMetricsFromDb(nodeId: string): NodeMetrics | undefined {
    const stmt = this.db.prepare('SELECT * FROM node_metrics WHERE node_id = ?');
    const row = stmt.get(nodeId) as MetricsRow | undefined;

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
    const now = Date.now();
    if (status === 'running') {
      const stmt = this.db.prepare(`
        UPDATE node_processes
        SET status = ?, pid = ?, last_started = ?
        WHERE node_id = ?
      `);
      stmt.run(status, pid ?? null, now, nodeId);
    } else if (status === 'stopped') {
      const stmt = this.db.prepare(`
        UPDATE node_processes
        SET status = ?, pid = NULL, last_stopped = ?
        WHERE node_id = ?
      `);
      stmt.run(status, now, nodeId);
    } else {
      const stmt = this.db.prepare(`
        UPDATE node_processes
        SET status = ?, pid = ?
        WHERE node_id = ?
      `);
      stmt.run(status, pid ?? null, nodeId);
    }
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

  private isRestorableSnapshotNode(node: Partial<ConfigurationSnapshotNode>): node is ConfigurationSnapshotNode {
    return Boolean(node.name && node.type && node.network);
  }

  private assertSecureSignerCompatibility(
    nodeType: NodeConfig["type"],
    signerProfileId?: string,
  ): void {
    if (!signerProfileId) {
      throw new Error("Secure signer protection requires a signer profile");
    }

    if (nodeType !== "neo-cli") {
      throw new Error("Secure signer protection currently requires a neo-cli node with SignClient support");
    }

    const profile = this.secureSignerManager.getProfile(signerProfileId);
    if (!profile || !profile.enabled) {
      throw new Error(`Secure signer profile ${signerProfileId} is not available`);
    }
  }
}
