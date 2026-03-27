import { vi, describe, it, expect, beforeEach } from 'vitest';

// Use real better-sqlite3 for these tests
vi.unmock('better-sqlite3');

import Database from 'better-sqlite3';
import { AuditLogger } from '../../src/core/AuditLogger';

function createTestDb(): Database.Database {
  const db = new Database(':memory:');
  db.exec(`
    CREATE TABLE audit_log (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      timestamp INTEGER NOT NULL,
      user_id TEXT,
      username TEXT,
      action TEXT NOT NULL,
      resource_type TEXT NOT NULL,
      resource_id TEXT,
      details TEXT,
      ip_address TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
  `);
  return db;
}

describe('AuditLogger', () => {
  let db: Database.Database;
  let logger: AuditLogger;

  beforeEach(() => {
    db = createTestDb();
    logger = new AuditLogger(db);
  });

  describe('log and query', () => {
    it('logs entries and query returns them newest first', () => {
      logger.log({ action: 'node.start', resourceType: 'node', resourceId: 'node-1', username: 'admin' });
      // Add a small delay to ensure different timestamps
      const now = Date.now();
      db.prepare('UPDATE audit_log SET timestamp = ? WHERE action = ?').run(now - 1000, 'node.start');

      logger.log({ action: 'node.stop', resourceType: 'node', resourceId: 'node-1', username: 'admin' });

      const entries = logger.query({});
      expect(entries.length).toBe(2);
      // Newest first: node.stop should be first
      expect(entries[0].action).toBe('node.stop');
      expect(entries[1].action).toBe('node.start');
    });

    it('stores all fields correctly', () => {
      logger.log({
        action: 'user.login',
        resourceType: 'user',
        resourceId: 'user-123',
        userId: 'uid-1',
        username: 'alice',
        details: 'successful login',
        ipAddress: '127.0.0.1',
      });

      const entries = logger.query({ limit: 1 });
      const entry = entries[0] as unknown as Record<string, unknown>;
      expect(entry.action).toBe('user.login');
      expect(entry.resource_type).toBe('user');
      expect(entry.resource_id).toBe('user-123');
      expect(entry.username).toBe('alice');
      expect(entry.details).toBe('successful login');
      expect(entry.ip_address).toBe('127.0.0.1');
      expect(entry.timestamp).toBeGreaterThan(0);
    });

    it('respects limit and offset', () => {
      for (let i = 0; i < 5; i++) {
        logger.log({ action: `action-${i}`, resourceType: 'test' });
      }
      const page1 = logger.query({ limit: 2, offset: 0 });
      const page2 = logger.query({ limit: 2, offset: 2 });
      expect(page1.length).toBe(2);
      expect(page2.length).toBe(2);
      expect(page1[0].action).not.toBe(page2[0].action);
    });
  });

  describe('prune', () => {
    it('prunes oldest entries when over maxRows', () => {
      for (let i = 0; i < 10; i++) {
        logger.log({ action: `action-${i}`, resourceType: 'test' });
      }
      const deleted = logger.prune(5);
      expect(deleted).toBe(5);
      const remaining = logger.query({ limit: 100 });
      expect(remaining.length).toBe(5);
    });

    it('returns 0 when count is at or below maxRows', () => {
      for (let i = 0; i < 3; i++) {
        logger.log({ action: `action-${i}`, resourceType: 'test' });
      }
      expect(logger.prune(5)).toBe(0);
      expect(logger.prune(3)).toBe(0);
    });
  });
});
