import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import crypto from "node:crypto";
import express from "express";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import request from "supertest";
import Database from "better-sqlite3";
import { createFastSyncRouter } from "../../src/api/routes/fastSync";
import { FastSyncManager, type FastSyncManagerOptions } from "../../src/core/FastSyncManager";
import { createAuthMiddleware, type SessionUser } from "../../src/api/middleware/auth";
import { requireAdmin } from "../../src/api/middleware/roles";

vi.unmock("better-sqlite3");
vi.unmock("fs/promises");
vi.unmock("node:fs/promises");

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

function validPayload(overrides: Record<string, unknown> = {}) {
  return {
    name: "N3 mainnet",
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

function bearerToken(): string {
  return "Bearer valid-token";
}

describe("Actual fast sync router", () => {
  let app: express.Application;
  let mockManager: {
    listSnapshots: ReturnType<typeof vi.fn>;
    registerSnapshot: ReturnType<typeof vi.fn>;
    verifySnapshot: ReturnType<typeof vi.fn>;
    downloadSnapshot: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockManager = {
      listSnapshots: vi.fn(),
      registerSnapshot: vi.fn(),
      verifySnapshot: vi.fn(),
      downloadSnapshot: vi.fn(),
    };

    app.use("/api/fast-sync", createFastSyncRouter(mockManager as never));
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  const createProtectedApp = (user: SessionUser | null) => {
    const protectedApp = express();
    protectedApp.use(express.json());
    protectedApp.use(
      "/api/fast-sync",
      createAuthMiddleware({ verifySession: vi.fn(() => user) }),
      requireAdmin,
      createFastSyncRouter(mockManager as never),
    );
    return protectedApp;
  };

  it("lists registered snapshot manifests", async () => {
    mockManager.listSnapshots.mockReturnValue([
      { id: "snap-1", name: "N3 mainnet", sha256: "a".repeat(64) },
    ]);

    const response = await request(app).get("/api/fast-sync/snapshots");

    expect(response.status).toBe(200);
    expect(response.body.snapshots).toHaveLength(1);
  });

  it("registers a snapshot manifest", async () => {
    mockManager.registerSnapshot.mockReturnValue({
      id: "snap-1",
      name: "N3 mainnet",
      sha256: "a".repeat(64),
    });

    const payload = validPayload();
    const response = await request(app).post("/api/fast-sync/snapshots").send(payload);

    expect(response.status).toBe(201);
    expect(response.body.snapshot.id).toBe("snap-1");
    expect(mockManager.registerSnapshot).toHaveBeenCalledWith(payload);
  });

  it("triggers local snapshot verification", async () => {
    mockManager.verifySnapshot.mockResolvedValue({
      id: "snap-1",
      name: "N3 mainnet",
      sizeBytes: 128,
      lastVerifiedAt: 123456,
    });

    const response = await request(app).post("/api/fast-sync/snapshots/snap-1/verify");

    expect(response.status).toBe(200);
    expect(response.body.snapshot.lastVerifiedAt).toBe(123456);
    expect(mockManager.verifySnapshot).toHaveBeenCalledWith("snap-1");
  });

  it("downloads a remote snapshot manifest", async () => {
    mockManager.downloadSnapshot.mockResolvedValue({
      id: "snap-1",
      name: "N3 mainnet",
      sourceType: "local",
      source: "/tmp/neonexus-fast-sync/snap-1/snapshot.tar.zst",
      sizeBytes: 128,
      lastVerifiedAt: 123456,
    });

    const response = await request(app).post("/api/fast-sync/snapshots/snap-1/download");

    expect(response.status).toBe(200);
    expect(response.body.snapshot.sourceType).toBe("local");
    expect(mockManager.downloadSnapshot).toHaveBeenCalledWith("snap-1");
  });

  it("returns structured errors from manager failures", async () => {
    mockManager.verifySnapshot.mockRejectedValue(new Error("Snapshot snap-404 not found"));

    const response = await request(app).post("/api/fast-sync/snapshots/snap-404/verify");

    expect(response.status).toBe(404);
    expect(response.body.error).toBe("Snapshot snap-404 not found");
    expect(response.body.suggestion).toBeDefined();
  });

  it("requires authentication before accessing fast sync routes", async () => {
    const protectedApp = createProtectedApp({
      id: "admin-1",
      username: "admin",
      role: "admin",
    });

    const response = await request(protectedApp).get("/api/fast-sync/snapshots");

    expect(response.status).toBe(401);
    expect(response.body.code).toBe("NO_TOKEN");
    expect(mockManager.listSnapshots).not.toHaveBeenCalled();
  });

  it("requires admin access before accessing fast sync routes", async () => {
    const viewer: SessionUser = { id: "test-user-id", username: "admin", role: "viewer" };
    const protectedApp = createProtectedApp(viewer);

    const response = await request(protectedApp)
      .get("/api/fast-sync/snapshots")
      .set("Authorization", bearerToken());

    expect(response.status).toBe(403);
    expect(response.body.code).toBe("ADMIN_REQUIRED");
    expect(mockManager.listSnapshots).not.toHaveBeenCalled();
  });

  it("allows admin users through the protected fast sync stack", async () => {
    const admin: SessionUser = { id: "test-user-id", username: "admin", role: "admin" };
    const protectedApp = createProtectedApp(admin);
    mockManager.listSnapshots.mockReturnValue([]);

    const response = await request(protectedApp)
      .get("/api/fast-sync/snapshots")
      .set("Authorization", bearerToken());

    expect(response.status).toBe(200);
    expect(response.body.snapshots).toEqual([]);
  });

  it("returns a client error for invalid manifest payloads with the real manager", async () => {
    const realApp = express();
    realApp.use(express.json());
    realApp.use("/api/fast-sync", createFastSyncRouter(createManager()));

    const response = await request(realApp).post("/api/fast-sync/snapshots").send([]);

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("FAST_SYNC_SNAPSHOT_INVALID");
  });

  it("rejects impossible chain and network combinations with the real manager", async () => {
    const realApp = express();
    realApp.use(express.json());
    realApp.use("/api/fast-sync", createFastSyncRouter(createManager()));

    const response = await request(realApp)
      .post("/api/fast-sync/snapshots")
      .send(validPayload({ network: "neox-mainnet" }));

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/network.*n3/i);
  });

  it("does not allow clients to mark user-provided snapshots trusted", async () => {
    const realApp = express();
    realApp.use(express.json());
    realApp.use("/api/fast-sync", createFastSyncRouter(createManager()));

    const response = await request(realApp)
      .post("/api/fast-sync/snapshots")
      .send(validPayload({ trusted: true }));

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/trusted/i);
  });

  it("returns conflict when local snapshot bytes do not match the manifest", async () => {
    const manager = createManager();
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-fast-sync-"));
    const snapshotPath = path.join(tmpDir, "snapshot.bin");
    fs.writeFileSync(snapshotPath, Buffer.from("actual snapshot bytes"));
    const snapshot = manager.registerSnapshot(validPayload({
      source: snapshotPath,
      sha256: "b".repeat(64),
    }));
    const realApp = express();
    realApp.use(express.json());
    realApp.use("/api/fast-sync", createFastSyncRouter(manager));

    const response = await request(realApp).post(`/api/fast-sync/snapshots/${snapshot.id}/verify`);

    expect(response.status).toBe(409);
    expect(response.body.code).toBe("FAST_SYNC_SNAPSHOT_HASH_MISMATCH");
  });

  it("downloads and verifies URL snapshots with the real manager", async () => {
    const bytes = Buffer.from("actual snapshot bytes");
    const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
    const manager = createManager({
      resolveHostname: vi.fn(async () => [{ address: "203.0.113.10", family: 4 }]),
      downloadTransport: vi.fn(async (_source, _targetAddress, destination) => {
        fs.writeFileSync(destination, bytes);
      }),
    });
    const snapshot = manager.registerSnapshot(validPayload({
      sourceType: "url",
      source: "https://snapshots.example.test/mainnet.tar.zst",
      sha256,
    }));
    const realApp = express();
    realApp.use(express.json());
    realApp.use("/api/fast-sync", createFastSyncRouter(manager));

    const response = await request(realApp).post(`/api/fast-sync/snapshots/${snapshot.id}/download`);

    expect(response.status).toBe(200);
    expect(response.body.snapshot).toMatchObject({
      sourceType: "local",
      sizeBytes: bytes.length,
    });
    expect(response.body.snapshot.source).toMatch(/mainnet\.tar\.zst$/);
    expect(response.body.snapshot.lastVerifiedAt).toBeTypeOf("number");
  });
});
