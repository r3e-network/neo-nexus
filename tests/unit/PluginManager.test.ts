import { beforeEach, describe, expect, it, vi } from "vitest";
import { copyFileSync, lstatSync, readdirSync, rmSync } from "node:fs";

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
  lstatSync: vi.fn(() => ({ isDirectory: () => false, isSymbolicLink: () => false })),
  rmSync: vi.fn(),
}));

describe("PluginManager", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    vi.mocked(readdirSync).mockReturnValue([]);
    vi.mocked(lstatSync).mockImplementation(() => ({ isDirectory: () => false, isSymbolicLink: () => false }) as never);
    delete process.env.NEO_PLUGIN_BUILD_DIR;
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

  it("resolves plugin release versions from node versions and falls back to latest release", async () => {
    const { resolvePluginReleaseVersion } = await import("../../src/core/PluginManager");

    expect(resolvePluginReleaseVersion("v3.9.2", "v9.9.9")).toBe("v3.7.5");
    expect(resolvePluginReleaseVersion("3.7.4", "v9.9.9")).toBe("v3.7.4");
    expect(resolvePluginReleaseVersion("3.8.0", "v9.9.9")).toBe("v9.9.9");
  });

  it("does not persist an install if plugin config generation fails", async () => {
    const writePluginConfig = vi.fn(() => {
      throw new Error("disk full");
    });
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
              default_config: "{}",
            })),
          };
        }

        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return { all: vi.fn(() => []) };
        }

        if (sql.includes("SELECT base_path FROM nodes WHERE id = ?")) {
          return { get: vi.fn(() => ({ base_path: "/tmp/node-1" })) };
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

        if (sql.includes("INSERT INTO node_plugins")) {
          return { run: insertRun };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    await expect(manager.installPlugin("node-1", "RpcServer", "3.9.2")).rejects.toThrow(/disk full/i);

    expect(insertRun).not.toHaveBeenCalled();
    expect(rmSync).toHaveBeenCalledWith("/tmp/node-1/Plugins/RpcServer", { recursive: true, force: true });
  });

  it("does not update persisted config if rewriting the plugin config file fails", async () => {
    const writePluginConfig = vi.fn(() => {
      throw new Error("disk full");
    });
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));

    const { PluginManager } = await import("../../src/core/PluginManager");
    const updateRun = vi.fn(() => ({ changes: 1 }));
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return {
            all: vi.fn(() => [
              {
                plugin_id: "RpcServer",
                version: "v3.7.5",
                config: JSON.stringify({ Port: 10332 }),
                installed_at: 1,
                enabled: 1,
              },
            ]),
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

        if (sql.includes("UPDATE node_plugins")) {
          return { run: updateRun };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    expect(() => manager.updatePluginConfig("node-1", "RpcServer", { Port: 20332 })).toThrow(/disk full/i);
    expect(updateRun).not.toHaveBeenCalled();
  });

  it("does not remove database or files if rewriting node config for uninstall fails", async () => {
    const writeNodeConfig = vi.fn(() => {
      throw new Error("disk full");
    });
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writeNodeConfig,
      },
    }));

    const { PluginManager } = await import("../../src/core/PluginManager");
    const deleteRun = vi.fn(() => ({ changes: 1 }));
    const db = {
      prepare: vi.fn((sql: string) => {
        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return {
            all: vi.fn(() => [
              {
                plugin_id: "RpcServer",
                version: "v3.7.5",
                config: "{}",
                installed_at: 1,
                enabled: 1,
              },
              {
                plugin_id: "RestServer",
                version: "v3.7.5",
                config: "{}",
                installed_at: 2,
                enabled: 1,
              },
            ]),
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
          return { get: vi.fn(() => ({ base_path: "/tmp/node-1" })) };
        }

        if (sql.includes("DELETE FROM node_plugins")) {
          return { run: deleteRun };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);

    await expect(manager.uninstallPlugin("node-1", "RpcServer")).rejects.toThrow(/disk full/i);

    expect(writeNodeConfig).toHaveBeenCalledWith(expect.objectContaining({ id: "node-1" }), ["RestServer"]);
    expect(deleteRun).not.toHaveBeenCalled();
    expect(rmSync).not.toHaveBeenCalled();
  });

  it("copies only plugin-specific local build artifacts plus native runtimes", async () => {
    const writePluginConfig = vi.fn();
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));
    process.env.NEO_PLUGIN_BUILD_DIR = "/tmp/local-neo-modules";
    vi.mocked(readdirSync).mockImplementation((path: string) => {
      if (path.endsWith("runtimes")) return ["native-lib.dylib"];
      return ["RpcServer.dll", "Neo.dll", "RpcServer.pdb", "config.json", "runtimes"];
    });
    vi.mocked(lstatSync).mockImplementation((path: string) => ({
      isDirectory: () => path.endsWith("runtimes"),
      isSymbolicLink: () => false,
    }) as never);

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
          return { all: vi.fn(() => []) };
        }

        if (sql.includes("SELECT base_path FROM nodes WHERE id = ?")) {
          return { get: vi.fn(() => ({ base_path: "/tmp/node-1" })) };
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

        if (sql.includes("INSERT INTO node_plugins")) {
          return { run: vi.fn() };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);
    await manager.installPlugin("node-1", "RpcServer", "3.9.2");

    expect(copyFileSync).toHaveBeenCalledWith("/tmp/plugin-source/RpcServer.dll", "/tmp/node-1/Plugins/RpcServer/RpcServer.dll");
    expect(copyFileSync).toHaveBeenCalledWith("/tmp/plugin-source/RpcServer.pdb", "/tmp/node-1/Plugins/RpcServer/RpcServer.pdb");
    expect(copyFileSync).toHaveBeenCalledWith("/tmp/plugin-source/config.json", "/tmp/node-1/Plugins/RpcServer/config.json");
    expect(copyFileSync).toHaveBeenCalledWith("/tmp/plugin-source/runtimes/native-lib.dylib", "/tmp/node-1/Plugins/RpcServer/runtimes/native-lib.dylib");
    expect(copyFileSync).not.toHaveBeenCalledWith("/tmp/plugin-source/Neo.dll", expect.any(String));
  });

  it("does not follow symlinks from plugin build output", async () => {
    const writePluginConfig = vi.fn();
    vi.doMock("../../src/core/ConfigManager", () => ({
      ConfigManager: {
        writePluginConfig,
      },
    }));
    vi.mocked(readdirSync).mockReturnValue(["RpcServer.dll", "leaked-key.json"] as never);
    vi.mocked(lstatSync).mockImplementation((path: string) => ({
      isDirectory: () => false,
      isSymbolicLink: () => path.endsWith("leaked-key.json"),
    }) as never);

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
          return { all: vi.fn(() => []) };
        }

        if (sql.includes("SELECT base_path FROM nodes WHERE id = ?")) {
          return { get: vi.fn(() => ({ base_path: "/tmp/node-1" })) };
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

        if (sql.includes("INSERT INTO node_plugins")) {
          return { run: vi.fn() };
        }

        throw new Error(`Unexpected SQL: ${sql}`);
      }),
    };

    const manager = new PluginManager(db as never);
    await manager.installPlugin("node-1", "RpcServer", "3.9.2");

    expect(copyFileSync).toHaveBeenCalledWith("/tmp/plugin-source/RpcServer.dll", "/tmp/node-1/Plugins/RpcServer/RpcServer.dll");
    expect(copyFileSync).not.toHaveBeenCalledWith("/tmp/plugin-source/leaked-key.json", expect.any(String));
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
        if (sql.includes("SELECT np.*, p.name, p.category")) {
          return { all: vi.fn(() => []) };
        }
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
