import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";
import { NodeManager } from "../../src/core/NodeManager";

describe("NodeManager secure signer binding", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("installs SignClient when a neo-cli node uses secure signer protection", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        installPlugin: ReturnType<typeof vi.fn>;
        updatePluginConfig: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
      secureSignerManager: {
        getProfile: ReturnType<typeof vi.fn>;
        buildSignClientConfig: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = {
      getInstalledPlugins: vi.fn()
        .mockReturnValueOnce([])
        .mockReturnValueOnce([{ id: "SignClient", enabled: true }]),
      installPlugin: vi.fn().mockResolvedValue(undefined),
      updatePluginConfig: vi.fn(),
      setPluginEnabled: vi.fn(),
    };
    manager.secureSignerManager = {
      getProfile: vi.fn(() => ({
        id: "signer-1",
        name: "Nitro Signer",
        endpoint: "vsock://2345:9991",
        enabled: true,
      })),
      buildSignClientConfig: vi.fn(() => ({
        Name: "Nitro Signer",
        Endpoint: "vsock://2345:9991",
      })),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await (manager as any).syncNodeSecureSigner("node-1");

    expect(manager.pluginManager.installPlugin).toHaveBeenCalledWith(
      "node-1",
      "SignClient",
      "3.7.5",
      {
        Name: "Nitro Signer",
        Endpoint: "vsock://2345:9991",
      },
    );
    expect(writeNodeConfig).toHaveBeenCalledWith(manager.getNode(), ["SignClient"]);
  });

  it("updates and enables SignClient when the plugin is already installed", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        installPlugin: ReturnType<typeof vi.fn>;
        updatePluginConfig: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
      secureSignerManager: {
        getProfile: ReturnType<typeof vi.fn>;
        buildSignClientConfig: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      version: "3.7.5",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = {
      getInstalledPlugins: vi.fn()
        .mockReturnValueOnce([{ id: "SignClient", enabled: false }])
        .mockReturnValueOnce([{ id: "SignClient", enabled: true }]),
      installPlugin: vi.fn(),
      updatePluginConfig: vi.fn(),
      setPluginEnabled: vi.fn(),
    };
    manager.secureSignerManager = {
      getProfile: vi.fn(() => ({
        id: "signer-1",
        name: "SGX Signer",
        endpoint: "https://sgx.example.com:9443",
        enabled: true,
      })),
      buildSignClientConfig: vi.fn(() => ({
        Name: "SGX Signer",
        Endpoint: "https://sgx.example.com:9443",
      })),
    };
    const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await (manager as any).syncNodeSecureSigner("node-1");

    expect(manager.pluginManager.installPlugin).not.toHaveBeenCalled();
    expect(manager.pluginManager.updatePluginConfig).toHaveBeenCalledWith("node-1", "SignClient", {
      Name: "SGX Signer",
      Endpoint: "https://sgx.example.com:9443",
    });
    expect(manager.pluginManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "SignClient", true);
    expect(writeNodeConfig).toHaveBeenCalledWith(manager.getNode(), ["SignClient"]);
  });

  it("rejects secure signer protection for neo-go nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      pluginManager: {
        getInstalledPlugins: ReturnType<typeof vi.fn>;
        installPlugin: ReturnType<typeof vi.fn>;
        updatePluginConfig: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
      };
      secureSignerManager: {
        getProfile: ReturnType<typeof vi.fn>;
        buildSignClientConfig: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-go",
      version: "0.118.0",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.pluginManager = {
      getInstalledPlugins: vi.fn(() => []),
      installPlugin: vi.fn(),
      updatePluginConfig: vi.fn(),
      setPluginEnabled: vi.fn(),
    };
    manager.secureSignerManager = {
      getProfile: vi.fn(),
      buildSignClientConfig: vi.fn(),
    };

    await expect((manager as any).syncNodeSecureSigner("node-1")).rejects.toThrow(/neo-cli/i);
  });
});
