import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";
import { DownloadManager } from "../../src/core/DownloadManager";
import { NodeManager } from "../../src/core/NodeManager";
import { StorageManager } from "../../src/core/StorageManager";

describe("NodeManager plugin mutations", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("blocks plugin configuration updates while a node is running", () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: { updatePluginConfig: ReturnType<typeof vi.fn> };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "running" },
      settings: {},
    }));
    manager.pluginManager = { updatePluginConfig: vi.fn() };

    expect(() => manager.updatePluginConfig("node-1", "RpcServer", { Port: 10332 })).toThrow(/running/i);
    expect(manager.pluginManager.updatePluginConfig).not.toHaveBeenCalled();
  });

  it("rewrites node config with only enabled plugins after enablement changes", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {},
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        setPluginEnabled: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.pluginManager = {
      setPluginEnabled: vi.fn(),
      getInstalledPlugins: vi.fn(() => [
        { id: "RpcServer", enabled: true },
        { id: "RestServer", enabled: false },
      ]),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.setPluginEnabled("node-1", "RpcServer", true);

    expect(manager.pluginManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "RpcServer", true);
    expect(writeNodeConfig).toHaveBeenCalledWith(node, ["RpcServer"]);
  });

  it("upserts RocksDBStore when neo-cli storage engine is rocksdb", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "rocksdb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([
          { id: "LevelDBStore", enabled: true },
          { id: "RocksDBStore", enabled: true },
        ])
        .mockReturnValue([
          { id: "LevelDBStore", enabled: false },
          { id: "RocksDBStore", enabled: true },
        ]),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.ensureStorageEngine("node-1", "rocksdb");

    expect(manager.pluginManager.upsertPlugin).toHaveBeenCalledWith("node-1", "RocksDBStore", expect.any(String), {});
    expect(writeNodeConfig).toHaveBeenCalledWith(node, ["RocksDBStore"]);
  });

  it("upserts RocksDBStore before writing config for new neo-cli rocksdb nodes", async () => {
    const savedNodes: Array<{ id: string; settings: { storageEngine?: string } }> = [];
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      portManager: {
        findNextIndex: ReturnType<typeof vi.fn>;
        allocatePorts: ReturnType<typeof vi.fn>;
        releasePorts: ReturnType<typeof vi.fn>;
      };
      repo: {
        transaction: ReturnType<typeof vi.fn>;
        saveNode: ReturnType<typeof vi.fn>;
        deleteNode: ReturnType<typeof vi.fn>;
      };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.portManager = {
      findNextIndex: vi.fn().mockResolvedValue(1),
      allocatePorts: vi.fn().mockResolvedValue({ rpc: 10332, p2p: 10333, websocket: 10334, metrics: 10335 }),
      releasePorts: vi.fn(),
    };
    manager.repo = {
      transaction: vi.fn((callback: () => void) => callback()),
      saveNode: vi.fn((config) => savedNodes.push(config)),
      deleteNode: vi.fn(),
    };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([
          { id: "LevelDBStore", enabled: true },
          { id: "RocksDBStore", enabled: true },
        ])
        .mockReturnValue([
          { id: "LevelDBStore", enabled: false },
          { id: "RocksDBStore", enabled: true },
        ]),
      setPluginEnabled: vi.fn(),
    };
    manager.getNode = vi.fn((nodeId: string) => ({
      ...savedNodes.find((node) => node.id === nodeId),
      id: nodeId,
      type: "neo-cli",
      version: "v3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "rocksdb" },
    }));
    vi.spyOn(DownloadManager, "hasNodeBinary").mockReturnValue(true);
    vi.spyOn(StorageManager, "ensureNodeDirectories").mockReturnValue(undefined);
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.createNode({
      name: "State node",
      type: "neo-cli",
      network: "mainnet",
      version: "v3.7.5",
      settings: { storageEngine: "rocksdb" },
    });

    expect(manager.pluginManager.upsertPlugin).toHaveBeenCalledWith(expect.any(String), "RocksDBStore", "v3.7.5", {});
    expect(manager.pluginManager.setPluginEnabled).toHaveBeenCalledWith(expect.any(String), "LevelDBStore", false);
    expect(writeNodeConfig).toHaveBeenCalledWith(expect.objectContaining({
      type: "neo-cli",
      settings: expect.objectContaining({ storageEngine: "rocksdb" }),
    }), ["RocksDBStore"]);
  });

  it("releases allocated ports when new neo-cli rocksdb plugin setup fails", async () => {
    const allocatedPorts = { rpc: 10332, p2p: 10333, websocket: 10334, metrics: 10335 };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      portManager: {
        findNextIndex: ReturnType<typeof vi.fn>;
        allocatePorts: ReturnType<typeof vi.fn>;
        releasePorts: ReturnType<typeof vi.fn>;
      };
      repo: {
        transaction: ReturnType<typeof vi.fn>;
        saveNode: ReturnType<typeof vi.fn>;
        deleteNode: ReturnType<typeof vi.fn>;
      };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
      };
    };

    manager.portManager = {
      findNextIndex: vi.fn().mockResolvedValue(1),
      allocatePorts: vi.fn().mockResolvedValue(allocatedPorts),
      releasePorts: vi.fn(),
    };
    manager.repo = {
      transaction: vi.fn((callback: () => void) => callback()),
      saveNode: vi.fn(),
      deleteNode: vi.fn(),
    };
    manager.pluginManager = {
      upsertPlugin: vi.fn().mockRejectedValue(new Error("plugin failed")),
      getInstalledPlugins: vi.fn(() => []),
    };
    vi.spyOn(DownloadManager, "hasNodeBinary").mockReturnValue(true);
    vi.spyOn(StorageManager, "ensureNodeDirectories").mockReturnValue(undefined);
    vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await expect(manager.createNode({
      name: "State node",
      type: "neo-cli",
      network: "mainnet",
      version: "v3.7.5",
      settings: { storageEngine: "rocksdb" },
    })).rejects.toThrow(/plugin failed/i);

    expect(manager.repo.deleteNode).toHaveBeenCalledWith(expect.any(String));
    expect(manager.portManager.releasePorts).toHaveBeenCalledWith(allocatedPorts);
  });

  it("rejects invalid create sync modes before mutating", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager;
    const hasNodeBinary = vi.spyOn(DownloadManager, "hasNodeBinary").mockReturnValue(true);

    await expect(manager.createNode({
      name: "Bad sync",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "turbo" as never,
    })).rejects.toThrow(/sync mode/i);

    expect(hasNodeBinary).not.toHaveBeenCalled();
  });

  it("rejects invalid storage engines before mutating storage or plugins", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "leveldb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi.fn(() => []),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await expect(manager.ensureStorageEngine("node-1", "bad" as never)).rejects.toThrow(/storage engine/i);

    expect(manager.repo.updateNode).not.toHaveBeenCalled();
    expect(manager.pluginManager.getInstalledPlugins).not.toHaveBeenCalled();
    expect(manager.pluginManager.upsertPlugin).not.toHaveBeenCalled();
    expect(manager.pluginManager.setPluginEnabled).not.toHaveBeenCalled();
    expect(writeNodeConfig).not.toHaveBeenCalled();
  });

  it("routes updateNode storage engine changes through RocksDBStore orchestration", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "leveldb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([])
        .mockReturnValue([{ id: "RocksDBStore", enabled: true }]),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.updateNode("node-1", { settings: { storageEngine: "rocksdb" } });

    expect(manager.pluginManager.upsertPlugin).toHaveBeenCalledWith("node-1", "RocksDBStore", expect.any(String), {});
    expect(manager.repo.updateNode).toHaveBeenCalledTimes(1);
    expect(writeNodeConfig).toHaveBeenCalledWith(node, ["RocksDBStore"]);
  });

  it("disables LevelDBStore when switching neo-cli storage to rocksdb", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "leveldb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([
          { id: "LevelDBStore", enabled: true },
          { id: "RocksDBStore", enabled: false },
        ])
        .mockReturnValueOnce([
          { id: "LevelDBStore", enabled: true },
          { id: "RocksDBStore", enabled: true },
        ])
        .mockReturnValue([
          { id: "LevelDBStore", enabled: false },
          { id: "RocksDBStore", enabled: true },
          { id: "RpcServer", enabled: true },
        ]),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.ensureStorageEngine("node-1", "rocksdb");

    expect(manager.pluginManager.upsertPlugin).toHaveBeenCalledWith("node-1", "RocksDBStore", expect.any(String), {});
    expect(manager.pluginManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "LevelDBStore", false);
    expect(writeNodeConfig).toHaveBeenCalledWith(node, ["RocksDBStore", "RpcServer"]);
  });

  it("rejects mixed updateNode storage engine changes before storage orchestration", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "leveldb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi.fn(() => []),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await expect(manager.updateNode("node-1", {
      name: "Renamed",
      settings: { storageEngine: "rocksdb" },
    })).rejects.toThrow(/submitted separately/i);

    expect(manager.repo.updateNode).not.toHaveBeenCalled();
    expect(manager.pluginManager.getInstalledPlugins).not.toHaveBeenCalled();
    expect(manager.pluginManager.upsertPlugin).not.toHaveBeenCalled();
    expect(manager.pluginManager.setPluginEnabled).not.toHaveBeenCalled();
    expect(writeNodeConfig).not.toHaveBeenCalled();
  });

  it("rewrites restored config when storage engine config rewrite fails", async () => {
    const originalNode = {
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      process: { status: "stopped" },
      settings: { storageEngine: "leveldb" },
    };
    const updatedNode = {
      ...originalNode,
      settings: { storageEngine: "rocksdb" },
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      repo: { updateNode: ReturnType<typeof vi.fn> };
      pluginManager: {
        upsertPlugin: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi
      .fn()
      .mockReturnValueOnce(originalNode)
      .mockReturnValueOnce(updatedNode)
      .mockReturnValueOnce(originalNode);
    manager.repo = { updateNode: vi.fn() };
    manager.pluginManager = {
      upsertPlugin: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([{ id: "RocksDBStore", enabled: false, config: {} }])
        .mockReturnValueOnce([{ id: "RocksDBStore", enabled: true, config: {} }])
        .mockReturnValueOnce([{ id: "RocksDBStore", enabled: true, config: {} }])
        .mockReturnValue([{ id: "RocksDBStore", enabled: false, config: {} }]),
      setPluginEnabled: vi.fn(),
    };
    const writeNodeConfig = vi
      .spyOn(ConfigManager, "writeNodeConfig")
      .mockRejectedValueOnce(new Error("disk full"))
      .mockResolvedValueOnce(undefined);

    await expect(manager.ensureStorageEngine("node-1", "rocksdb")).rejects.toThrow(/disk full/i);

    expect(manager.pluginManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "RocksDBStore", false);
    expect(writeNodeConfig).toHaveBeenNthCalledWith(1, updatedNode, ["RocksDBStore"]);
    expect(writeNodeConfig).toHaveBeenNthCalledWith(2, originalNode, []);
  });

  it("rolls back plugin enablement if rewriting node config fails", async () => {
    const node = {
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {},
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        setPluginEnabled: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => node);
    manager.pluginManager = {
      setPluginEnabled: vi.fn(),
      getInstalledPlugins: vi
        .fn()
        .mockReturnValueOnce([{ id: "RpcServer", enabled: false }])
        .mockReturnValueOnce([{ id: "RpcServer", enabled: true }]),
    };
    vi.spyOn(ConfigManager, "writeNodeConfig").mockRejectedValue(new Error("disk full"));

    await expect(manager.setPluginEnabled("node-1", "RpcServer", true)).rejects.toThrow(/disk full/i);

    expect(manager.pluginManager.setPluginEnabled).toHaveBeenNthCalledWith(1, "node-1", "RpcServer", true);
    expect(manager.pluginManager.setPluginEnabled).toHaveBeenNthCalledWith(2, "node-1", "RpcServer", false);
  });

  it("blocks direct SignClient config updates when secure signer protection owns it", () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: { updatePluginConfig: ReturnType<typeof vi.fn> };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = { updatePluginConfig: vi.fn() };

    expect(() => manager.updatePluginConfig("node-1", "SignClient", { Endpoint: "http://127.0.0.1:9991" }))
      .toThrow(/secure signer/i);
    expect(manager.pluginManager.updatePluginConfig).not.toHaveBeenCalled();
  });

  it("blocks disabling SignClient when secure signer protection requires it", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        setPluginEnabled: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = {
      setPluginEnabled: vi.fn(),
      getInstalledPlugins: vi.fn(() => [{ id: "SignClient", enabled: true }]),
    };

    expect(() => manager.setPluginEnabled("node-1", "SignClient", false)).toThrow(/secure signer/i);
    expect(manager.pluginManager.setPluginEnabled).not.toHaveBeenCalled();
  });

  it("blocks uninstalling SignClient when secure signer protection requires it", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: { uninstallPlugin: ReturnType<typeof vi.fn> };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = { uninstallPlugin: vi.fn() };

    await expect(manager.uninstallPlugin("node-1", "SignClient")).rejects.toMatchObject({
      code: "SIGNER_PLUGIN_REQUIRED",
    });
    expect(manager.pluginManager.uninstallPlugin).not.toHaveBeenCalled();
  });
});
