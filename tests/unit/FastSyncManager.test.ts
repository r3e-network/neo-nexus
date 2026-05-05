import { describe, expect, it, vi } from "vitest";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { PassThrough } from "node:stream";

vi.unmock("better-sqlite3");
vi.unmock("fs/promises");
vi.unmock("node:fs/promises");

import Database from "better-sqlite3";
import { FastSyncManager, type FastSyncManagerOptions } from "../../src/core/FastSyncManager";
import type { FastSyncSnapshot } from "../../src/types";

const VALID_SHA = "a".repeat(64);

function createManager(options?: FastSyncManagerOptions) {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE fast_sync_snapshots (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      source_type TEXT NOT NULL,
      source TEXT NOT NULL,
      chain TEXT NOT NULL,
      network TEXT NOT NULL,
      node_type TEXT NOT NULL,
      storage_engine TEXT NOT NULL,
      height INTEGER NOT NULL,
      block_hash TEXT,
      sha256 TEXT NOT NULL,
      size_bytes INTEGER,
      signature TEXT,
      trusted INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      last_verified_at INTEGER
    );
  `);
  return new FastSyncManager(db, options);
}

function validInput(overrides: Record<string, unknown> = {}) {
  return {
    name: "N3 mainnet rocksdb",
    sourceType: "local",
    source: "/tmp/snapshot.tar.zst",
    chain: "n3",
    network: "mainnet",
    nodeType: "neo-go",
    storageEngine: "rocksdb",
    height: 12345,
    sha256: VALID_SHA,
    ...overrides,
  };
}

describe("FastSyncManager", () => {
  it.each([
    ["missing", undefined],
    ["too short", "abc"],
    ["non-hex", "g".repeat(64)],
  ])("rejects %s sha256 during registration", (_label, sha256) => {
    const manager = createManager();

    expect(() => manager.registerSnapshot(validInput({ sha256 }))).toThrow(/sha256/i);
  });

  it.each([
    ["missing name", { name: "" }, /name/i],
    ["missing source", { source: "" }, /source/i],
    ["invalid source type", { sourceType: "ftp" }, /sourceType/i],
    ["invalid chain", { chain: "legacy" }, /chain/i],
    ["invalid network", { network: "devnet" }, /network/i],
    ["invalid node type", { nodeType: "neo-python" }, /nodeType/i],
    ["invalid storage engine", { storageEngine: "sqlite" }, /storageEngine/i],
    ["zero height", { height: 0 }, /height/i],
    ["negative size", { sizeBytes: -1 }, /sizeBytes/i],
    ["trusted client flag", { trusted: true }, /trusted/i],
    ["n3 snapshot with neox network", { network: "neox-mainnet" }, /network.*n3/i],
    ["x snapshot with n3 node type", { chain: "x", network: "neox-mainnet", nodeType: "neo-go" }, /nodeType.*chain x/i],
  ])("rejects %s during registration", (_label, overrides, message) => {
    const manager = createManager();

    expect(() => manager.registerSnapshot(validInput(overrides))).toThrow(message);
  });

  it("marks user-registered snapshots untrusted until a server trust flow exists", () => {
    const manager = createManager();
    const snapshot = manager.registerSnapshot(validInput({ trusted: false }));

    expect(snapshot.trusted).toBe(false);
    expect(manager.listSnapshots()[0]?.trusted).toBe(false);
  });

  it("verifies a local snapshot sha256 and records file metadata", async () => {
    const manager = createManager();
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-fast-sync-"));
    const snapshotPath = path.join(tmpDir, "snapshot.bin");
    fs.writeFileSync(snapshotPath, Buffer.from("local snapshot bytes"));
    const sha256 = crypto.createHash("sha256").update(fs.readFileSync(snapshotPath)).digest("hex");

    const registered = manager.registerSnapshot(validInput({
      source: snapshotPath,
      sha256,
      sizeBytes: 1,
    }));
    const verified = await manager.verifySnapshot(registered.id);

    expect(verified.sha256).toBe(sha256);
    expect(verified.sizeBytes).toBe(fs.statSync(snapshotPath).size);
    expect(verified.lastVerifiedAt).toEqual(expect.any(Number));
    expect(manager.listSnapshots()[0]?.lastVerifiedAt).toBe(verified.lastVerifiedAt);
  });

  it("rejects local verification for non-local snapshot sources", async () => {
    const manager = createManager();
    const registered = manager.registerSnapshot(validInput({
      sourceType: "url",
      source: "https://example.com/snapshot.tar.zst",
    }));

    await expect(manager.verifySnapshot(registered.id)).rejects.toMatchObject({
      code: "FAST_SYNC_VERIFY_UNSUPPORTED",
      status: 400,
    });
  });

  it("returns not found when the local snapshot file is missing", async () => {
    const manager = createManager();
    const registered = manager.registerSnapshot(validInput({
      source: path.join(os.tmpdir(), `missing-${crypto.randomUUID()}.tar.zst`),
    }));

    await expect(manager.verifySnapshot(registered.id)).rejects.toMatchObject({
      code: "FAST_SYNC_SNAPSHOT_FILE_NOT_FOUND",
      status: 404,
    });
  });

  it("rejects local snapshots whose bytes do not match the manifest sha256", async () => {
    const manager = createManager();
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-fast-sync-"));
    const snapshotPath = path.join(tmpDir, "snapshot.bin");
    fs.writeFileSync(snapshotPath, Buffer.from("tampered snapshot bytes"));
    const registered = manager.registerSnapshot(validInput({
      source: snapshotPath,
      sha256: "b".repeat(64),
    }));

    await expect(manager.verifySnapshot(registered.id)).rejects.toMatchObject({
      code: "FAST_SYNC_SNAPSHOT_HASH_MISMATCH",
      status: 409,
    });
  });

  it("rejects snapshots that do not match node compatibility fields", () => {
    const manager = createManager();
    const snapshot = manager.registerSnapshot(validInput({
      network: "testnet",
      nodeType: "neo-go",
      storageEngine: "rocksdb",
    }));

    expect(() => manager.assertCompatible(snapshot, {
      chain: "n3",
      network: "mainnet",
      type: "neo-cli",
      storageEngine: "leveldb",
    })).toThrow(/network.*nodeType.*storageEngine/i);
  });

  it("accepts snapshots that match node compatibility fields", () => {
    const manager = createManager();
    const snapshot: FastSyncSnapshot = manager.registerSnapshot(validInput());

    expect(() => manager.assertCompatible(snapshot, {
      chain: "n3",
      network: "mainnet",
      type: "neo-go",
      storageEngine: "rocksdb",
    })).not.toThrow();
  });

  it("downloads URL snapshots only after public target validation", async () => {
    const bytes = Buffer.from("actual snapshot bytes");
    const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
    const downloadTransport = vi.fn(async (_source: URL, targetAddress: string, destination: string) => {
      expect(targetAddress).toBe("203.0.113.10");
      fs.writeFileSync(destination, bytes);
    });
    const manager = createManager({
      resolveHostname: vi.fn(async () => [{ address: "203.0.113.10", family: 4 }]),
      downloadTransport,
    });
    const snapshot = manager.registerSnapshot(validInput({
      sourceType: "url",
      source: "https://snapshots.example.test/mainnet.tar.zst",
      sha256,
    }));

    const downloaded = await manager.downloadSnapshot(snapshot.id);

    expect(downloadTransport).toHaveBeenCalledOnce();
    expect(downloaded).toMatchObject({
      sourceType: "local",
      sizeBytes: bytes.length,
    });
    expect(downloaded.source).toMatch(/mainnet\.tar\.zst$/);
  });

  it("blocks literal private fast-sync download targets before transport", async () => {
    const downloadTransport = vi.fn(async () => undefined);
    const manager = createManager({ downloadTransport });
    const snapshot = manager.registerSnapshot(validInput({
      sourceType: "url",
      source: "https://127.0.0.1/snapshot.tar.zst",
    }));

    await expect(manager.downloadSnapshot(snapshot.id)).rejects.toMatchObject({
      code: "FAST_SYNC_DOWNLOAD_PRIVATE_TARGET",
      status: 400,
    });
    expect(downloadTransport).not.toHaveBeenCalled();
  });

  it("blocks DNS-resolved private fast-sync download targets before transport", async () => {
    const downloadTransport = vi.fn(async () => undefined);
    const manager = createManager({
      resolveHostname: vi.fn(async () => [{ address: "10.0.0.25", family: 4 }]),
      downloadTransport,
    });
    const snapshot = manager.registerSnapshot(validInput({
      sourceType: "url",
      source: "https://snapshots.example.test/snapshot.tar.zst",
    }));

    await expect(manager.downloadSnapshot(snapshot.id)).rejects.toMatchObject({
      code: "FAST_SYNC_DOWNLOAD_PRIVATE_TARGET",
      status: 400,
    });
    expect(downloadTransport).not.toHaveBeenCalled();
  });

  it("blocks fast-sync download redirects instead of following them", async () => {
    const request = vi.fn((_options, callback: (response: NodeJS.ReadableStream & {
      statusCode?: number;
      headers: Record<string, string>;
      resume: () => NodeJS.ReadableStream;
    }) => void) => {
      const response = new PassThrough() as NodeJS.ReadableStream & {
        statusCode?: number;
        headers: Record<string, string>;
        resume: () => NodeJS.ReadableStream;
      };
      response.statusCode = 302;
      response.headers = { location: "https://169.254.169.254/latest/meta-data" };
      queueMicrotask(() => {
        callback(response);
        response.emit("end");
      });
      return { on: vi.fn(), end: vi.fn() };
    });
    const manager = createManager({
      resolveHostname: vi.fn(async () => [{ address: "203.0.113.10", family: 4 }]),
      httpsRequest: request as never,
    });
    const snapshot = manager.registerSnapshot(validInput({
      sourceType: "url",
      source: "https://snapshots.example.test/redirect.tar.zst",
    }));

    await expect(manager.downloadSnapshot(snapshot.id)).rejects.toMatchObject({
      code: "FAST_SYNC_DOWNLOAD_REDIRECT_BLOCKED",
      status: 400,
    });
    expect(request).toHaveBeenCalledOnce();
  });
});
