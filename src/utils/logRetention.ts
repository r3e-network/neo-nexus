import type Database from 'better-sqlite3';

export function pruneNodeLogs(db: Database.Database, nodeId: string, maxRows: number): number {
  const countRow = db.prepare('SELECT COUNT(*) as c FROM logs WHERE node_id = ?').get(nodeId) as { c: number };
  if (countRow.c <= maxRows) return 0;
  const toDelete = countRow.c - maxRows;
  return db.prepare('DELETE FROM logs WHERE id IN (SELECT id FROM logs WHERE node_id = ? ORDER BY timestamp ASC LIMIT ?)').run(nodeId, toDelete).changes;
}

export function pruneAllLogs(db: Database.Database, maxRowsPerNode: number, globalMax: number): number {
  let totalDeleted = 0;
  const nodeIds = db.prepare('SELECT DISTINCT node_id FROM logs').all() as Array<{ node_id: string }>;
  for (const { node_id } of nodeIds) {
    totalDeleted += pruneNodeLogs(db, node_id, maxRowsPerNode);
  }
  const globalCount = db.prepare('SELECT COUNT(*) as c FROM logs').get() as { c: number };
  if (globalCount.c > globalMax) {
    totalDeleted += db.prepare('DELETE FROM logs WHERE id IN (SELECT id FROM logs ORDER BY timestamp ASC LIMIT ?)').run(globalCount.c - globalMax).changes;
  }
  return totalDeleted;
}
