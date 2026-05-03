import { describe, expect, it, vi } from "vitest";
import Database from "better-sqlite3";
import { ApiError } from "../../src/api/errors";
import { PrivateNetworkManager, n3AddressFromPublicKey } from "../../src/core/PrivateNetworkManager";

vi.unmock("better-sqlite3");

function createDb() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE private_network_plans (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      template TEXT NOT NULL CHECK (template IN ('single', 'four', 'seven')),
      network_magic INTEGER NOT NULL,
      plan TEXT NOT NULL,
      status TEXT NOT NULL CHECK (status IN ('draft', 'applied', 'failed')),
      created_at INTEGER NOT NULL,
      applied_at INTEGER
    );
  `);
  return db;
}

function createManager() {
  return new PrivateNetworkManager(createDb());
}

describe("PrivateNetworkManager", () => {
  it("derives the official Neo N3 address for a compressed public key", () => {
    expect(n3AddressFromPublicKey("03cdb067d930fd5adaa6c68545016044aaddec64ba39e548250eaea551172e535c"))
      .toBe("NNLi44dJNXtDNSBkofB48aTVYtb1zZrNEs");
  });

  it.each([
    ["single", 1],
    ["four", 4],
    ["seven", 7],
  ] as const)("creates a %s-node plan with deterministic port offsets", (template, nodeCount) => {
    const manager = createManager();
    const plan = manager.createPlan({
      name: `${template} private`,
      template,
      networkMagic: 123456,
      baseRpcPort: 30332,
      baseP2pPort: 30333,
      baseWebsocketPort: 30334,
      baseMetricsPort: 32112,
      nodeNamePrefix: "validator",
    });

    expect(plan.status).toBe("draft");
    expect(plan.plan.nodes).toHaveLength(nodeCount);
    expect(plan.plan.validatorsCount).toBe(nodeCount);
    expect(plan.plan.standbyCommittee).toEqual(plan.plan.nodes.map((node) => node.publicKey));
    expect(plan.plan.seedList).toEqual(plan.plan.nodes.map((_, index) => `127.0.0.1:${30333 + index * 10}`));

    plan.plan.nodes.forEach((node, index) => {
      expect(node.name).toBe(`validator-${index + 1}`);
      expect(node.type).toBe("neo-cli");
      expect(node.storageEngine).toBe("leveldb");
      expect(node.ports).toEqual({
        rpc: 30332 + index * 10,
        p2p: 30333 + index * 10,
        websocket: 30334 + index * 10,
        metrics: 32112 + index * 10,
      });
      expect(node.publicKey).toMatch(/^(02|03)[0-9a-f]{64}$/);
      expect(node.address).toMatch(/^N[1-9A-HJ-NP-Za-km-z]{33}$/);
      expect(node.roleIds).toContain("builtin-consensus");
    });
  });

  it("defaults optional fields and rejects unsupported plan input", () => {
    const manager = createManager();
    const plan = manager.createPlan({ name: "Local Chain", template: "single" });

    expect(plan.networkMagic).toBeGreaterThan(0);
    expect(plan.plan.nodes[0].name).toBe("local-chain-1");
    expect(plan.plan.nodes[0].ports).toMatchObject({
      rpc: 20332,
      p2p: 20333,
      websocket: 20334,
      metrics: 22112,
    });

    for (const input of [
      { name: "", template: "single" },
      { name: "bad", template: "three" },
      { name: "bad", template: "single", nodeType: "neox-go" },
      { name: "bad", template: "single", storageEngine: "sqlite" },
      { name: "bad", template: "single", networkMagic: 0 },
      { name: "bad", template: "single", networkMagic: 0x1_0000_0000 },
      { name: "bad", template: "single", baseRpcPort: -1 },
      { name: "bad", template: "four", baseRpcPort: 30332, baseP2pPort: 30342 },
    ]) {
      expect(() => manager.createPlan(input)).toThrow(ApiError);
    }
  });

  it("persists, lists, gets, and marks plans applied", () => {
    const manager = createManager();
    const plan = manager.createPlan({ name: "Four Local", template: "four", networkMagic: 424242 });

    expect(manager.listPlans()).toHaveLength(1);
    expect(manager.getPlan(plan.id)?.networkMagic).toBe(424242);

    const applied = manager.markApplied(plan.id);
    expect(applied.status).toBe("applied");
    expect(applied.appliedAt).toEqual(expect.any(Number));
    expect(manager.getPlan(plan.id)?.status).toBe("applied");

    expect(() => manager.getPlan("missing", { required: true })).toThrow(ApiError);
    expect(() => manager.markApplied("missing")).toThrow(ApiError);
  });

  it("builds a restorable configuration snapshot with private network settings and plugins", () => {
    const manager = createManager();
    const plan = manager.createPlan({
      name: "Seven Local",
      template: "seven",
      networkMagic: 7654321,
      storageEngine: "rocksdb",
      baseRpcPort: 40332,
      baseP2pPort: 40333,
      baseWebsocketPort: 40334,
      baseMetricsPort: 42112,
    });

    const snapshot = manager.buildConfigurationSnapshot(plan.id);

    expect(snapshot.version).toBe("private-network-plan-v1");
    expect(snapshot.nodes).toHaveLength(7);
    expect(snapshot.nodes[0]).toMatchObject({
      name: "seven-local-1",
      type: "neo-cli",
      network: "private",
      syncMode: "full",
      ports: { rpc: 40332, p2p: 40333, websocket: 40334, metrics: 42112 },
      settings: {
        storageEngine: "rocksdb",
        syncStrategy: "full",
        customConfig: {
          privateNetwork: {
            networkMagic: 7654321,
            validatorsCount: 7,
            standbyCommittee: plan.plan.standbyCommittee,
            seedList: plan.plan.seedList,
            publicKey: plan.plan.nodes[0].publicKey,
            address: plan.plan.nodes[0].address,
          },
        },
      },
    });
    expect(snapshot.nodes[0].plugins?.map((plugin) => plugin.id)).toEqual(["DBFTPlugin", "RpcServer"]);
    expect(snapshot.nodes[1].plugins?.map((plugin) => plugin.id)).toEqual(["DBFTPlugin"]);
  });

  it("reports corrupt stored plan JSON with a structured error", () => {
    const db = createDb();
    db.prepare(`
      INSERT INTO private_network_plans (id, name, template, network_magic, plan, status, created_at, applied_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `).run("plan-corrupt", "Corrupt", "single", 123, "{not-json", "draft", 1, null);
    const manager = new PrivateNetworkManager(db);

    expect(() => manager.getPlan("plan-corrupt", { required: true })).toThrow(ApiError);
    try {
      manager.getPlan("plan-corrupt", { required: true });
    } catch (error) {
      expect(error).toMatchObject({
        code: "PRIVATE_NETWORK_PLAN_CORRUPT",
        status: 500,
      });
    }
  });
});
