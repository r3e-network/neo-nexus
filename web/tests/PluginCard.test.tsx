import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { PluginCard } from "../src/pages/plugins/PluginCard";

const rpcPlugin = {
  id: "RpcServer",
  name: "RPC Server",
  description: "Provides JSON-RPC",
  category: "API" as const,
  requiresConfig: true,
  defaultConfig: {},
};

describe("PluginCard", () => {
  it("renders installed version and an enablement switch for disabled plugins", () => {
    const html = renderToStaticMarkup(
      <PluginCard
        plugin={rpcPlugin}
        installed={{
          id: "RpcServer",
          version: "v3.7.5",
          config: { Port: 10332 },
          installedAt: 1,
          enabled: false,
        }}
        configValues={{ Port: 10332 }}
        onConfigChange={vi.fn()}
        onInstall={vi.fn()}
        onRemove={vi.fn()}
        onSetEnabled={vi.fn()}
        onSaveConfig={vi.fn()}
        isInstalling={false}
        isRemoving={false}
        isSettingEnabled={false}
        isSaving={false}
      />,
    );

    expect(html).toContain("v3.7.5");
    expect(html).toContain("Installed but disabled");
    expect(html).toContain("aria-checked=\"false\"");
    expect(html).toContain("Remove");
    expect(html).not.toContain("toggle above will uninstall");
  });

  it("renders an explicit install action for plugins that are not installed", () => {
    const html = renderToStaticMarkup(
      <PluginCard
        plugin={rpcPlugin}
        installed={undefined}
        configValues={{ Port: 10332 }}
        onConfigChange={vi.fn()}
        onInstall={vi.fn()}
        onRemove={vi.fn()}
        onSetEnabled={vi.fn()}
        onSaveConfig={vi.fn()}
        isInstalling={false}
        isRemoving={false}
        isSettingEnabled={false}
        isSaving={false}
      />,
    );

    expect(html).toContain("Install");
    expect(html).toContain("When enabled:");
    expect(html).not.toContain("Remove");
  });
});
