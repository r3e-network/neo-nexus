import crypto from 'node:crypto';
import type Database from 'better-sqlite3';
import type { NodeDataContext, RoleSyncStrategy, StorageEngine } from '../types';
import type { NodeDataContextRow } from '../types/database';
import { validateDataContextId } from '../utils/paths';

export interface CreateNodeDataContextInput {
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
}

export class NodeDataContextManager {
  constructor(private readonly db: Database.Database) {}

  listContexts(nodeId: string): NodeDataContext[] {
    const rows = this.db
      .prepare('SELECT * FROM node_data_contexts WHERE node_id = ? ORDER BY created_at')
      .all(nodeId) as NodeDataContextRow[];
    return rows.map((row) => this.mapRow(row));
  }

  getActiveContext(nodeId: string): NodeDataContext | null {
    const row = this.db
      .prepare('SELECT * FROM node_data_contexts WHERE node_id = ? AND active = 1')
      .get(nodeId) as NodeDataContextRow | undefined;
    return row ? this.mapRow(row) : null;
  }

  createContext(nodeId: string, input: CreateNodeDataContextInput): NodeDataContext {
    const now = Date.now();
    const hasExisting = this.listContexts(nodeId).length > 0;
    const context: NodeDataContext = {
      id: `ctx-${crypto.randomUUID()}`,
      nodeId,
      label: input.label,
      storageEngine: input.storageEngine,
      syncStrategy: input.syncStrategy,
      checkpointHeight: input.checkpointHeight,
      checkpointHash: input.checkpointHash,
      snapshotId: input.snapshotId,
      active: !hasExisting,
      createdAt: now,
      updatedAt: now,
    };

    this.db
      .prepare(`
        INSERT INTO node_data_contexts (
          id, node_id, label, storage_engine, sync_strategy, checkpoint_height,
          checkpoint_hash, snapshot_id, active, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `)
      .run(
        context.id,
        nodeId,
        context.label,
        context.storageEngine,
        context.syncStrategy,
        context.checkpointHeight ?? null,
        context.checkpointHash ?? null,
        context.snapshotId ?? null,
        context.active ? 1 : 0,
        context.createdAt,
        context.updatedAt,
      );

    return context;
  }

  activateContext(nodeId: string, contextId: string): NodeDataContext {
    validateDataContextId(contextId);
    const context = this.listContexts(nodeId).find((candidate) => candidate.id === contextId);
    if (!context) throw new Error(`Data context ${contextId} not found for node ${nodeId}`);

    const tx = this.db.transaction(() => {
      const now = Date.now();
      this.db.prepare('UPDATE node_data_contexts SET active = 0, updated_at = ? WHERE node_id = ?').run(now, nodeId);
      this.db
        .prepare('UPDATE node_data_contexts SET active = 1, updated_at = ? WHERE node_id = ? AND id = ?')
        .run(now, nodeId, contextId);
    });
    tx();

    return this.getActiveContext(nodeId)!;
  }

  private mapRow(row: NodeDataContextRow): NodeDataContext {
    return {
      id: row.id,
      nodeId: row.node_id,
      label: row.label,
      storageEngine: row.storage_engine as StorageEngine,
      syncStrategy: row.sync_strategy as RoleSyncStrategy,
      checkpointHeight: row.checkpoint_height ?? undefined,
      checkpointHash: row.checkpoint_hash ?? undefined,
      snapshotId: row.snapshot_id ?? undefined,
      active: row.active === 1,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }
}
