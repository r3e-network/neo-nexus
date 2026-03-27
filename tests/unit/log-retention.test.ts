import { vi, describe, it, expect, beforeEach } from 'vitest';

// Use real better-sqlite3 for these tests
vi.unmock('better-sqlite3');

import Database from 'better-sqlite3';
import { pruneNodeLogs, pruneAllLogs } from '../../src/utils/logRetention';

function createTestDb(): Database.Database {
  const db = new Database(':memory:');
  db.exec(`
    CREATE TABLE logs (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      node_id TEXT NOT NULL,
      timestamp INTEGER NOT NULL,
      level TEXT NOT NULL,
      source TEXT,
      message TEXT NOT NULL
    )
  `);
  return db;
}

function insertLogs(db: Database.Database, nodeId: string, count: number, startTime = 0): void {
  const insert = db.prepare('INSERT INTO logs (node_id, timestamp, level, message) VALUES (?, ?, ?, ?)');
  for (let i = 0; i < count; i++) {
    insert.run(nodeId, startTime + i, 'info', `message ${i}`);
  }
}

function countLogs(db: Database.Database, nodeId?: string): number {
  if (nodeId) {
    return (db.prepare('SELECT COUNT(*) as c FROM logs WHERE node_id = ?').get(nodeId) as { c: number }).c;
  }
  return (db.prepare('SELECT COUNT(*) as c FROM logs').get() as { c: number }).c;
}

describe('pruneNodeLogs', () => {
  let db: Database.Database;

  beforeEach(() => {
    db = createTestDb();
  });

  it('prunes 70 rows when 100 logs exist and max is 30', () => {
    insertLogs(db, 'node-1', 100);
    const deleted = pruneNodeLogs(db, 'node-1', 30);
    expect(deleted).toBe(70);
    expect(countLogs(db, 'node-1')).toBe(30);
  });

  it('returns 0 when count is already at or below max', () => {
    insertLogs(db, 'node-1', 20);
    const deleted = pruneNodeLogs(db, 'node-1', 30);
    expect(deleted).toBe(0);
    expect(countLogs(db, 'node-1')).toBe(20);
  });

  it('returns 0 when count equals max exactly', () => {
    insertLogs(db, 'node-1', 30);
    const deleted = pruneNodeLogs(db, 'node-1', 30);
    expect(deleted).toBe(0);
    expect(countLogs(db, 'node-1')).toBe(30);
  });

  it('keeps the newest logs (oldest are deleted)', () => {
    insertLogs(db, 'node-1', 10, 1000);
    pruneNodeLogs(db, 'node-1', 5);
    const rows = db.prepare('SELECT timestamp FROM logs WHERE node_id = ? ORDER BY timestamp ASC').all('node-1') as Array<{ timestamp: number }>;
    expect(rows.length).toBe(5);
    // Should keep rows with timestamps 1005..1009
    expect(rows[0].timestamp).toBe(1005);
  });
});

describe('pruneAllLogs', () => {
  let db: Database.Database;

  beforeEach(() => {
    db = createTestDb();
  });

  it('prunes per-node when each node exceeds maxRowsPerNode', () => {
    insertLogs(db, 'node-1', 60);
    insertLogs(db, 'node-2', 60);
    const deleted = pruneAllLogs(db, 30, 1000000);
    expect(deleted).toBe(60); // 30 from each node
    expect(countLogs(db, 'node-1')).toBe(30);
    expect(countLogs(db, 'node-2')).toBe(30);
  });

  it('does not prune nodes that are under the per-node limit', () => {
    insertLogs(db, 'node-1', 100);
    insertLogs(db, 'node-2', 20);
    pruneAllLogs(db, 50, 1000000);
    expect(countLogs(db, 'node-1')).toBe(50);
    expect(countLogs(db, 'node-2')).toBe(20);
  });

  it('applies global cap after per-node pruning', () => {
    insertLogs(db, 'node-1', 60, 0);
    insertLogs(db, 'node-2', 60, 1000);
    // Per-node max is 100 (no per-node pruning), globalMax is 80
    const deleted = pruneAllLogs(db, 100, 80);
    expect(deleted).toBe(40); // 120 - 80 = 40
    expect(countLogs(db)).toBe(80);
  });

  it('returns 0 when everything is within limits', () => {
    insertLogs(db, 'node-1', 10);
    insertLogs(db, 'node-2', 10);
    const deleted = pruneAllLogs(db, 50, 1000);
    expect(deleted).toBe(0);
  });
});
