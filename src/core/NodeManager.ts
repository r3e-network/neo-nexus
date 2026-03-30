import { existsSync, rmSync, readFileSync } from 'node:fs';
import { execSync } from 'node:child_process';
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
import { paths, getNodePath, getNodeDataPath, getNodeLogsPath, getNodeConfigPath, getNodeWalletPath } from '../utils/paths';
import { NodeRepository } from './NodeRepository';
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
import { Errors } from '../api/errors';

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
  private repo: NodeRepository;
  private portManager: PortManager;
  private pluginManager: PluginManager;
  private secureSignerManager: SecureSignerManager;

  constructor(private db: Database.Database) {
    super();
    this.repo = new NodeRepository(db);
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
      throw Errors.detectionFailed(request.existingPath);
    }

    // Validate the detected configuration
    const validation = NodeDetector.validateImport(detected);
    if (!validation.valid) {
      throw Errors.importInvalid(validation.errors);
    }

    // Use detected values or override with user-provided values
    const type = request.type || detected.type;
    const network = request.network || detected.network;
    const version = request.version || detected.version;
    const ports = { ...detected.ports, ...request.ports };

    // Allocate ports for any that weren't detected, then release the
    // speculative allocation so overridden ports don't leak in PortManager.
    const nodeIndex = await this.portManager.findNextIndex();
    const allocatedPorts = await this.portManager.allocatePorts(nodeIndex);
    const finalPorts = {
      rpc: ports.rpc || allocatedPorts.rpc,
      p2p: ports.p2p || allocatedPorts.p2p,
      websocket: ports.websocket || allocatedPorts.websocket,
      metrics: ports.metrics || allocatedPorts.metrics,
    };
    this.portManager.releasePorts(allocatedPorts);

    // Create node configuration
    const nodeId = `node-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 7)}`;
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
    this.repo.saveNode(config);

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
      this.repo.updateStatus(nodeId, 'running', pid);
      
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
    const nodeId = `node-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 7)}`;
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
      this.repo.transaction(() => {
        this.repo.saveNode(config);
      });

      if (secureSignerBinding?.mode === "secure-signer") {
        await this.syncNodeSecureSigner(nodeId);
      }

      return this.getNode(nodeId)!;
    } catch (error) {
      // Clean up all related tables (CASCADE handles node_processes and node_metrics)
      this.repo.deleteNode(nodeId);
      rmSync(config.paths.base, { recursive: true, force: true });
      throw error;
    }
  }

  /**
   * Get a node by ID
   */
  getNode(nodeId: string): NodeInstance | null {
    const config = this.repo.getNodeConfig(nodeId);
    if (!config) return null;

    const process = this.repo.getProcess(nodeId);
    const metrics = this.repo.getMetrics(nodeId);
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
    return this.repo.getAllNodeIds()
      .map(id => this.getNode(id))
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
            this.repo.updateStatus(node.id, 'running', pid);
          } else {
            console.warn(`⚠️ Node ${node.name} stale PID`);
            this.repo.updateStatus(node.id, 'stopped');
          }
        } else {
          console.warn(`⚠️ Node ${node.name} PID dead`);
          this.repo.updateStatus(node.id, 'stopped');
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
    const nodes = this.getAllNodes().map(({ process: _process, metrics: _metrics, ...config }) => config);

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
      } catch {
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
      throw Errors.nodeNotFound(nodeId);
    }

    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
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

    this.repo.updateNode(nodeId, updates, values);

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
      throw Errors.nodeNotFound(nodeId);
    }

    // Stop if running
    if (node.process.status === 'running') {
      await this.stopNode(nodeId);
    }

    // Release ports
    this.portManager.releasePorts(node.ports);

    // Remove from database (CASCADE handles node_processes, node_metrics, node_plugins, logs)
    this.repo.deleteNode(nodeId);

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
      throw Errors.nodeNotFound(nodeId);
    }

    // Check if already running
    const existingNode = this.nodes.get(nodeId);
    if (existingNode?.isRunning()) {
      throw Errors.nodeAlreadyRunning();
    }

    // Create appropriate node instance
    const nodeInstance = node.type === 'neo-cli' 
      ? new NeoCliNode(node)
      : new NeoGoNode(node);

    // Set up event handlers
    nodeInstance.on('status', (status, previous) => {
      this.repo.updateStatus(nodeId, status);
      this.emit('nodeStatus', { nodeId, status, previousStatus: previous });
    });

    nodeInstance.on('log', (entry) => {
      this.repo.saveLogEntry(nodeId, entry);
      this.emit('nodeLog', { nodeId, entry });
    });

    nodeInstance.on('error', (error) => {
      console.error(`Node ${nodeId} error:`, error);
    });

    // Start the node
    await nodeInstance.start();
    this.nodes.set(nodeId, nodeInstance);

    // Update database
    this.repo.updateStatus(nodeId, 'running', nodeInstance.getStatus().pid);
  }

  /**
   * Stop a node
   */
  async stopNode(nodeId: string, force = false): Promise<void> {
    const nodeInstance = this.nodes.get(nodeId);
    if (!nodeInstance) {
      throw Errors.nodeNotRunning();
    }

    await nodeInstance.stop(force);
    this.nodes.delete(nodeId);

    // Update database
    this.repo.updateStatus(nodeId, 'stopped');
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
      throw Errors.nodeNotFound(nodeId);
    }

    return StorageManager.getNodeStorageInfo(nodeId, node.paths);
  }

  /**
   * Install a plugin on a node
   */
  async installPlugin(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    if (node.type !== 'neo-cli') {
      throw Errors.pluginsCliOnly();
    }

    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
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
      throw Errors.nodeNotFound(nodeId);
    }

    const keyProtection = node.settings?.keyProtection;
    if (!keyProtection || keyProtection.mode !== "secure-signer") {
      return;
    }

    const signerProfileId = keyProtection.signerProfileId;
    this.assertSecureSignerCompatibility(node.type, signerProfileId);

    const profile = this.secureSignerManager.getProfile(signerProfileId!);
    if (!profile || !profile.enabled) {
      throw Errors.signerNotAvailable(signerProfileId!);
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
    this.repo.updateSyncProgress(nodeId, syncProgress);
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

      this.repo.saveMetrics(nodeId, metrics);
      this.emit('nodeMetrics', { nodeId, metrics });
    } catch (error) {
      console.error(`Failed to update metrics for ${nodeId}:`, error);
    }
  }

  getRepository(): NodeRepository {
    return this.repo;
  }

  private isRestorableSnapshotNode(node: Partial<ConfigurationSnapshotNode>): node is ConfigurationSnapshotNode {
    return Boolean(node.name && node.type && node.network);
  }

  private assertSecureSignerCompatibility(
    nodeType: NodeConfig["type"],
    signerProfileId?: string,
  ): void {
    if (!signerProfileId) {
      throw Errors.signerRequiresProfile();
    }

    if (nodeType !== "neo-cli") {
      throw Errors.signerNeoCliOnly();
    }

    const profile = this.secureSignerManager.getProfile(signerProfileId);
    if (!profile || !profile.enabled) {
      throw Errors.signerNotAvailable(signerProfileId);
    }
  }
}
