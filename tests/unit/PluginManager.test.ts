import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../src/core/DownloadManager", () => ({
  DownloadManager: {
    downloadPlugin: vi.fn(async () => "/tmp/plugin-source"),
    getLatestPluginRelease: vi.fn(async () => ({
      version: "v3.7.5",
      url: "",
      publishedAt: "",
    })),
  },
}));

vi.mock("node:fs", () => ({
  existsSync: vi.fn((path: string) => !path.endsWith("plugin-RpcServer-v3.9.2.zip")),
  mkdirSync: vi.fn(),
  copyFileSync: vi.fn(),
  readdirSync: vi.fn(() => []),
  statSync: vi.fn(() => ({ isDirectory: () => false })),
}));

describe("PluginManager", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("writes plugin config using the real node config from the database", async () => {
    const writePluginConfig = vi.fn();
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));

    const { PluginManager } = await import("../../src/core/PluginManager");

    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("SELECT * FROM plugins WHERE id")) {
          return {
            get: vi.fn(() => ({
              id: "RpcServer",
              name: "RPC Server",
              description: "Provides RPC access",
              category: "API",
              requires_config: 1,
              dependencies: null,
              default_config: "{}",
            })),
          };
        }

        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return {
            all: vi.fn(() => []),
          };
        }

        if (sql.includes("SELECT * FROM nodes WHERE id = ?")) {
          return {
            get: vi.fn(() => ({
              id: "node-1",
              name: "Remote CLI Node",
              type: "neo-cli",
              network: "mainnet",
              sync_mode: "full",
              version: "3.9.2",
              rpc_port: 10332,
              p2p_port: 10333,
              websocket_port: 10334,
              metrics_port: 2112,
              base_path: "/tmp/node-1",
              data_path: "/tmp/node-1/data",
              logs_path: "/tmp/node-1/logs",
              config_path: "/tmp/node-1/config",
              wallet_path: null,
              settings: "{}",
              created_at: 1,
              updated_at: 1,
            })),
          };
        }

        if (sql.includes("SELECT base_path FROM nodes WHERE id = ?")) {
          return {
            get: vi.fn(() => ({
              base_path: "/tmp/node-1",
            })),
          };
        }

        if (sql.includes("INSERT INTO node_plugins")) {
          return {
            run: vi.fn(),
          };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);
    await manager.installPlugin("node-1", "RpcServer", "3.9.2");

    expect(writePluginConfig).toHaveBeenCalledWith(
      "RpcServer",
      expect.objectContaining({
        id: "node-1",
        type: "neo-cli",
        network: "mainnet",
        ports: expect.objectContaining({
          rpc: 10332,
          websocket: 10334,
        }),
      }),
      {},
    );
  });

  it("does not persist static catalog defaults as node-specific plugin overrides", async () => {
    const writePluginConfig = vi.fn();
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));

    const { PluginManager } = await import("../../src/core/PluginManager");
    const insertRun = vi.fn();
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("SELECT * FROM plugins WHERE id")) {
          return {
            get: vi.fn(() => ({
              id: "RpcServer",
              name: "RPC Server",
              description: "Provides RPC access",
              category: "API",
              requires_config: 1,
              dependencies: null,
              default_config: JSON.stringify({
                Network: 860833102,
                BindAddress: "0.0.0.0",
                Port: 10332,
              }),
            })),
          };
        }

        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return {
            all: vi.fn(() => []),
          };
        }

        if (sql.includes("SELECT * FROM nodes WHERE id = ?")) {
          return {
            get: vi.fn(() => ({
              id: "node-1",
              name: "Testnet CLI Node",
              type: "neo-cli",
              network: "testnet",
              sync_mode: "full",
              version: "3.9.2",
              rpc_port: 20332,
              p2p_port: 20333,
              websocket_port: 20334,
              metrics_port: 2112,
              base_path: "/tmp/node-1",
              data_path: "/tmp/node-1/data",
              logs_path: "/tmp/node-1/logs",
              config_path: "/tmp/node-1/config",
              wallet_path: null,
              settings: "{}",
              created_at: 1,
              updated_at: 1,
            })),
          };
        }

        if (sql.includes("SELECT base_path FROM nodes WHERE id = ?")) {
          return {
            get: vi.fn(() => ({
              base_path: "/tmp/node-1",
            })),
          };
        }

        if (sql.includes("INSERT INTO node_plugins")) {
          return {
            run: insertRun,
          };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);
    await manager.installPlugin("node-1", "RpcServer", "3.9.2");

    expect(writePluginConfig).toHaveBeenCalledWith(
      "RpcServer",
      expect.objectContaining({
        network: "testnet",
        ports: expect.objectContaining({
          rpc: 20332,
        }),
      }),
      {},
    );
    expect(insertRun).toHaveBeenCalledWith(
      "node-1",
      "RpcServer",
      "v3.7.5",
      "{}",
      expect.any(Number),
    );
  });

  it("rejects config updates for plugins that are not installed on the node", async () => {
    const writePluginConfig = vi.fn();
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));

    const { PluginManager } = await import("../../src/core/PluginManager");
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("UPDATE node_plugins")) {
          return { run: vi.fn(() => ({ changes: 0 })) };
        }
        if (sql.includes("SELECT * FROM nodes WHERE id = ?")) {
          return { get: vi.fn(() => ({ id: "node-1" })) };
        }
        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    expect(() => manager.updatePluginConfig("node-1", "RpcServer", { Port: 10332 })).toThrow(/not installed/i);
    expect(writePluginConfig).not.toHaveBeenCalled();
  });

  it("rejects enablement changes for plugins that are not installed on the node", async () => {
    const { PluginManager } = await import("../../src/core/PluginManager");
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("UPDATE node_plugins")) {
          return { run: vi.fn(() => ({ changes: 0 })) };
        }
        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    expect(() => manager.setPluginEnabled("node-1", "RpcServer", true)).toThrow(/not installed/i);
  });

  it("treats disabled RocksDBStore as inactive when selecting the storage plugin", async () => {
    const { PluginManager } = await import("../../src/core/PluginManager");
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return {
            all: vi.fn(() => [
              {
                plugin_id: "RocksDBStore",
                version: "v3.7.5",
                config: "{}",
                installed_at: 1,
                enabled: 0,
              },
            ]),
          };
        }
        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    expect(manager.getStoragePlugin("node-1")).toBe("LevelDBStore");
  });
});
