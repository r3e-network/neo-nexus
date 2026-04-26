import { existsSync, rmSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { isAbsolute, join, relative, resolve } from 'node:path';
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
  ImportedNodeOwnershipMode,
  NodeSettings,
  NodeKeyProtectionSettings,
} from '../types/index';
import { paths, getNodePath, getNodeDataPath, getNodeLogsPath, getNodeConfigPath, getNodeWalletPath } from '../utils/paths';
import { NodeRepository } from './NodeRepository';
import { isProcessAlive, getProcessCommand, getProcessCwd, getProcessArgv } from '../utils/lifecycle';
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
      settings: {
        import: {
          imported: true,
          ownershipMode: NodeManager.normalizeImportedOwnershipMode(request.ownershipMode),
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

    if (this.getAttachedProcessState(node, pid) === 'active') {
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
      const candidates = result
        .split('\n')
        .map((line) => Number.parseInt(line.trim(), 10))
        .filter((pid, index, all): pid is number => this.isValidProcessId(pid) && all.indexOf(pid) === index)
        .map((pid) => ({ pid, score: this.scoreAttachCandidate(node, pid) }))
        .filter((candidate) => candidate.score > 0)
        .sort((a, b) => b.score - a.score);

      for (const candidate of candidates) {
        if (this.getAttachedProcessState(node, candidate.pid) === 'active') {
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
      settings: NodeManager.stripReservedImportSettings(request.settings) || {},
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
  exportConfiguration(): {
    generatedAt: number;
    version: string;
    nodes: Array<Omit<NodeInstance, 'process' | 'metrics'>>;
  } {
    const nodes = this.getAllNodes().map(({ process: _process, metrics: _metrics, ...config }) => config);

    let version = "2.0.0";
    try {
      const pkg = JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf-8'));
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

      if (NodeManager.isManagedNodeDirectory(node)) {
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
          settings: NodeManager.stripReservedImportSettings(node.settings),
        });

        for (const plugin of node.plugins ?? []) {
          await this.installPlugin(restoredNode.id, plugin.id, plugin.config);
          if (plugin.enabled === false) {
            this.setPluginEnabled(restoredNode.id, plugin.id, false);
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

    this.assertCanWriteImportedNode(node, 'configuration updates');

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
      await ConfigManager.writeNodeConfig(updatedNode, updatedNode.plugins?.map(p => p.id));

      return updatedNode;
    } catch (error) {
      if (nextSettings) {
        this.repo.updateNode(nodeId, ['settings = ?', 'updated_at = ?'], [JSON.stringify(previousSettings), Date.now(), nodeId]);
        const restoredNode = this.getNode(nodeId);
        if (restoredNode) {
          await ConfigManager.writeNodeConfig(restoredNode, restoredNode.plugins?.map(p => p.id));
        }
      }
      throw error;
    }
  }

  async updateImportedNodeOwnership(nodeId: string, ownershipMode: ImportedNodeOwnershipMode): Promise<NodeInstance> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }

    if (NodeManager.isManagedNodeDirectory(node)) {
      throw Errors.nodeOwnershipNotImported();
    }

    const normalizedMode = NodeManager.normalizeImportedOwnershipMode(ownershipMode);
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
    if (removeFiles && NodeManager.isManagedNodeDirectory(node)) {
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
      if (this.getAttachedProcessState(node) === 'active') {
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

        if (this.getAttachedProcessState(node) !== 'active') {
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

  private getAttachedProcessState(node: NodeInstance, pid = node.process.pid): 'active' | 'stale' {
    if (!this.isValidProcessId(pid)) {
      return 'stale';
    }

    if (!isProcessAlive(pid)) {
      return 'stale';
    }

    return this.isExpectedNodeProcess(node, pid) ? 'active' : 'stale';
  }

  private getImportedOwnershipMode(node: Pick<NodeInstance, 'id' | 'paths' | 'settings'>): ImportedNodeOwnershipMode | null {
    const importSettings = node.settings?.import;
    if (importSettings) {
      return NodeManager.normalizeImportedOwnershipMode(importSettings.ownershipMode);
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
    return NodeManager.isManagedNodeDirectory(node) ? null : 'observe-only';
  }

  private sanitizeSettingsUpdate(node: NodeInstance, settings?: NodeSettings): NodeSettings | undefined {
    if (!settings) {
      return undefined;
    }

    const mutableSettings = NodeManager.stripReservedImportSettings(settings) ?? {};
    if (node.settings?.import) {
      return {
        ...mutableSettings,
        import: node.settings.import,
      };
    }

    return mutableSettings;
  }

  private static stripReservedImportSettings(settings?: NodeSettings): NodeSettings | undefined {
    if (!settings) {
      return undefined;
    }

    const { import: _reservedImport, ...mutableSettings } = settings;
    return mutableSettings;
  }

  private static normalizeImportedOwnershipMode(mode: unknown): ImportedNodeOwnershipMode {
    if (mode === 'managed-config' || mode === 'managed-process') {
      return mode;
    }
    return 'observe-only';
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

  private scoreAttachCandidate(node: NodeInstance, pid: number): number {
    if (!this.isValidProcessId(pid) || !isProcessAlive(pid)) {
      return 0;
    }

    const command = getProcessCommand(pid);
    if (!command || !this.commandMatchesNodeType(command, node.type)) {
      return 0;
    }

    let score = 2;
    const cwd = getProcessCwd(pid);
    if (cwd && NodeManager.isPathWithinOrEqual(cwd, node.paths.base)) {
      score += 4;
    }

    const argv = getProcessArgv(pid) ?? [];
    const pathCandidates = argv.flatMap((arg) => NodeManager.extractPathCandidates(arg));
    if (pathCandidates.some((candidate) => isAbsolute(candidate) && resolve(candidate) === resolve(node.paths.config))) {
      score += 8;
    }
    if (pathCandidates.some((candidate) => this.pathCandidateMatchesNode(candidate, node))) {
      score += 3;
    }

    const argvText = argv.join(' ');
    const expectedPorts = [node.ports.rpc, node.ports.p2p, node.ports.websocket, node.ports.metrics]
      .filter((port): port is number => Number.isInteger(port));
    for (const port of expectedPorts) {
      if (new RegExp(`(^|[^0-9])${port}([^0-9]|$)`).test(argvText)) {
        score += 1;
      }
    }

    return score;
  }

  private isValidProcessId(pid: unknown): pid is number {
    return Number.isSafeInteger(pid) && Number(pid) > 0;
  }

  private isExpectedNodeProcess(node: NodeInstance, pid: number): boolean {
    const command = getProcessCommand(pid);
    if (!command || !this.commandMatchesNodeType(command, node.type)) {
      return false;
    }

    const cwd = getProcessCwd(pid);
    if (cwd && NodeManager.isPathWithinOrEqual(cwd, node.paths.base)) {
      return true;
    }

    const argv = getProcessArgv(pid);
    if (!argv) {
      return false;
    }

    return argv
      .flatMap((arg) => NodeManager.extractPathCandidates(arg))
      .some((candidate) => this.pathCandidateMatchesNode(candidate, node));
  }

  private pathCandidateMatchesNode(candidate: string, node: NodeInstance): boolean {
    if (!isAbsolute(candidate)) {
      return false;
    }

    return NodeManager.isPathWithinOrEqual(candidate, node.paths.base);
  }

  private static extractPathCandidates(arg: string): string[] {
    const candidates = [arg];
    const equalsIndex = arg.indexOf('=');
    if (equalsIndex >= 0) {
      candidates.push(arg.slice(equalsIndex + 1));
    }
    return candidates.map((candidate) => candidate.trim()).filter(Boolean);
  }

  private commandMatchesNodeType(command: string, type: NodeConfig['type']): boolean {
    const normalizedCommand = command.toLowerCase();
    if (type === 'neo-cli') {
      return normalizedCommand.includes('neo-cli') || normalizedCommand.includes('neo.cli');
    }
    return normalizedCommand.includes('neo-go');
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

  private static isPathWithinOrEqual(pathToCheck: string, allowedPrefix: string): boolean {
    const resolvedPath = resolve(pathToCheck);
    const resolvedPrefix = resolve(allowedPrefix);
    const pathRelativeToPrefix = relative(resolvedPrefix, resolvedPath);
    return pathRelativeToPrefix === '' || (!pathRelativeToPrefix.startsWith('..') && !isAbsolute(pathRelativeToPrefix));
  }

  private static isSafeGeneratedNodeId(nodeId: string): boolean {
    return /^node-[A-Za-z0-9-]+$/.test(nodeId);
  }

  private static isOwnershipDeniedError(error: unknown): boolean {
    return Boolean(
      error &&
      typeof error === 'object' &&
      'code' in error &&
      (error as { code?: unknown }).code === 'NODE_OWNERSHIP_DENIED',
    );
  }

  private static isManagedNodeDirectory(node: Pick<NodeInstance, 'id' | 'paths' | 'settings'>): boolean {
    if (node.settings?.import || !NodeManager.isSafeGeneratedNodeId(node.id)) {
      return false;
    }

    const managedRoot = resolve(paths.nodes);
    const expectedNodePath = resolve(getNodePath(node.id));
    const expectedRelativeToRoot = relative(managedRoot, expectedNodePath);
    if (
      expectedRelativeToRoot === '' ||
      expectedRelativeToRoot.startsWith('..') ||
      isAbsolute(expectedRelativeToRoot) ||
      expectedRelativeToRoot.includes('/') ||
      expectedRelativeToRoot.includes('\\')
    ) {
      return false;
    }

    return resolve(node.paths.base) === expectedNodePath;
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

    this.assertCanWriteImportedNode(node, 'plugin installation');

    await this.pluginManager.installPlugin(nodeId, pluginId, node.version, config);

    // Update node config
    const plugins = this.pluginManager.getInstalledPlugins(nodeId).map(p => p.id);
    await ConfigManager.writeNodeConfig(node, plugins);
  }

  updatePluginConfig(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): void {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    this.assertCanWriteImportedNode(node, 'plugin configuration updates');
    this.pluginManager.updatePluginConfig(nodeId, pluginId, config ?? {});
  }

  async uninstallPlugin(nodeId: string, pluginId: PluginId): Promise<void> {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    this.assertCanWriteImportedNode(node, 'plugin removal');
    await this.pluginManager.uninstallPlugin(nodeId, pluginId);
  }

  setPluginEnabled(nodeId: string, pluginId: PluginId, enabled: boolean): void {
    const node = this.getNode(nodeId);
    if (!node) {
      throw Errors.nodeNotFound(nodeId);
    }
    this.assertCanWriteImportedNode(node, 'plugin enablement changes');
    this.pluginManager.setPluginEnabled(nodeId, pluginId, enabled);
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
