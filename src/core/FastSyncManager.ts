import crypto from "node:crypto";
import { createReadStream } from "node:fs";
import { open } from "node:fs/promises";
import type Database from "better-sqlite3";
import type {
  FastSyncSnapshot,
  FastSyncSourceType,
  NodeChain,
  NodeNetwork,
  NodeSettings,
  NodeType,
  StorageEngine,
} from "../types";
import type { FastSyncSnapshotRow } from "../types/database";
import { ApiError } from "../api/errors";

const SOURCE_TYPES = ["local", "url", "catalog"] as const satisfies readonly FastSyncSourceType[];
const CHAINS = ["n3", "x"] as const satisfies readonly NodeChain[];
const NETWORKS = ["mainnet", "testnet", "private", "neox-mainnet", "neox-testnet"] as const satisfies readonly NodeNetwork[];
const NODE_TYPES = ["neo-cli", "neo-go", "neox-go"] as const satisfies readonly NodeType[];
const STORAGE_ENGINES = ["leveldb", "rocksdb"] as const satisfies readonly StorageEngine[];
const N3_NETWORKS = new Set<NodeNetwork>(["mainnet", "testnet", "private"]);
const X_NETWORKS = new Set<NodeNetwork>(["neox-mainnet", "neox-testnet"]);
const N3_NODE_TYPES = new Set<NodeType>(["neo-cli", "neo-go"]);
const SHA256_PATTERN = /^[a-f0-9]{64}$/i;
const FAST_SYNC_INVALID_SUGGESTION = "Provide a complete fast sync snapshot manifest with matching chain, network, node type, storage engine, height, and sha256.";

export interface RegisterFastSyncSnapshotInput {
  name: unknown;
  sourceType: unknown;
  source: unknown;
  chain: unknown;
  network: unknown;
  nodeType: unknown;
  storageEngine: unknown;
  height: unknown;
  blockHash?: unknown;
  sha256: unknown;
  sizeBytes?: unknown;
  signature?: unknown;
  trusted?: unknown;
}

export interface FastSyncCompatibleNode {
  chain: NodeChain;
  network: NodeNetwork;
  type: NodeType;
  storageEngine?: StorageEngine;
  settings?: Pick<NodeSettings, "storageEngine">;
}

export class FastSyncManager {
  constructor(private readonly db: Database.Database) {}

  listSnapshots(): FastSyncSnapshot[] {
    const rows = this.db
      .prepare("SELECT * FROM fast_sync_snapshots ORDER BY created_at DESC")
      .all() as FastSyncSnapshotRow[];
    return rows.map((row) => this.mapRow(row));
  }

  getSnapshot(id: string): FastSyncSnapshot | null {
    const row = this.db
      .prepare("SELECT * FROM fast_sync_snapshots WHERE id = ?")
      .get(id) as FastSyncSnapshotRow | undefined;
    return row ? this.mapRow(row) : null;
  }

  registerSnapshot(input: RegisterFastSyncSnapshotInput): FastSyncSnapshot {
    const snapshot = this.validateRegistrationInput(input);
    const now = Date.now();
    const id = `snapshot-${crypto.randomUUID()}`;

    this.db
      .prepare(`
        INSERT INTO fast_sync_snapshots (
          id, name, source_type, source, chain, network, node_type, storage_engine,
          height, block_hash, sha256, size_bytes, signature, trusted, created_at,
          last_verified_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `)
      .run(
        id,
        snapshot.name,
        snapshot.sourceType,
        snapshot.source,
        snapshot.chain,
        snapshot.network,
        snapshot.nodeType,
        snapshot.storageEngine,
        snapshot.height,
        snapshot.blockHash ?? null,
        snapshot.sha256,
        snapshot.sizeBytes ?? null,
        snapshot.signature ?? null,
        snapshot.trusted ? 1 : 0,
        now,
        null,
      );

    return {
      id,
      ...snapshot,
      createdAt: now,
    };
  }

  async verifySnapshot(id: string): Promise<FastSyncSnapshot> {
    const snapshot = this.getSnapshot(id);
    if (!snapshot) {
      throw new ApiError(
        "FAST_SYNC_SNAPSHOT_NOT_FOUND",
        `Fast sync snapshot ${id} not found`,
        "Register the snapshot manifest first, then retry verification.",
        404,
      );
    }
    if (snapshot.sourceType !== "local") {
      throw new ApiError(
        "FAST_SYNC_VERIFY_UNSUPPORTED",
        `Fast sync snapshot ${id} cannot be verified locally because sourceType is ${snapshot.sourceType}`,
        "Only local snapshot manifests can be verified by this operation.",
      );
    }

    const { sha256: actualSha256, sizeBytes } = await this.hashLocalFile(snapshot.source);
    if (actualSha256 !== snapshot.sha256.toLowerCase()) {
      throw new ApiError(
        "FAST_SYNC_SNAPSHOT_HASH_MISMATCH",
        `Fast sync snapshot ${id} sha256 mismatch: expected ${snapshot.sha256}, got ${actualSha256}`,
        "Replace the file or update the manifest with the verified digest before using this snapshot.",
        409,
      );
    }

    const lastVerifiedAt = Date.now();
    this.db
      .prepare("UPDATE fast_sync_snapshots SET size_bytes = ?, last_verified_at = ? WHERE id = ?")
      .run(sizeBytes, lastVerifiedAt, id);

    return {
      ...snapshot,
      sizeBytes,
      lastVerifiedAt,
    };
  }

  assertCompatible(snapshot: FastSyncSnapshot, node: FastSyncCompatibleNode): void {
    const nodeStorageEngine = node.storageEngine ?? node.settings?.storageEngine ?? "leveldb";
    const mismatches: string[] = [];

    if (snapshot.chain !== node.chain) mismatches.push(`chain snapshot=${snapshot.chain} node=${node.chain}`);
    if (snapshot.network !== node.network) mismatches.push(`network snapshot=${snapshot.network} node=${node.network}`);
    if (snapshot.nodeType !== node.type) mismatches.push(`nodeType snapshot=${snapshot.nodeType} node=${node.type}`);
    if (snapshot.storageEngine !== nodeStorageEngine) {
      mismatches.push(`storageEngine snapshot=${snapshot.storageEngine} node=${nodeStorageEngine}`);
    }

    if (mismatches.length > 0) {
      throw new Error(`Fast sync snapshot ${snapshot.id} is not compatible: ${mismatches.join(", ")}`);
    }
  }

  private validateRegistrationInput(input: RegisterFastSyncSnapshotInput): Omit<FastSyncSnapshot, "id" | "createdAt" | "lastVerifiedAt"> {
    if (!this.isRecord(input)) {
      throw this.invalidManifest("request body must be an object");
    }

    const name = this.assertNonEmptyString(input.name, "name");
    const source = this.assertNonEmptyString(input.source, "source");
    const sourceType = this.assertEnum(input.sourceType, SOURCE_TYPES, "sourceType");
    const chain = this.assertEnum(input.chain, CHAINS, "chain");
    const network = this.assertEnum(input.network, NETWORKS, "network");
    const nodeType = this.assertEnum(input.nodeType, NODE_TYPES, "nodeType");
    const storageEngine = this.assertEnum(input.storageEngine, STORAGE_ENGINES, "storageEngine");
    this.assertChainCompatibility(chain, network, nodeType);
    const height = this.assertPositiveInteger(input.height, "height");
    const sha256 = this.assertSha256(input.sha256);
    const blockHash = this.optionalString(input.blockHash, "blockHash");
    const signature = this.optionalString(input.signature, "signature");
    const sizeBytes = input.sizeBytes === undefined || input.sizeBytes === null
      ? undefined
      : this.assertNonNegativeInteger(input.sizeBytes, "sizeBytes");
    if (input.trusted === true) {
      throw this.invalidManifest("trusted is derived by server verification and cannot be set by clients");
    }

    return {
      name,
      sourceType,
      source,
      chain,
      network,
      nodeType,
      storageEngine,
      height,
      blockHash,
      sha256,
      sizeBytes,
      signature,
      trusted: false,
    };
  }

  private assertNonEmptyString(value: unknown, field: string): string {
    if (typeof value !== "string" || value.trim() === "") {
      throw this.invalidManifest(`${field} is required`);
    }
    return value.trim();
  }

  private optionalString(value: unknown, field: string): string | undefined {
    if (value === undefined || value === null) {
      return undefined;
    }
    if (typeof value !== "string") {
      throw this.invalidManifest(`${field} must be a string`);
    }
    return value;
  }

  private assertPositiveInteger(value: unknown, field: string): number {
    if (typeof value !== "number" || !Number.isInteger(value) || value <= 0) {
      throw this.invalidManifest(`${field} must be a positive integer`);
    }
    return value;
  }

  private assertNonNegativeInteger(value: unknown, field: string): number {
    if (typeof value !== "number" || !Number.isInteger(value) || value < 0) {
      throw this.invalidManifest(`${field} must be a non-negative integer`);
    }
    return value;
  }

  private assertSha256(value: unknown): string {
    if (typeof value !== "string" || !SHA256_PATTERN.test(value)) {
      throw this.invalidManifest("sha256 must be a 64-character hex string");
    }
    return value.toLowerCase();
  }

  private assertEnum<T extends string>(value: unknown, allowed: readonly T[], field: string): T {
    if (typeof value === "string" && allowed.includes(value as T)) {
      return value as T;
    }
    throw this.invalidManifest(`${field} must be one of: ${allowed.join(", ")}`);
  }

  private assertChainCompatibility(chain: NodeChain, network: NodeNetwork, nodeType: NodeType): void {
    if (chain === "n3") {
      if (!N3_NETWORKS.has(network)) {
        throw this.invalidManifest(`network ${network} is not valid for chain n3`);
      }
      if (!N3_NODE_TYPES.has(nodeType)) {
        throw this.invalidManifest(`nodeType ${nodeType} is not valid for chain n3`);
      }
      return;
    }

    if (!X_NETWORKS.has(network)) {
      throw this.invalidManifest(`network ${network} is not valid for chain x`);
    }
    if (nodeType !== "neox-go") {
      throw this.invalidManifest(`nodeType ${nodeType} is not valid for chain x`);
    }
  }

  private async hashLocalFile(filePath: string): Promise<{ sha256: string; sizeBytes: number }> {
    let fileHandle: Awaited<ReturnType<typeof open>>;
    try {
      fileHandle = await open(filePath, "r");
    } catch (error) {
      throw this.localFileAccessError(filePath, error);
    }

    try {
      const stats = await fileHandle.stat();
      if (!stats.isFile()) {
        throw new ApiError(
          "FAST_SYNC_SNAPSHOT_FILE_INVALID",
          `Fast sync snapshot source is not a file: ${filePath}`,
          "Use a local snapshot archive file path, not a directory or special file.",
        );
      }

      const sha256 = await new Promise<string>((resolve, reject) => {
        const hash = crypto.createHash("sha256");
        const stream = createReadStream(filePath, { fd: fileHandle.fd, autoClose: false });

        stream.on("data", (chunk) => hash.update(chunk));
        stream.on("error", reject);
        stream.on("end", () => resolve(hash.digest("hex")));
      });

      return { sha256, sizeBytes: stats.size };
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError(
        "FAST_SYNC_SNAPSHOT_FILE_READ_FAILED",
        `Fast sync snapshot file could not be read: ${filePath}`,
        "Check file permissions and retry verification.",
      );
    } finally {
      await fileHandle.close().catch(() => undefined);
    }
  }

  private localFileAccessError(filePath: string, error: unknown): ApiError {
    const code = typeof error === "object" && error !== null && "code" in error
      ? String((error as { code?: unknown }).code)
      : "";
    if (code === "ENOENT" || code === "ENOTDIR") {
      return new ApiError(
        "FAST_SYNC_SNAPSHOT_FILE_NOT_FOUND",
        `Fast sync snapshot file not found: ${filePath}`,
        "Check that the local snapshot file exists and is readable by NeoNexus.",
        404,
      );
    }
    return new ApiError(
      "FAST_SYNC_SNAPSHOT_FILE_NOT_READABLE",
      `Fast sync snapshot file is not readable: ${filePath}`,
      "Check file permissions and retry verification.",
    );
  }

  private invalidManifest(message: string): ApiError {
    return new ApiError(
      "FAST_SYNC_SNAPSHOT_INVALID",
      `Invalid fast sync snapshot manifest: ${message}`,
      FAST_SYNC_INVALID_SUGGESTION,
    );
  }

  private isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null && !Array.isArray(value);
  }

  private mapRow(row: FastSyncSnapshotRow): FastSyncSnapshot {
    return {
      id: row.id,
      name: row.name,
      sourceType: row.source_type as FastSyncSourceType,
      source: row.source,
      chain: row.chain as NodeChain,
      network: row.network as NodeNetwork,
      nodeType: row.node_type as NodeType,
      storageEngine: row.storage_engine as StorageEngine,
      height: row.height,
      blockHash: row.block_hash ?? undefined,
      sha256: row.sha256,
      sizeBytes: row.size_bytes ?? undefined,
      signature: row.signature ?? undefined,
      trusted: row.trusted === 1,
      createdAt: row.created_at,
      lastVerifiedAt: row.last_verified_at ?? undefined,
    };
  }
}
