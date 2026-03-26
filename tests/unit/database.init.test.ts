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
});
