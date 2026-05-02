import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";
import { NodeManager } from "../../src/core/NodeManager";

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
