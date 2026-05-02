import { describe, expect, it, vi } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";
import { NodeManager } from "../../src/core/NodeManager";

describe("NodeManager plugin mutations", () => {
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
});
