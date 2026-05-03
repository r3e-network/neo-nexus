import { describe, expect, it } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ConfigManager } from "../../src/core/ConfigManager";
import type { NodeConfig } from "../../src/types";

function createNeoCliNode(baseDir: string): NodeConfig {
  return {
    id: "node-secure-signer",
    name: "Secure Signer Node",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.7.5",
    ports: {
      rpc: 10332,
      p2p: 10333,
    },
    paths: {
      base: baseDir,
      data: path.join(baseDir, "Data"),
      logs: path.join(baseDir, "Logs"),
      config: path.join(baseDir, "config"),
      wallet: path.join(baseDir, "wallets"),
    },
    settings: {},
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
}

describe("secure signer config generation", () => {
  it("keeps the historical neo-cli Data path without an active data context", async () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-default-storage"));

    const config = await ConfigManager.generateNeoCliConfig(node);

    expect((config.ApplicationConfiguration as any).Storage).toEqual({
      Engine: "LevelDBStore",
      Path: "Data",
    });
  });

  it("writes neo-cli storage path for the active data context", async () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-storage"));
    node.settings.activeDataContextId = "ctx-state";
    node.settings.storageEngine = "rocksdb";

    const config = await ConfigManager.generateNeoCliConfig(node, ["RocksDBStore"]);

    expect((config.ApplicationConfiguration as any).Storage).toEqual({
      Engine: "RocksDBStore",
      Path: "data-contexts/ctx-state",
    });
  });

  it("does not write RocksDBStore when the rocksdb plugin is not installed", async () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-missing-rocksdb"));
    node.settings.activeDataContextId = "ctx-state";
    node.settings.storageEngine = "rocksdb";

    const config = await ConfigManager.generateNeoCliConfig(node);

    expect((config.ApplicationConfiguration as any).Storage).toEqual({
      Engine: "LevelDBStore",
      Path: "data-contexts/ctx-state",
    });
  });

  it("keeps explicit leveldb storage even when RocksDBStore is installed", async () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-leveldb"));
    node.settings.storageEngine = "leveldb";

    const config = await ConfigManager.generateNeoCliConfig(node, ["RocksDBStore"]);

    expect((config.ApplicationConfiguration as any).Storage.Engine).toBe("LevelDBStore");
  });

  it.each(["../ctx", "ctx/state", "ctx\\state", ""])("rejects unsafe neo-cli data context id %j", async (activeDataContextId) => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-unsafe"));
    node.settings.activeDataContextId = activeDataContextId;

    await expect(ConfigManager.generateNeoCliConfig(node)).rejects.toThrow(/Invalid data context id/);
  });

  it("generates SignClient plugin config from secure signer settings", () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-generate"));

    const config = (ConfigManager as any).generatePluginConfig("SignClient", node, {
      Name: "Nitro Council Signer",
      Endpoint: "vsock://2345:9991",
    });

    expect(config).toEqual({
      PluginConfiguration: {
        Name: "Nitro Council Signer",
        Endpoint: "vsock://2345:9991",
      },
    });
  });

  it("writes SignClient.json with the stored secure signer endpoint", () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-signclient-write-"));
    const node = createNeoCliNode(baseDir);

    (ConfigManager as any).writePluginConfig("SignClient", node, {
      Name: "SGX Signer",
      Endpoint: "https://signer.example.com:9443",
    });

    const written = JSON.parse(fs.readFileSync(path.join(baseDir, "Plugins", "SignClient", "SignClient.json"), "utf8"));
    expect(written).toEqual({
      PluginConfiguration: {
        Name: "SGX Signer",
        Endpoint: "https://signer.example.com:9443",
      },
    });
  });
});
