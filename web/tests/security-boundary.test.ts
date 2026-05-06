import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const TEST_DIR = dirname(fileURLToPath(import.meta.url));
const FRONTEND_SRC = join(TEST_DIR, "..", "src");
const SOURCE_EXTENSIONS = new Set([".ts", ".tsx"]);

const FORBIDDEN_DB_PATTERNS: Array<{ name: string; pattern: RegExp }> = [
  {
    name: "database driver import",
    pattern: /(?:from\s+|import\s*\(|require\s*\()\s*["'][^"']*(?:better-sqlite3|sqlite3|sql\.js|@libsql|pg|mysql2|knex|drizzle-orm|prisma)[^"']*["']/i,
  },
  { name: "database constructor", pattern: /\bnew\s+Database\s*\(/i },
  { name: "prepared statement API", pattern: /\.prepare\s*\(/i },
  { name: "DDL statement", pattern: /\b(?:CREATE|DROP|ALTER)\s+TABLE\b/i },
  { name: "SELECT statement", pattern: /\bSELECT\b[\s\S]{0,120}\bFROM\b/i },
  { name: "INSERT statement", pattern: /\bINSERT\b[\s\S]{0,120}\bINTO\b/i },
  { name: "UPDATE statement", pattern: /\bUPDATE\b[\s\S]{0,120}\bSET\b/i },
  { name: "DELETE statement", pattern: /\bDELETE\s+FROM\b/i },
  { name: "PRAGMA statement", pattern: /\bPRAGMA\b/i },
];

describe("frontend security boundary", () => {
  it("does not include database drivers, SQL statements, or prepared statement APIs", () => {
    const violations = collectSourceFiles(FRONTEND_SRC).flatMap((file) => {
      const source = readFileSync(file, "utf8");
      return FORBIDDEN_DB_PATTERNS
        .filter(({ pattern }) => pattern.test(source))
        .map(({ name }) => `${file.replace(`${FRONTEND_SRC}/`, "")}: ${name}`);
    });

    expect(violations).toEqual([]);
  });
});

function collectSourceFiles(dir: string): string[] {
  const result: string[] = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      result.push(...collectSourceFiles(path));
    } else if (SOURCE_EXTENSIONS.has(path.slice(path.lastIndexOf(".")))) {
      result.push(path);
    }
  }
  return result;
}
