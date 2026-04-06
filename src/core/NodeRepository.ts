import type Database from 'better-sqlite3';
import type { NodeConfig, NodeStatus, NodeMetrics, LogEntry } from '../types/index';
import { NodeRow, ProcessRow, MetricsRow, nodeRowToConfig } from '../types/database';

export class NodeRepository {
  constructor(private db: Database.Database) {}

  saveNode(config: NodeConfig): void {
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

  getNodeConfig(nodeId: string): NodeConfig | null {
    const stmt = this.db.prepare('SELECT * FROM nodes WHERE id = ?');
    const row = stmt.get(nodeId) as NodeRow | undefined;

    if (!row) return null;

    return nodeRowToConfig(row);
  }

  getProcess(nodeId: string): { status: NodeStatus; pid?: number; errorMessage?: string } {
    const stmt = this.db.prepare('SELECT * FROM node_processes WHERE node_id = ?');
    const row = stmt.get(nodeId) as ProcessRow | undefined;

    return {
      status: row?.status ?? 'stopped',
      pid: row?.pid ?? undefined,
      errorMessage: row?.error_message ?? undefined,
    };
  }

  getMetrics(nodeId: string): NodeMetrics | undefined {
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

  getAllNodeIds(): string[] {
    const stmt = this.db.prepare('SELECT id FROM nodes');
    const rows = stmt.all() as Array<{ id: string }>;
    return rows.map(row => row.id);
  }

  updateStatus(nodeId: string, status: NodeStatus, pid?: number): void {
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

  saveLogEntry(nodeId: string, entry: LogEntry): void {
    const stmt = this.db.prepare(`
      INSERT INTO logs (node_id, timestamp, level, source, message)
      VALUES (?, ?, ?, ?, ?)
    `);
    stmt.run(nodeId, entry.timestamp, entry.level, entry.source, entry.message);
  }

  saveMetrics(nodeId: string, metrics: NodeMetrics): void {
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

  updateSyncProgress(nodeId: string, syncProgress: number): void {
    const stmt = this.db.prepare('UPDATE node_metrics SET sync_progress = ? WHERE node_id = ?');
    stmt.run(syncProgress, nodeId);
  }

  deleteNode(nodeId: string): void {
    const stmt = this.db.prepare('DELETE FROM nodes WHERE id = ?');
    stmt.run(nodeId);
  }

  private static readonly ALLOWED_UPDATE_COLUMNS = new Set([
    'name', 'settings', 'version', 'updated_at',
  ]);

  updateNode(_nodeId: string, updates: string[], values: (string | number)[]): void {
    // Validate that all SET clauses use whitelisted columns with parameterized values
    for (const clause of updates) {
      const match = clause.match(/^(\w+)\s*=\s*\?$/);
      if (!match || !NodeRepository.ALLOWED_UPDATE_COLUMNS.has(match[1])) {
        throw new Error(`Invalid update clause: ${clause}`);
      }
    }

    const stmt = this.db.prepare(`
      UPDATE nodes
      SET ${updates.join(', ')}
      WHERE id = ?
    `);
    stmt.run(...values);
  }

  transaction<T>(fn: () => T): T {
    return this.db.transaction(fn)();
  }
}
