import { existsSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
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
  PortConfig,
  LogEntry,
  ConfigurationSnapshot,
  ImportedNodeOwnershipMode,
  NodeSettings,
  NodeKeyProtectionSettings,
  StorageEngine,
  RoleSyncStrategy,
  SyncMode,
} from '../types/index';
import { chainOf, isSidecarNodeType } from '../types/index';
import { getNodePath, getNodeDataPath, getNodeLogsPath, getNodeConfigPath, getNodeWalletPath } from '../utils/paths';
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
import { NeoXNode } from '../nodes/NeoXNode';
import { NeofuraNode } from '../nodes/NeofuraNode';
import { BaseNode } from '../nodes/BaseNode';
import { Errors } from '../api/errors';
import { assertNodeNetwork, assertNodeType, assertReleaseVersion } from '../utils/nodeValidation';
import {
  getAttachedProcessState,
  isManagedNodeDirectory,
  parseProcessIds,
  scoreAttachCandidate,
} from './nodeProcessAttachment';
import { collectRuntimeMetrics } from './nodeRuntimeMetrics';
import {
  NodeSnapshotService,
  type ExportedConfigurationSnapshot,
  type RestoreConfigurationResult,
} from './NodeSnapshotService';
import { normalizeImportedOwnershipMode, stripReservedImportSettings } from './nodeSettings';

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

    // Imported native nodes should keep their detected/operator-provided
    // ports. Do not probe NeoNexus' default managed range unless the import
    // lacks required RPC/P2P ports, otherwise unrelated local services can
    // block adoption of a perfectly valid existing node.
    let finalPorts: PortConfig;
    if (typeof ports.rpc === 'number' && typeof ports.p2p === 'number') {
      finalPorts = {
        rpc: ports.rpc,
        p2p: ports.p2p,
        ...(typeof ports.websocket === 'number' ? { websocket: ports.websocket } : {}),
        ...(typeof ports.metrics === 'number' ? { metrics: ports.metrics } : {}),
      };
    } else {
      const nodeIndex = await this.portManager.findNextIndex(100, chainOf(type));
      const allocatedPorts = await this.portManager.allocatePorts(nodeIndex, chainOf(type));
      finalPorts = {
        rpc: ports.rpc || allocatedPorts.rpc,
        p2p: ports.p2p || allocatedPorts.p2p,
        websocket: ports.websocket || allocatedPorts.websocket,
        metrics: ports.metrics || allocatedPorts.metrics,
      };
      this.portManager.releasePorts(allocatedPorts);
    }
    await this.portManager.reservePorts(finalPorts);

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
      chain: chainOf(type),
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
      settings: {
        import: {
          imported: true,
          ownershipMode: normalizeImportedOwnershipMode(request.ownershipMode),
          existingPath,
          importedAt: now,
        },
      },
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
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    if (getAttachedProcessState(node, pid) === 'active') {
      // Update database to reflect running status only after validating that
      // the PID is alive and matches the imported native node.
      this.repo.updateStatus(nodeId, 'running', pid);
      console.log(`Attached to existing process ${pid} for node ${nodeId}`);
      return;
    }

    console.warn(`Process ${pid} is not a trusted running process for node ${nodeId}`);
  }

  /**
   * Try to find and attach to a running node process
   */
  private async attachToRunningProcess(nodeId: string, type: string): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    try {
      const processName = type === 'neo-cli' ? 'neo-cli' : 'neo-go';
      const result = execFileSync('pgrep', ['-f', processName], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
      const candidates = parseProcessIds(result)
        .map((pid) => ({ pid, score: scoreAttachCandidate(node, pid) }))
        .filter((candidate) => candidate.score > 0)
        .sort((a, b) => b.score - a.score);

      for (const candidate of candidates) {
        if (getAttachedProcessState(node, candidate.pid) === 'active') {
          await this.attachToExistingProcess(nodeId, candidate.pid);
          return;
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
    const type = assertNodeType(request.type);
    const network = assertNodeNetwork(request.network);
    this.assertSyncMode(request.syncMode);
    this.assertSettingsRuntimeValues(request.settings);
    const secureSignerBinding = request.settings?.keyProtection;
    if (secureSignerBinding?.mode === "secure-signer") {
      this.assertSecureSignerCompatibility(type, secureSignerBinding.signerProfileId);
    }

    // Sidecars (neofura) are observe-only — NeoNexus doesn't manage
    // their binary lifecycle. Skip download / version-resolution
    // entirely. A symbolic version string is still recorded in the
    // config for display purposes.
    const isSidecar = isSidecarNodeType(type);

    // Validate version or get latest
    let version = request.version ? assertReleaseVersion(request.version) : undefined;
    if (!version) {
      if (isSidecar) {
        // No GitHub releases for neo3fura. Use a placeholder so the
        // shape of NodeConfig stays consistent across types.
        version = "external";
      } else {
        const release = await DownloadManager.getLatestRelease(type);
        if (!release) {
          throw new Error(`Could not determine latest version for ${type}`);
        }
        version = assertReleaseVersion(release.version);
      }
    }

    // Ensure binary is downloaded — skip for sidecars (their process
    // is owned by an external supervisor).
    const chain = chainOf(type);
    if (!isSidecar && !DownloadManager.hasNodeBinary(type, version)) {
      console.log(`Downloading ${type} ${version}...`);
      if (type === 'neo-cli') {
        await DownloadManager.downloadNeoCli(version);
      } else if (type === 'neox-go') {
        await DownloadManager.downloadNeoX(version);
      } else {
        await DownloadManager.downloadNeoGo(version);
      }
    }

    // Allocate ports — Neo X uses a separate range so chains can coexist.
    const nodeIndex = await this.portManager.findNextIndex(100, chain);
    const allocatedPorts = await this.portManager.allocatePorts(nodeIndex, chain);
    const ports = request.customPorts
      ? { ...allocatedPorts, ...request.customPorts }
      : allocatedPorts;
    if (request.customPorts) {
      this.portManager.releasePorts(allocatedPorts);
      await this.portManager.reservePorts(ports);
    }

    // Create node configuration
    const nodeId = `node-${Date.now().toString(36)}-${Math.random().toString(36).substring(2, 7)}`;
    const now = Date.now();
    const sanitizedSettings = stripReservedImportSettings(request.settings) || {};

    const config: NodeConfig = {
      id: nodeId,
      name: request.name,
      chain: chainOf(type),
      type,
      network,
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
      settings: {
        storageEngine: request.settings?.storageEngine ?? "leveldb",
        syncStrategy: request.settings?.syncStrategy ?? request.syncMode ?? "full",
        ...sanitizedSettings,
      },
      createdAt: now,
      updatedAt: now,
    };

    try {
      // Create directories
      StorageManager.ensureNodeDirectories(config.paths, {
        activeDataContextId: config.settings.activeDataContextId,
      });

      // Save to database in a transaction
      this.repo.transaction(() => {
        this.repo.saveNode(config);
      });

      if (type === 'neo-cli' && config.settings.storageEngine === 'rocksdb') {
        await this.pluginManager.upsertPlugin(nodeId, 'RocksDBStore', version, {});
        this.disableConflictingStoragePlugin(nodeId, 'rocksdb');
      }

      if (secureSignerBinding?.mode === "secure-signer") {
        await this.syncNodeSecureSigner(nodeId);
      }

      const createdNode = this.getNode(nodeId)!;
      await ConfigManager.writeNodeConfig(createdNode, this.getEnabledPluginIds(nodeId));
      return createdNode;
    } catch (error) {
      // Clean up all related tables (CASCADE handles node_processes and node_metrics)
      this.repo.deleteNode(nodeId);
      this.portManager.releasePorts(ports);
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
   * Re-instantiate sidecar (observe-only) nodes whose persisted status was
   * 'running' before the process exited. Managed nodes are handled by
   * reconcileProcessStates() via PID re-attach; sidecars have no PID, so
   * without this sweep they appear as 'running' in the DB but have no
   * in-memory instance — metric polling silently no-ops and the operator
   * has to click Start again after every systemd restart.
   *
   * Failures are logged, not thrown — a misconfigured sidecar shouldn't
   * block server startup.
   */
  async resumeSidecarNodes(): Promise<void> {
    const nodes = this.getAllNodes();
    for (const node of nodes) {
      if (!isSidecarNodeType(node.type)) continue;
      if (node.process.status !== 'running' && node.process.status !== 'starting') continue;
      if (this.nodes.has(node.id)) continue;
      try {
        let instance: BaseNode;
        if (node.type === 'neofura') {
          instance = new NeofuraNode(node);
        } else {
          continue;
        }
        instance.on('status', (status, previous) => {
          this.repo.updateStatus(node.id, status);
          this.emit('nodeStatus', { nodeId: node.id, status, previousStatus: previous });
        });
        instance.on('log', (entry) => {
          this.repo.saveLogEntry(node.id, entry);
          this.emit('nodeLog', { nodeId: node.id, entry });
        });
        await instance.start();
        this.nodes.set(node.id, instance);
        console.log(`♻️  Resumed sidecar ${node.name} (${node.type}) — polling observe-only target`);
      } catch (err) {
        console.warn(`⚠️  Failed to resume sidecar ${node.name}:`, err instanceof Error ? err.message : err);
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
      // Sidecars (observe-only) have no process to stop — flipping their
      // status to 'stopped' on graceful shutdown loses the desired-state
      // signal that resumeSidecarNodes() needs on next boot. Leave their
      // status alone; they'll be re-instantiated automatically.
      if (isSidecarNodeType(node.type)) {
        alreadyStoppedCount++;
        continue;
      }
      if (node.process.status === 'running') {
        try {
          await this.stopNode(node.id);
          stoppedCount++;
        } catch (error) {
          if (NodeManager.isOwnershipDeniedError(error)) {
            alreadyStoppedCount++;
            continue;
          }
          throw error;
        }
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
      if (this.getImportedOwnershipMode(node) === 'observe-only') {
        continue;
      }
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

  async cleanNodeLogs(nodeId: string, maxAgeDays = 30): Promise<number> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    this.assertCanWriteImportedNode(node, 'log file cleanup');
    return StorageManager.cleanOldLogs(node.paths.logs, maxAgeDays);
  }

  /**
   * Export a configuration snapshot for all managed nodes
   */
  exportConfiguration(): ExportedConfigurationSnapshot {
    return this.createSnapshotService().exportConfiguration();
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

      if (isManagedNodeDirectory(node)) {
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
  ): Promise<RestoreConfigurationResult> {
    return this.createSnapshotService().restoreConfiguration(snapshot, options);
  }

  private createSnapshotService(): NodeSnapshotService {
    return new NodeSnapshotService({
      getAllNodes: () => this.getAllNodes(),
      createNode: (request) => this.createNode(request),
      installPlugin: (nodeId, pluginId, config) => this.installPlugin(nodeId, pluginId, config),
      setPluginEnabled: (nodeId, pluginId, enabled) => this.setPluginEnabled(nodeId, pluginId, enabled),
      resetAllNodeData: () => this.resetAllNodeData(),
    });
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

    this.assertCanWriteImportedNode(node, 'configuration updates');
    this.assertSettingsRuntimeValues(request.settings);

    if (request.settings && Object.hasOwn(request.settings, 'storageEngine')) {
      this.assertStorageEngineUpdateIsolated(request);
      const storageEngine = this.assertStorageEngine(request.settings.storageEngine);
      return this.ensureStorageEngine(nodeId, storageEngine);
    }

    const updates: string[] = [];
    const values: (string | number)[] = [];
    let nextSettings: NodeSettings | undefined;
    const previousSettings = node.settings || {};

    if (request.name) {
      updates.push('name = ?');
      values.push(request.name);
    }

    if (request.settings) {
      const currentSettings = node.settings || {};
      const sanitizedSettings = this.sanitizeSettingsUpdate(node, request.settings) ?? {};
      const newSettings = { ...currentSettings, ...sanitizedSettings };
      this.assertSecureSignerBindingValid(node.type, newSettings.keyProtection);
      nextSettings = newSettings;
      updates.push('settings = ?');
      values.push(JSON.stringify(newSettings));
    }

    updates.push('updated_at = ?');
    values.push(Date.now());
    values.push(nodeId);

    this.repo.updateNode(nodeId, updates, values);

    try {
      if (nextSettings?.keyProtection?.mode === "secure-signer") {
        await this.syncNodeSecureSigner(nodeId);
      }

      const updatedNode = this.getNode(nodeId)!;
      await ConfigManager.writeNodeConfig(updatedNode, this.getEnabledPluginIds(nodeId));

      return updatedNode;
    } catch (error) {
      if (nextSettings) {
        this.repo.updateNode(nodeId, ['settings = ?', 'updated_at = ?'], [JSON.stringify(previousSettings), Date.now(), nodeId]);
        const restoredNode = this.getNode(nodeId);
        if (restoredNode) {
          await ConfigManager.writeNodeConfig(restoredNode, this.getEnabledPluginIds(nodeId));
        }
      }
      throw error;
    }
  }

  async ensureStorageEngine(nodeId: string, storageEngine: StorageEngine): Promise<NodeInstance> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
    }
    this.assertCanWriteImportedNode(node, 'storage engine updates');
    const nextStorageEngine = this.assertStorageEngine(storageEngine);

    const previousSettings = node.settings || {};
    const previousInstalledPlugins = this.pluginManager.getInstalledPlugins(nodeId);
    const previousRocksDBStore = previousInstalledPlugins.find((plugin) => plugin.id === 'RocksDBStore');
    const previousLevelDBStore = previousInstalledPlugins.find((plugin) => plugin.id === 'LevelDBStore');
    const nextSettings: NodeSettings = { ...previousSettings, storageEngine: nextStorageEngine };
    this.repo.updateNode(nodeId, ['settings = ?', 'updated_at = ?'], [JSON.stringify(nextSettings), Date.now(), nodeId]);

    try {
      if (node.type === 'neo-cli' && nextStorageEngine === 'rocksdb') {
        await this.pluginManager.upsertPlugin(nodeId, 'RocksDBStore', node.version, {});
      }
      if (node.type === 'neo-cli') {
        this.disableConflictingStoragePlugin(nodeId, nextStorageEngine);
      }

      const updatedNode = this.getNode(nodeId)!;
      await ConfigManager.writeNodeConfig(updatedNode, this.getEnabledPluginIds(nodeId));
      return updatedNode;
    } catch (error) {
      this.repo.updateNode(nodeId, ['settings = ?', 'updated_at = ?'], [JSON.stringify(previousSettings), Date.now(), nodeId]);
      if (previousRocksDBStore) {
        try {
          this.pluginManager.setPluginEnabled(nodeId, 'RocksDBStore', previousRocksDBStore.enabled);
        } catch (rollbackError) {
          console.error(`Failed to rollback RocksDBStore enablement for ${nodeId}:`, rollbackError);
        }
      } else if (this.pluginManager.getInstalledPlugins(nodeId).some((plugin) => plugin.id === 'RocksDBStore')) {
        try {
          this.pluginManager.removePluginState(nodeId, 'RocksDBStore');
        } catch (rollbackError) {
          console.error(`Failed to rollback RocksDBStore installation for ${nodeId}:`, rollbackError);
        }
      }
      if (previousLevelDBStore) {
        try {
          this.pluginManager.setPluginEnabled(nodeId, 'LevelDBStore', previousLevelDBStore.enabled);
        } catch (rollbackError) {
          console.error(`Failed to rollback LevelDBStore enablement for ${nodeId}:`, rollbackError);
        }
      }
      const restoredNode = this.getNode(nodeId);
      if (restoredNode) {
        try {
          await ConfigManager.writeNodeConfig(restoredNode, this.getEnabledPluginIds(nodeId));
        } catch (rollbackError) {
          console.error(`Failed to rewrite restored node config for ${nodeId}:`, rollbackError);
        }
      }
      throw error;
    }
  }

  private disableConflictingStoragePlugin(nodeId: string, storageEngine: StorageEngine): void {
    const conflictingPluginId: PluginId = storageEngine === 'rocksdb' ? 'LevelDBStore' : 'RocksDBStore';
    const conflictingPlugin = this.pluginManager
      .getInstalledPlugins(nodeId)
      .find((plugin) => plugin.id === conflictingPluginId);
    if (conflictingPlugin?.enabled) {
      this.pluginManager.setPluginEnabled(nodeId, conflictingPluginId, false);
    }
  }

  async updateImportedNodeOwnership(nodeId: string, ownershipMode: ImportedNodeOwnershipMode): Promise<NodeInstance> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    if (isManagedNodeDirectory(node)) {
      throw Errors.nodeOwnershipNotImported();
    }

    const normalizedMode = normalizeImportedOwnershipMode(ownershipMode);
    const nextSettings: NodeSettings = {
      ...(node.settings || {}),
      import: {
        ...node.settings.import,
        imported: true,
        ownershipMode: normalizedMode,
      },
    };

    this.repo.updateNode(nodeId, ['settings = ?', 'updated_at = ?'], [JSON.stringify(nextSettings), Date.now(), nodeId]);
    const updatedNode = this.getNode(nodeId);
    if (!updatedNode) {
      throw Errors.nodeNotFound(nodeId);
    }
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

    let stopDeniedByOwnership = false;

    // Stop if running. Observe-only imported nodes are detached from NeoNexus
    // metadata without mutating their external process.
    if (node.process.status === 'running') {
      try {
        await this.stopNode(nodeId);
      } catch (error) {
        if (!NodeManager.isOwnershipDeniedError(error)) {
          throw error;
        }
        stopDeniedByOwnership = true;
      }
    }

    // Release ports
    this.portManager.releasePorts(node.ports);

    // Remove from database (CASCADE handles node_processes, node_metrics, node_plugins, logs)
    this.repo.deleteNode(nodeId);

    // Remove from memory
    const runningNode = this.nodes.get(nodeId);
    if (runningNode) {
      if (stopDeniedByOwnership) {
        runningNode.detach();
      } else {
        runningNode.destroy();
      }
      this.nodes.delete(nodeId);
    }

    // Clean up filesystem data for NeoNexus-managed nodes
    if (removeFiles && isManagedNodeDirectory(node)) {
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

    this.assertCanManageImportedProcess(node, 'starting processes');
    this.assertCanStartImportedNeoCliProcess(node);

    // Check if already running. Imported/native processes can be marked
    // running in the repository even when NeoNexus does not own an in-memory
    // child process wrapper after restart or attach. Reconcile stale or
    // untrusted PIDs instead of permanently blocking restart.
    if (node.process.status === 'running' || node.process.status === 'starting') {
      if (getAttachedProcessState(node) === 'active') {
        throw Errors.nodeAlreadyRunning();
      }
      this.repo.updateStatus(nodeId, 'stopped');
    }

    // Check if already running in this process
    const existingNode = this.nodes.get(nodeId);
    if (existingNode?.isRunning()) {
      throw Errors.nodeAlreadyRunning();
    }

    // Create appropriate node instance
    const nodeInstance = node.type === 'neo-cli'
      ? new NeoCliNode(node)
      : node.type === 'neox-go'
        ? new NeoXNode(node)
        : node.type === 'neofura'
          ? new NeofuraNode(node)
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
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    const nodeInstance = this.nodes.get(nodeId);
    if (!nodeInstance) {
      if (node.process.status === 'running' || node.process.status === 'starting') {
        this.assertCanManageImportedProcess(node, 'stopping processes');

        if (getAttachedProcessState(node) !== 'active') {
          this.repo.updateStatus(nodeId, 'stopped');
          return;
        }

        await this.stopAttachedProcess(nodeId, node.process.pid!, force);
        return;
      }

      throw Errors.nodeNotRunning();
    }

    this.assertCanManageImportedProcess(node, 'stopping processes');

    await nodeInstance.stop(force);
    this.nodes.delete(nodeId);

    // Update database
    this.repo.updateStatus(nodeId, 'stopped');
  }

  private getImportedOwnershipMode(node: Pick<NodeInstance, 'id' | 'paths' | 'settings'>): ImportedNodeOwnershipMode | null {
    const importSettings = node.settings?.import;
    if (importSettings) {
      return normalizeImportedOwnershipMode(importSettings.ownershipMode);
    }

    // Legacy databases can contain imported native nodes created before
    // ownership metadata existed. Treat complete node records outside the
    // NeoNexus managed node directory layout as observe-only until explicitly
    // adopted. Some tests and aggregate helpers use partial node projections
    // without paths.base; those are not enough evidence to infer external
    // ownership, so keep them as regular managed metadata.
    if (!node.paths?.base) {
      return null;
    }
    return isManagedNodeDirectory(node) ? null : 'observe-only';
  }

  private sanitizeSettingsUpdate(node: NodeInstance, settings?: NodeSettings): NodeSettings | undefined {
    if (!settings) {
      return undefined;
    }

    const mutableSettings = stripReservedImportSettings(settings) ?? {};
    if (node.settings?.import) {
      return {
        ...mutableSettings,
        import: node.settings.import,
      };
    }

    return mutableSettings;
  }

  private assertSettingsRuntimeValues(settings?: Partial<NodeSettings>): void {
    if (!settings) {
      return;
    }
    if (Object.hasOwn(settings, 'storageEngine')) {
      this.assertStorageEngine(settings.storageEngine);
    }
    if (Object.hasOwn(settings, 'syncStrategy')) {
      this.assertSyncStrategy(settings.syncStrategy);
    }
  }

  private assertStorageEngineUpdateIsolated(request: UpdateNodeRequest): void {
    if (!request.settings || !Object.hasOwn(request.settings, 'storageEngine')) {
      return;
    }
    const hasOtherSettings = Object.keys(request.settings).some((key) => key !== 'storageEngine');
    if (request.name || hasOtherSettings) {
      throw new Error('Storage engine updates must be submitted separately');
    }
  }

  private assertStorageEngine(storageEngine: unknown): StorageEngine {
    if (storageEngine === 'leveldb' || storageEngine === 'rocksdb') {
      return storageEngine;
    }
    throw new Error('Invalid storage engine: use leveldb or rocksdb');
  }

  private assertSyncStrategy(syncStrategy: unknown): RoleSyncStrategy {
    if (syncStrategy === 'full' || syncStrategy === 'light' || syncStrategy === 'fast-sync') {
      return syncStrategy;
    }
    throw new Error('Invalid sync strategy: use full, light, or fast-sync');
  }

  private assertSyncMode(syncMode: unknown): SyncMode | undefined {
    if (syncMode === undefined) {
      return undefined;
    }
    if (syncMode === 'full' || syncMode === 'light') {
      return syncMode;
    }
    throw new Error('Invalid sync mode: use full or light');
  }

  private assertCanWriteImportedNode(node: NodeInstance, action: string): void {
    const mode = this.getImportedOwnershipMode(node);
    if (mode === 'observe-only') {
      throw Errors.nodeOwnershipDenied(action, mode);
    }
  }

  private assertCanManageImportedProcess(node: NodeInstance, action: string): void {
    const mode = this.getImportedOwnershipMode(node);
    if (mode && mode !== 'managed-process') {
      throw Errors.nodeOwnershipDenied(action, mode);
    }
  }

  private assertCanStartImportedNeoCliProcess(node: NodeInstance): void {
    if (this.getImportedOwnershipMode(node) !== 'managed-process' || node.type !== 'neo-cli') {
      return;
    }

    const configuredPath = node.paths.config;
    const primaryConfigPath = configuredPath.toLowerCase().endsWith('.json')
      ? configuredPath
      : join(configuredPath, 'config.json');
    const workingDirectoryConfigPath = join(node.paths.base, 'config.json');

    if (resolve(primaryConfigPath) !== resolve(workingDirectoryConfigPath)) {
      throw Errors.importedNeoCliConfigStartUnsupported(primaryConfigPath);
    }
  }

  private async stopAttachedProcess(nodeId: string, pid: number, force: boolean): Promise<void> {
    const signal = force ? 'SIGKILL' : 'SIGTERM';
    this.repo.updateStatus(nodeId, 'stopping', pid);

    try {
      process.kill(pid, signal);
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code === 'ESRCH') {
        this.repo.updateStatus(nodeId, 'stopped');
        return;
      }
      this.repo.updateStatus(nodeId, 'error', pid);
      throw error;
    }

    if (await this.waitForProcessExit(pid, force)) {
      this.repo.updateStatus(nodeId, 'stopped');
      return;
    }

    this.repo.updateStatus(nodeId, 'error', pid);
    throw new Error(`Timed out waiting for process ${pid} to exit`);
  }

  private async waitForProcessExit(pid: number, force: boolean): Promise<boolean> {
    const maxAttempts = force ? 5 : 20;
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      if (!isProcessAlive(pid)) {
        return true;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    return !isProcessAlive(pid);
  }

  private static isOwnershipDeniedError(error: unknown): boolean {
    return Boolean(
      error &&
      typeof error === 'object' &&
      'code' in error &&
      (error as { code?: unknown }).code === 'NODE_OWNERSHIP_DENIED',
    );
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

    return StorageManager.getNodeStorageInfo(nodeId, node.paths, {
      activeDataContextId: node.settings.activeDataContextId,
    });
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

    this.assertCanWriteImportedNode(node, 'plugin installation');

    await this.installPluginAndRewriteConfig(node, pluginId, config);
  }

  private async installPluginAndRewriteConfig(
    node: NodeInstance,
    pluginId: PluginId,
    config?: Record<string, unknown>,
    previousEnabledPlugins = this.getEnabledPluginIds(node.id),
  ): Promise<void> {
    await this.pluginManager.installPlugin(node.id, pluginId, node.version, config);
    try {
      // Update node config
      await ConfigManager.writeNodeConfig(node, this.getEnabledPluginIds(node.id));
    } catch (error) {
      try {
        this.pluginManager.removePluginState(node.id, pluginId);
      } catch (rollbackError) {
        console.error(`Failed to rollback plugin installation for ${node.id}/${pluginId}:`, rollbackError);
      }
      try {
        await ConfigManager.writeNodeConfig(node, previousEnabledPlugins);
      } catch (rollbackError) {
        console.error(`Failed to rewrite restored node config for ${node.id}:`, rollbackError);
      }
      throw error;
    }
  }

  updatePluginConfig(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): void {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
    }
    this.assertCanWriteImportedNode(node, 'plugin configuration updates');
    this.assertCanMutatePlugin(node, pluginId);
    this.pluginManager.updatePluginConfig(nodeId, pluginId, config ?? {});
  }

  async uninstallPlugin(nodeId: string, pluginId: PluginId): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
    }
    this.assertCanWriteImportedNode(node, 'plugin removal');
    this.assertCanMutatePlugin(node, pluginId);
    await this.pluginManager.uninstallPlugin(nodeId, pluginId);
  }

  setPluginEnabled(nodeId: string, pluginId: PluginId, enabled: boolean): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    if (node.process.status === 'running') {
      throw Errors.nodeRunning();
    }
    this.assertCanWriteImportedNode(node, 'plugin enablement changes');
    if (!enabled) {
      this.assertCanMutatePlugin(node, pluginId);
    }
    const previousPlugin = this.pluginManager.getInstalledPlugins(nodeId).find((plugin) => plugin.id === pluginId);
    this.pluginManager.setPluginEnabled(nodeId, pluginId, enabled);
    return ConfigManager.writeNodeConfig(node, this.getEnabledPluginIds(nodeId, node)).catch((error: unknown) => {
      if (previousPlugin) {
        try {
          this.pluginManager.setPluginEnabled(nodeId, pluginId, previousPlugin.enabled);
        } catch (rollbackError) {
          console.error(`Failed to rollback plugin enablement for ${nodeId}/${pluginId}:`, rollbackError);
        }
      }
      throw error;
    });
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

    this.assertCanWriteImportedNode(node, 'secure signer plugin configuration');

    const signerProfileId = keyProtection.signerProfileId;
    this.assertSecureSignerCompatibility(node.type, signerProfileId);

    const profile = this.secureSignerManager.getProfile(signerProfileId!);
    if (!profile || !profile.enabled) {
      throw Errors.signerNotAvailable(signerProfileId!);
    }

    const config = this.secureSignerManager.buildSignClientConfig(profile, keyProtection.policy);
    const installedPlugins = this.pluginManager.getInstalledPlugins(nodeId);
    const existingSignClient = installedPlugins.find((plugin) => plugin.id === "SignClient");
    const previousEnabledPlugins = installedPlugins
      .filter((plugin) => plugin.enabled)
      .map((plugin) => plugin.id);

    if (existingSignClient) {
      try {
        this.pluginManager.updatePluginConfig(nodeId, "SignClient", config);
        this.pluginManager.setPluginEnabled(nodeId, "SignClient", true);
        await ConfigManager.writeNodeConfig(node, this.getEnabledPluginIds(nodeId));
      } catch (error) {
        try {
          this.pluginManager.updatePluginConfig(nodeId, "SignClient", existingSignClient.config ?? {});
        } catch (rollbackError) {
          console.error(`Failed to rollback SignClient config for ${nodeId}:`, rollbackError);
        }
        try {
          this.pluginManager.setPluginEnabled(nodeId, "SignClient", existingSignClient.enabled);
        } catch (rollbackError) {
          console.error(`Failed to rollback SignClient enablement for ${nodeId}:`, rollbackError);
        }
        try {
          await ConfigManager.writeNodeConfig(node, previousEnabledPlugins);
        } catch (rollbackError) {
          console.error(`Failed to rewrite restored node config for ${nodeId}:`, rollbackError);
        }
        throw error;
      }
      return;
    }

    await this.installPluginAndRewriteConfig(node, "SignClient", config, previousEnabledPlugins);
  }

  private getEnabledPluginIds(nodeId: string, node?: Pick<NodeInstance, "plugins">): PluginId[] {
    const pluginManager = (this as unknown as { pluginManager?: PluginManager }).pluginManager;
    const plugins = pluginManager?.getInstalledPlugins(nodeId) ?? node?.plugins ?? [];
    return plugins
      .filter((plugin) => plugin.enabled)
      .map((plugin) => plugin.id);
  }

  private assertCanMutatePlugin(node: NodeInstance, pluginId: PluginId): void {
    if (pluginId === "SignClient" && node.settings?.keyProtection?.mode === "secure-signer") {
      throw Errors.signerPluginRequired();
    }
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
      const metrics = await collectRuntimeMetrics(nodeInstance);

      this.repo.saveMetrics(nodeId, metrics);
      this.emit('nodeMetrics', { nodeId, metrics });
    } catch (error) {
      console.error(`Failed to update metrics for ${nodeId}:`, error);
    }
  }

  getRepository(): NodeRepository {
    return this.repo;
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

  private assertSecureSignerBindingValid(
    nodeType: NodeConfig["type"],
    keyProtection?: NodeKeyProtectionSettings,
  ): void {
    if (!keyProtection || keyProtection.mode !== "secure-signer") {
      return;
    }

    this.assertSecureSignerCompatibility(nodeType, keyProtection.signerProfileId);
    const profile = this.secureSignerManager.getProfile(keyProtection.signerProfileId!);
    if (!profile || !profile.enabled) {
      throw Errors.signerNotAvailable(keyProtection.signerProfileId!);
    }

    // Build the SignClient config as preflight validation before persisting
    // node settings. This catches fail-closed policy errors (for example
    // hardware-required policy bound to a software signer) before the database
    // can claim a protection mode that the installed plugin config does not enforce.
    this.secureSignerManager.buildSignClientConfig(profile, keyProtection.policy);
  }
}
