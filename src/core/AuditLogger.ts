import type Database from 'better-sqlite3';

export interface AuditEntry {
  action: string;
  resourceType: string;
  resourceId?: string;
  userId?: string;
  username?: string;
  details?: string;
  ipAddress?: string;
}

export interface AuditRecord extends AuditEntry {
  id: number;
  timestamp: number;
}

export class AuditLogger {
  constructor(private db: Database.Database) {}

  log(entry: AuditEntry): void {
    this.db.prepare('INSERT INTO audit_log (timestamp, user_id, username, action, resource_type, resource_id, details, ip_address) VALUES (?, ?, ?, ?, ?, ?, ?, ?)')
      .run(Date.now(), entry.userId ?? null, entry.username ?? null, entry.action, entry.resourceType, entry.resourceId ?? null, entry.details ?? null, entry.ipAddress ?? null);
  }

  query(opts: { limit?: number; offset?: number }): AuditRecord[] {
    return this.db.prepare('SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ? OFFSET ?')
      .all(opts.limit ?? 100, opts.offset ?? 0) as AuditRecord[];
  }

  prune(maxRows: number): number {
    const count = (this.db.prepare('SELECT COUNT(*) as c FROM audit_log').get() as { c: number }).c;
    if (count <= maxRows) return 0;
    return this.db.prepare('DELETE FROM audit_log WHERE id IN (SELECT id FROM audit_log ORDER BY timestamp ASC LIMIT ?)').run(count - maxRows).changes;
  }
}
