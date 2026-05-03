import crypto from "node:crypto";
import type Database from "better-sqlite3";
import { ApiError } from "../api/errors";
import type {
  ConfigurationSnapshot,
  N3NodeType,
  PluginId,
  PortConfig,
  PrivateNetworkPlan,
  PrivateNetworkPlanStatus,
  PrivateNetworkTemplate,
  StorageEngine,
} from "../types";
import type { PrivateNetworkPlanRow } from "../types/database";

const TEMPLATES: Record<PrivateNetworkTemplate, number> = {
  single: 1,
  four: 4,
  seven: 7,
};
const NODE_TYPES = ["neo-cli", "neo-go"] as const satisfies readonly N3NodeType[];
const STORAGE_ENGINES = ["leveldb", "rocksdb"] as const satisfies readonly StorageEngine[];
const DEFAULT_RPC_PORT = 20332;
const DEFAULT_P2P_PORT = 20333;
const DEFAULT_WEBSOCKET_PORT = 20334;
const DEFAULT_METRICS_PORT = 22112;
const PORT_OFFSET = 10;
const MAX_UINT32 = 0xffffffff;
const ADDRESS_VERSION = 0x35;
const BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const INVALID_SUGGESTION = "Provide a non-empty name, template single/four/seven, N3 node type, storage engine, positive network magic, and valid local base ports.";

export interface CreatePrivateNetworkPlanInput {
  name?: unknown;
  template?: unknown;
  nodeType?: unknown;
  storageEngine?: unknown;
  networkMagic?: unknown;
  baseRpcPort?: unknown;
  baseP2pPort?: unknown;
  baseWebsocketPort?: unknown;
  baseMetricsPort?: unknown;
  nodeNamePrefix?: unknown;
}

interface PrivateNetworkNodePlan {
  name: string;
  type: N3NodeType;
  roleIds: string[];
  storageEngine: StorageEngine;
  ports: Partial<PortConfig>;
  publicKey: string;
  address: string;
}

export function n3AddressFromPublicKey(publicKey: string): string {
  if (!/^(02|03)[0-9a-f]{64}$/i.test(publicKey)) {
    throw invalidPlan("publicKey must be a compressed 33-byte secp256r1 key");
  }

  const script = Buffer.concat([
    Buffer.from([0x0c, 0x21]),
    Buffer.from(publicKey, "hex"),
    Buffer.from("4156e7b327", "hex"),
  ]);
  const scriptHash = crypto.createHash("ripemd160").update(
    crypto.createHash("sha256").update(script).digest(),
  ).digest();
  return base58Check(Buffer.concat([Buffer.from([ADDRESS_VERSION]), scriptHash]));
}

export class PrivateNetworkManager {
  constructor(private readonly db: Database.Database) {}

  listPlans(): PrivateNetworkPlan[] {
    const rows = this.db
      .prepare("SELECT * FROM private_network_plans ORDER BY created_at DESC")
      .all() as PrivateNetworkPlanRow[];
    return rows.map((row) => this.mapRow(row));
  }

  getPlan(id: string, options?: { required?: false }): PrivateNetworkPlan | null;
  getPlan(id: string, options: { required: true }): PrivateNetworkPlan;
  getPlan(id: string, options: { required?: boolean } = {}): PrivateNetworkPlan | null {
    const row = this.db
      .prepare("SELECT * FROM private_network_plans WHERE id = ?")
      .get(id) as PrivateNetworkPlanRow | undefined;
    if (!row) {
      if (options.required) {
        throw notFound(id);
      }
      return null;
    }
    return this.mapRow(row);
  }

  createPlan(input: CreatePrivateNetworkPlanInput): PrivateNetworkPlan {
    if (!isRecord(input)) {
      throw invalidPlan("request body must be an object");
    }

    const name = this.nonEmptyString(input.name, "name");
    const template = this.enumValue(input.template, Object.keys(TEMPLATES) as PrivateNetworkTemplate[], "template");
    const nodeType = input.nodeType === undefined || input.nodeType === null
      ? "neo-cli"
      : this.enumValue(input.nodeType, NODE_TYPES, "nodeType");
    const storageEngine = input.storageEngine === undefined || input.storageEngine === null
      ? "leveldb"
      : this.enumValue(input.storageEngine, STORAGE_ENGINES, "storageEngine");
    const networkMagic = input.networkMagic === undefined || input.networkMagic === null
      ? this.randomNetworkMagic()
      : this.networkMagic(input.networkMagic);
    const baseRpcPort = input.baseRpcPort === undefined || input.baseRpcPort === null
      ? DEFAULT_RPC_PORT
      : this.positiveInteger(input.baseRpcPort, "baseRpcPort");
    const baseP2pPort = input.baseP2pPort === undefined || input.baseP2pPort === null
      ? DEFAULT_P2P_PORT
      : this.positiveInteger(input.baseP2pPort, "baseP2pPort");
    const baseWebsocketPort = input.baseWebsocketPort === undefined || input.baseWebsocketPort === null
      ? DEFAULT_WEBSOCKET_PORT
      : this.positiveInteger(input.baseWebsocketPort, "baseWebsocketPort");
    const baseMetricsPort = input.baseMetricsPort === undefined || input.baseMetricsPort === null
      ? DEFAULT_METRICS_PORT
      : this.positiveInteger(input.baseMetricsPort, "baseMetricsPort");
    const nodeNamePrefix = input.nodeNamePrefix === undefined || input.nodeNamePrefix === null
      ? slugifyName(name)
      : slugifyName(this.nonEmptyString(input.nodeNamePrefix, "nodeNamePrefix"));
    const nodeCount = TEMPLATES[template];

    this.assertGeneratedPortsAreSafe({
      baseRpcPort,
      baseP2pPort,
      baseWebsocketPort,
      baseMetricsPort,
      nodeCount,
    });

    const nodes: PrivateNetworkNodePlan[] = Array.from({ length: nodeCount }, (_, index) => {
      const publicKey = this.generateCompressedPublicKey();
      return {
        name: `${nodeNamePrefix}-${index + 1}`,
        type: nodeType,
        roleIds: index === 0 ? ["builtin-consensus", "builtin-rpc-api"] : ["builtin-consensus"],
        storageEngine,
        ports: {
          rpc: baseRpcPort + index * PORT_OFFSET,
          p2p: baseP2pPort + index * PORT_OFFSET,
          websocket: baseWebsocketPort + index * PORT_OFFSET,
          metrics: baseMetricsPort + index * PORT_OFFSET,
        },
        publicKey,
        address: n3AddressFromPublicKey(publicKey),
      };
    });

    const now = Date.now();
    const plan: PrivateNetworkPlan = {
      id: `private-plan-${crypto.randomUUID()}`,
      name,
      template,
      networkMagic,
      plan: {
        nodes,
        seedList: nodes.map((node) => `127.0.0.1:${node.ports.p2p}`),
        validatorsCount: nodeCount,
        standbyCommittee: nodes.map((node) => node.publicKey),
      },
      status: "draft",
      createdAt: now,
    };

    this.db.prepare(`
      INSERT INTO private_network_plans (id, name, template, network_magic, plan, status, created_at, applied_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      plan.id,
      plan.name,
      plan.template,
      plan.networkMagic,
      JSON.stringify(plan.plan),
      plan.status,
      plan.createdAt,
      null,
    );

    return plan;
  }

  buildConfigurationSnapshot(planId: string): ConfigurationSnapshot {
    const plan = this.getPlan(planId, { required: true });

    return {
      generatedAt: Date.now(),
      version: "private-network-plan-v1",
      nodes: plan.plan.nodes.map((node) => ({
        name: node.name,
        type: node.type,
        network: "private",
        syncMode: "full",
        ports: node.ports,
        settings: {
          storageEngine: node.storageEngine,
          syncStrategy: "full",
          customConfig: {
            privateNetwork: {
              networkMagic: plan.networkMagic,
              validatorsCount: plan.plan.validatorsCount,
              standbyCommittee: plan.plan.standbyCommittee,
              seedList: plan.plan.seedList,
              publicKey: node.publicKey,
              address: node.address,
            },
          },
        },
        plugins: node.type === "neo-cli" ? this.pluginsForRoles(node.roleIds) : undefined,
      })),
    };
  }

  markApplied(planId: string): PrivateNetworkPlan {
    const plan = this.getPlan(planId, { required: true });
    const appliedAt = Date.now();
    this.db
      .prepare("UPDATE private_network_plans SET status = ?, applied_at = ? WHERE id = ?")
      .run("applied", appliedAt, planId);
    return {
      ...plan,
      status: "applied",
      appliedAt,
    };
  }

  private mapRow(row: PrivateNetworkPlanRow): PrivateNetworkPlan {
    const plan = this.parseStoredPlan(row.id, row.plan);
    return {
      id: row.id,
      name: row.name,
      template: this.rowEnum(row.template, Object.keys(TEMPLATES) as PrivateNetworkTemplate[], "template"),
      networkMagic: row.network_magic,
      plan,
      status: this.rowEnum(row.status, ["draft", "applied", "failed"] as PrivateNetworkPlanStatus[], "status"),
      createdAt: row.created_at,
      appliedAt: row.applied_at ?? undefined,
    };
  }

  private pluginsForRoles(roleIds: string[]) {
    const pluginIds: PluginId[] = [];
    if (roleIds.includes("builtin-consensus") || roleIds.includes("consensus")) {
      pluginIds.push("DBFTPlugin");
    }
    if (roleIds.includes("builtin-rpc-api") || roleIds.includes("rpc")) {
      pluginIds.push("RpcServer");
    }
    if (roleIds.includes("builtin-state") || roleIds.includes("state")) {
      pluginIds.push("StateService");
    }
    if (roleIds.includes("builtin-oracle") || roleIds.includes("oracle")) {
      pluginIds.push("OracleService");
    }
    return pluginIds.map((id) => ({ id, enabled: true }));
  }

  private generateCompressedPublicKey(): string {
    const ecdh = crypto.createECDH("prime256v1");
    ecdh.generateKeys();
    return ecdh.getPublicKey(undefined, "compressed").toString("hex");
  }

  private randomNetworkMagic(): number {
    let magic = 0;
    while (magic === 0) {
      magic = crypto.randomInt(1, 0x7fffffff);
    }
    return magic;
  }

  private nonEmptyString(value: unknown, field: string): string {
    if (typeof value !== "string" || value.trim() === "") {
      throw invalidPlan(`${field} is required`);
    }
    return value.trim();
  }

  private positiveInteger(value: unknown, field: string): number {
    if (typeof value !== "number" || !Number.isInteger(value) || value <= 0) {
      throw invalidPlan(`${field} must be a positive integer`);
    }
    return value;
  }

  private networkMagic(value: unknown): number {
    const magic = this.positiveInteger(value, "networkMagic");
    if (magic > MAX_UINT32) {
      throw invalidPlan("networkMagic must fit in an unsigned 32-bit integer");
    }
    return magic;
  }

  private enumValue<T extends string>(value: unknown, allowed: readonly T[], field: string): T {
    if (typeof value !== "string" || !allowed.includes(value as T)) {
      throw invalidPlan(`${field} must be one of: ${allowed.join(", ")}`);
    }
    return value as T;
  }

  private rowEnum<T extends string>(value: string, allowed: readonly T[], field: string): T {
    if (!allowed.includes(value as T)) {
      throw new Error(`Invalid stored private network ${field}: ${value}`);
    }
    return value as T;
  }

  private assertGeneratedPortsAreSafe(input: {
    baseRpcPort: number;
    baseP2pPort: number;
    baseWebsocketPort: number;
    baseMetricsPort: number;
    nodeCount: number;
  }): void {
    const seen = new Map<number, string>();
    const bases = [
      ["rpc", input.baseRpcPort],
      ["p2p", input.baseP2pPort],
      ["websocket", input.baseWebsocketPort],
      ["metrics", input.baseMetricsPort],
    ] as const;

    for (const [kind, basePort] of bases) {
      for (let index = 0; index < input.nodeCount; index += 1) {
        const port = basePort + index * PORT_OFFSET;
        const label = `${kind} port for node ${index + 1}`;
        if (port > 65535) {
          throw invalidPlan(`${label} must stay within TCP port range`);
        }
        const existing = seen.get(port);
        if (existing) {
          throw invalidPlan(`${label} ${port} conflicts with ${existing}`);
        }
        seen.set(port, label);
      }
    }
  }

  private parseStoredPlan(planId: string, rawPlan: string): PrivateNetworkPlan["plan"] {
    let parsed: unknown;
    try {
      parsed = JSON.parse(rawPlan);
    } catch {
      throw corruptPlan(planId);
    }

    if (!isRecord(parsed) || !Array.isArray(parsed.nodes) || !Array.isArray(parsed.seedList) || !Array.isArray(parsed.standbyCommittee)) {
      throw corruptPlan(planId);
    }

    const validatorsCount = parsed.validatorsCount;
    if (typeof validatorsCount !== "number" || !Number.isInteger(validatorsCount) || validatorsCount <= 0) {
      throw corruptPlan(planId);
    }

    const nodes = parsed.nodes.map((node) => this.parseStoredNode(planId, node));
    if (
      nodes.length !== validatorsCount ||
      !parsed.seedList.every((seed) => typeof seed === "string") ||
      !parsed.standbyCommittee.every((key) => typeof key === "string")
    ) {
      throw corruptPlan(planId);
    }

    return {
      nodes,
      seedList: [...parsed.seedList],
      validatorsCount,
      standbyCommittee: [...parsed.standbyCommittee],
    };
  }

  private parseStoredNode(planId: string, value: unknown): PrivateNetworkPlan["plan"]["nodes"][number] {
    if (!isRecord(value) || !isRecord(value.ports)) {
      throw corruptPlan(planId);
    }
    const ports = value.ports;
    if (
      typeof value.name !== "string" ||
      !NODE_TYPES.includes(value.type as N3NodeType) ||
      !Array.isArray(value.roleIds) ||
      !value.roleIds.every((roleId) => typeof roleId === "string") ||
      !STORAGE_ENGINES.includes(value.storageEngine as StorageEngine) ||
      !isOptionalPort(ports.rpc) ||
      !isOptionalPort(ports.p2p) ||
      !isOptionalPort(ports.websocket) ||
      !isOptionalPort(ports.metrics) ||
      typeof value.publicKey !== "string" ||
      typeof value.address !== "string"
    ) {
      throw corruptPlan(planId);
    }
    return {
      name: value.name,
      type: value.type as N3NodeType,
      roleIds: [...value.roleIds],
      storageEngine: value.storageEngine as StorageEngine,
      ports: {
        ...(ports.rpc !== undefined ? { rpc: ports.rpc } : {}),
        ...(ports.p2p !== undefined ? { p2p: ports.p2p } : {}),
        ...(ports.websocket !== undefined ? { websocket: ports.websocket } : {}),
        ...(ports.metrics !== undefined ? { metrics: ports.metrics } : {}),
      },
      publicKey: value.publicKey,
      address: value.address,
    };
  }
}

function base58Check(payload: Buffer): string {
  const checksum = crypto.createHash("sha256").update(
    crypto.createHash("sha256").update(payload).digest(),
  ).digest().subarray(0, 4);
  return base58Encode(Buffer.concat([payload, checksum]));
}

function base58Encode(bytes: Buffer): string {
  let value = BigInt(`0x${bytes.toString("hex")}`);
  let encoded = "";
  while (value > 0n) {
    const remainder = Number(value % 58n);
    encoded = BASE58_ALPHABET[remainder] + encoded;
    value /= 58n;
  }
  for (const byte of bytes) {
    if (byte !== 0) break;
    encoded = `${BASE58_ALPHABET[0]}${encoded}`;
  }
  return encoded || BASE58_ALPHABET[0];
}

function slugifyName(value: string): string {
  const slug = value.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  return slug || "private-node";
}

function invalidPlan(message: string): ApiError {
  return new ApiError("PRIVATE_NETWORK_PLAN_INVALID", message, INVALID_SUGGESTION);
}

function notFound(id: string): ApiError {
  return new ApiError(
    "PRIVATE_NETWORK_PLAN_NOT_FOUND",
    `Private network plan ${id} not found`,
    "Create the private network plan first, then retry the operation.",
    404,
  );
}

function corruptPlan(id: string): ApiError {
  return new ApiError(
    "PRIVATE_NETWORK_PLAN_CORRUPT",
    `Private network plan ${id} is corrupt`,
    "Delete and recreate the private network plan before applying it.",
    500,
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isOptionalPort(value: unknown): value is number | undefined {
  return value === undefined || (typeof value === "number" && Number.isInteger(value) && value > 0 && value <= 65535);
}
