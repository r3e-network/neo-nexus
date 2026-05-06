import { describe, expect, it, vi } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

vi.unmock("better-sqlite3");
vi.unmock("node:fs/promises");

import Database from "better-sqlite3";
import { NodeDataContextManager } from "../../src/core/NodeDataContextManager";
import { StorageManager } from "../../src/core/StorageManager";

function createManager() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE node_data_contexts (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      label TEXT NOT NULL,
      storage_engine TEXT NOT NULL,
      sync_strategy TEXT NOT NULL,
      checkpoint_height INTEGER,
      checkpoint_hash TEXT,
      snapshot_id TEXT,
      active INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE UNIQUE INDEX idx_node_data_contexts_one_active
      ON node_data_contexts(node_id)
      WHERE active = 1;
  `);
  return new NodeDataContextManager(db);
}

describe("NodeDataContextManager", () => {
  it("creates a first context as active", () => {
    const manager = createManager();
    const context = manager.createContext("node-1", {
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
      checkpointHeight: 5800000,
      checkpointHash: "0xabc",
    });

    expect(context.active).toBe(true);
    expect(manager.listContexts("node-1")).toHaveLength(1);
  });

  it("activates one context and deactivates the previous active context", () => {
    const manager = createManager();
    const first = manager.createContext("node-1", { label: "default", storageEngine: "leveldb", syncStrategy: "full" });
    const second = manager.createContext("node-1", { label: "state", storageEngine: "rocksdb", syncStrategy: "fast-sync" });

    manager.activateContext("node-1", second.id);

    expect(manager.getActiveContext("node-1")?.id).toBe(second.id);
    expect(manager.listContexts("node-1").find((ctx) => ctx.id === first.id)?.active).toBe(false);
  });

  it.each(["../ctx", "ctx/state", "ctx\\state", ""])("rejects unsafe context id %j during activation", (contextId) => {
    const manager = createManager();

    expect(() => manager.activateContext("node-1", contextId)).toThrow(/Invalid data context id/);
  });
});

describe("StorageManager data context paths", () => {
  it("creates the active data context directory instead of the fallback data directory", () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-dirs-"));
    const paths = {
      base: baseDir,
      data: path.join(baseDir, "Data"),
      logs: path.join(baseDir, "Logs"),
      config: path.join(baseDir, "config"),
    };

    StorageManager.ensureNodeDirectories(paths, { activeDataContextId: "ctx-state" });

    expect(fs.existsSync(path.join(baseDir, "data-contexts", "ctx-state"))).toBe(true);
    expect(fs.existsSync(paths.data)).toBe(false);
    expect(fs.existsSync(paths.logs)).toBe(true);
    expect(fs.existsSync(paths.config)).toBe(true);
  });

  it("reports chain storage from the active data context directory", async () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-storage-"));
    const paths = {
      base: baseDir,
      data: path.join(baseDir, "Data"),
      logs: path.join(baseDir, "Logs"),
      config: path.join(baseDir, "config"),
    };
    const contextPath = path.join(baseDir, "data-contexts", "ctx-state");

    fs.mkdirSync(paths.data, { recursive: true });
    fs.mkdirSync(paths.logs, { recursive: true });
    fs.mkdirSync(contextPath, { recursive: true });
    fs.writeFileSync(path.join(paths.data, "old-chain.dat"), "legacy-data");
    fs.writeFileSync(path.join(contextPath, "chain.dat"), "ctx");

    const info = await StorageManager.getNodeStorageInfo("node-1", paths, { activeDataContextId: "ctx-state" });

    expect(info.chain.path).toBe(contextPath);
    expect(info.chain.size).toBe(3);
  });

  it("does not follow symlinks while measuring chain storage", async () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-symlink-size-"));
    const outsideDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-outside-"));
    fs.writeFileSync(path.join(outsideDir, "outside.dat"), "outside-secret");
    fs.mkdirSync(path.join(baseDir, "Data"), { recursive: true });
    fs.writeFileSync(path.join(baseDir, "Data", "inside.dat"), "inside");
    fs.symlinkSync(outsideDir, path.join(baseDir, "Data", "outside-link"), "dir");

    const size = await StorageManager.getDirectorySize(path.join(baseDir, "Data"));

    expect(size).toBe("inside".length);
  });

  it("removes chain-data symlinks without deleting their targets", async () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-symlink-clean-"));
    const outsideDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-outside-clean-"));
    const outsideFile = path.join(outsideDir, "outside.dat");
    const linkPath = path.join(baseDir, "Data", "outside-link");
    fs.writeFileSync(outsideFile, "outside-secret");
    fs.mkdirSync(path.join(baseDir, "Data"), { recursive: true });
    fs.symlinkSync(outsideDir, linkPath, "dir");

    await StorageManager.cleanChainData(path.join(baseDir, "Data"));

    expect(fs.existsSync(linkPath)).toBe(false);
    expect(fs.existsSync(outsideFile)).toBe(true);
  });

  it.each(["../ctx", "ctx/state", "ctx\\state", ""])("rejects unsafe context id %j when resolving storage paths", (contextId) => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-unsafe-"));
    const paths = {
      base: baseDir,
      data: path.join(baseDir, "Data"),
    };

    expect(() => StorageManager.getEffectiveChainDataPath(paths, { activeDataContextId: contextId })).toThrow(/Invalid data context id/);
  });

  it.each(["../ctx", "ctx/state", "ctx\\state", ""])("rejects unsafe context id %j before creating storage directories", (contextId) => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-context-create-"));
    const paths = {
      base: baseDir,
      data: path.join(baseDir, "Data"),
      logs: path.join(baseDir, "Logs"),
      config: path.join(baseDir, "config"),
    };

    expect(() => StorageManager.ensureNodeDirectories(paths, { activeDataContextId: contextId })).toThrow(/Invalid data context id/);
    expect(fs.existsSync(path.join(baseDir, "data-contexts"))).toBe(false);
  });
});
