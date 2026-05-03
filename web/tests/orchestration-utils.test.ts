import { describe, expect, it } from "vitest";
import type { FastSyncSnapshot, NodeRoleProfile } from "../src/hooks/useNodeOrchestration";
import type { Node } from "../src/hooks/useNodes";
import {
  compatibleSnapshots,
  currentStorageEngine,
  currentSyncStrategy,
  defaultPrivateNetworkName,
  formatStorageEngine,
  privateNetworkTemplateNodeCount,
  roleSupportsNode,
  summarizeRole,
} from "../src/utils/orchestration";

const node: Node = {
  id: "node-1",
  name: "Node 1",
  chain: "n3",
  type: "neo-cli",
  network: "mainnet",
  syncMode: "full",
  version: "3.9.2",
  ports: { rpc: 10332, p2p: 10333 },
  process: { status: "stopped" },
  settings: {
    storageEngine: "rocksdb",
    syncStrategy: "fast-sync",
  },
};

describe("orchestration helpers", () => {
  it("summarizes role presets with storage, sync, plugins, and data context", () => {
    const role: NodeRoleProfile = {
      id: "builtin-state",
      name: "State Node",
      kind: "builtin",
      nodeTypes: ["neo-cli"],
      profile: {
        storageEngine: "rocksdb",
        plugins: [{ id: "StateService", enabled: true }],
        dataContext: { mode: "reuse-or-create", labelTemplate: "state-{network}-{storageEngine}" },
        sync: { strategy: "fast-sync" },
      },
      createdAt: 0,
      updatedAt: 0,
    };

    expect(roleSupportsNode(role, "neo-cli")).toBe(true);
    expect(roleSupportsNode(role, "neo-go")).toBe(false);
    expect(summarizeRole(role)).toBe("RocksDB · Fast sync · 1 plugin · isolated context");
  });

  it("falls back to node settings for storage and sync strategy", () => {
    expect(currentStorageEngine(node)).toBe("rocksdb");
    expect(currentSyncStrategy(node)).toBe("fast-sync");
    expect(formatStorageEngine("leveldb")).toBe("LevelDB");
  });

  it("filters fast sync snapshots by chain, network, type, and storage", () => {
    const snapshots: FastSyncSnapshot[] = [
      {
        id: "snapshot-1",
        name: "mainnet rocks",
        sourceType: "local",
        source: "/tmp/snapshot.tar.zst",
        chain: "n3",
        network: "mainnet",
        nodeType: "neo-cli",
        storageEngine: "rocksdb",
        height: 1,
        sha256: "a".repeat(64),
        trusted: false,
        createdAt: 1,
      },
      {
        id: "snapshot-2",
        name: "mainnet level",
        sourceType: "local",
        source: "/tmp/snapshot-level.tar.zst",
        chain: "n3",
        network: "mainnet",
        nodeType: "neo-cli",
        storageEngine: "leveldb",
        height: 1,
        sha256: "b".repeat(64),
        trusted: false,
        createdAt: 1,
      },
    ];

    expect(compatibleSnapshots(snapshots, node).map((snapshot) => snapshot.id)).toEqual(["snapshot-1"]);
  });

  it("labels private network templates consistently", () => {
    expect(privateNetworkTemplateNodeCount("seven")).toBe(7);
    expect(defaultPrivateNetworkName("four")).toBe("Local 4-node private network");
  });
});
