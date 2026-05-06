import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("node:fs", () => ({
  mkdirSync: vi.fn(),
}));

describe("initializeDatabase", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("ensures the base data directory exists before opening sqlite", async () => {
    vi.doMock("better-sqlite3", () => ({
      default: vi.fn(() => ({
        pragma: vi.fn(),
        exec: vi.fn(),
        prepare: vi.fn((sql: string) => {
          if (sql.includes("SELECT COUNT(*) as count FROM users")) {
            return { get: vi.fn(() => ({ count: 1 })) };
          }
          return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
        }),
      })),
    }));

    const fs = await import("node:fs");
    const { initializeDatabase } = await import("../../src/database/schema");
    const { paths } = await import("../../src/utils/paths");

    await initializeDatabase();

    expect(fs.mkdirSync).toHaveBeenCalledWith(paths.base, { recursive: true });
  });

  it("does not create predictable default admin credentials on first boot", async () => {
    const insertDefaultAdmin = vi.fn();

    vi.doMock("better-sqlite3", () => ({
      default: vi.fn(() => ({
        pragma: vi.fn(),
        exec: vi.fn(),
        prepare: vi.fn((sql: string) => {
          if (sql.includes("SELECT COUNT(*) as count FROM users")) {
            return { get: vi.fn(() => ({ count: 0 })) };
          }
          if (sql.includes("INSERT INTO users")) {
            return { run: insertDefaultAdmin };
          }
          return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
        }),
      })),
    }));

    const { initializeDatabase } = await import("../../src/database/schema");

    await initializeDatabase();

    expect(insertDefaultAdmin).not.toHaveBeenCalled();
  });

  it("enables sqlite foreign key enforcement for cascade cleanup", async () => {
    const pragma = vi.fn();

    vi.doMock("better-sqlite3", () => ({
      default: vi.fn(() => ({
        pragma,
        exec: vi.fn(),
        prepare: vi.fn((sql: string) => {
          if (sql.includes("SELECT COUNT(*) as count FROM users")) {
            return { get: vi.fn(() => ({ count: 1 })) };
          }
          return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
        }),
      })),
    }));

    const { initializeDatabase } = await import("../../src/database/schema");

    await initializeDatabase();

    expect(pragma).toHaveBeenCalledWith("journal_mode = WAL");
    expect(pragma).toHaveBeenCalledWith("foreign_keys = ON");
  });

  it("creates role, data context, fast sync, and private network tables", async () => {
    const execSql: string[] = [];

    vi.doMock("better-sqlite3", () => ({
      default: vi.fn(() => ({
        pragma: vi.fn(),
        exec: vi.fn((sql: string) => {
          execSql.push(sql);
        }),
        prepare: vi.fn((sql: string) => {
          if (sql.includes("SELECT name FROM sqlite_master WHERE type = 'table'")) {
            return {
              all: vi.fn(() => {
                const schemaSql = execSql.join("\n");
                const tableMatches = [...schemaSql.matchAll(/CREATE TABLE IF NOT EXISTS\s+([a-z_]+)/g)];
                return tableMatches.map((match) => ({ name: match[1] }));
              }),
            };
          }
          if (sql.includes("SELECT COUNT(*) as count FROM users")) {
            return { get: vi.fn(() => ({ count: 1 })) };
          }
          return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
        }),
      })),
    }));

    const { initializeDatabase } = await import("../../src/database/schema");
    const db = await initializeDatabase();

    const tables = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table'").all() as Array<{ name: string }>;
    const tableNames = new Set(tables.map((table) => table.name));

    expect(tableNames.has("node_role_profiles")).toBe(true);
    expect(tableNames.has("node_role_applications")).toBe(true);
    expect(tableNames.has("node_data_contexts")).toBe(true);
    expect(tableNames.has("fast_sync_snapshots")).toBe(true);
    expect(tableNames.has("private_network_plans")).toBe(true);
  });

  it("creates a partial unique index for active data contexts per node", async () => {
    const execSql: string[] = [];

    vi.doMock("better-sqlite3", () => ({
      default: vi.fn(() => ({
        pragma: vi.fn(),
        exec: vi.fn((sql: string) => {
          execSql.push(sql);
        }),
        prepare: vi.fn((sql: string) => {
          if (sql.includes("SELECT COUNT(*) as count FROM users")) {
            return { get: vi.fn(() => ({ count: 1 })) };
          }
          return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
        }),
      })),
    }));

    const { initializeDatabase } = await import("../../src/database/schema");

    await initializeDatabase();

    const schemaSql = execSql.join("\n").replace(/\s+/g, " ");
    expect(schemaSql).toContain(
      "CREATE UNIQUE INDEX IF NOT EXISTS idx_node_data_contexts_one_active ON node_data_contexts(node_id) WHERE active = 1",
    );
  });
});
