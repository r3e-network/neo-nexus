import { describe, expect, it, vi } from "vitest";
import { ALL_TOOLS, executeTool, toolsForUser } from "../../src/agent/tools";
import type { ToolContext } from "../../src/agent/types";
import type { NodeInstance } from "../../src/types";

function makeContext(overrides: Partial<ToolContext> = {}): ToolContext {
  return {
    user: { id: "u1", username: "alice", role: "viewer" },
    deps: {
      nodeManager: {
        getAllNodes: () => [],
        getNode: () => null,
        getNodeLogs: () => [],
        startNode: vi.fn(),
        stopNode: vi.fn(),
        restartNode: vi.fn(),
        setPluginEnabled: vi.fn(),
      },
      remoteServerManager: { listServersWithStatus: async () => [] },
      integrationManager: { listAll: () => [] },
      metricsCollector: { collectSystemMetrics: async () => ({ cpu: { usage: 0, cores: 1 }, memory: { total: 1, used: 0, free: 1, percentage: 0 }, disk: { total: 1, used: 0, free: 1, percentage: 0 }, network: { rx: 0, tx: 0 } }) },
      networkHeightTracker: { getHeight: () => 12345 },
      auditLogger: { log: vi.fn() },
    } as never,
    ...overrides,
  };
}

function makeNode(): NodeInstance {
  return {
    id: "node-1",
    name: "Validator 1",
    chain: "n3",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.8.1",
    ports: { rpc: 10332, p2p: 10333 },
    paths: {
      base: "/srv/neo/node-1",
      data: "/srv/neo/node-1/data",
      logs: "/srv/neo/node-1/logs",
      config: "/srv/neo/node-1/config",
      wallet: "/srv/neo/node-1/wallet.json",
    },
    settings: {
      maxConnections: 40,
      relay: true,
      activeDataContextId: "ctx-secret",
      customConfig: { walletPassword: "secret-password" },
      keyProtection: { mode: "secure-signer", signerProfileId: "signer-secret" },
    },
    process: { status: "running", pid: 1234 },
    plugins: [
      {
        id: "RpcServer",
        version: "1.0.0",
        installedAt: 1710000000000,
        enabled: true,
        config: { bindAddress: "0.0.0.0", token: "plugin-secret" },
      },
    ],
    createdAt: 1710000000000,
    updatedAt: 1710000001000,
  };
}

describe("agent tools", () => {
  it("toolsForUser hides admin-only tools from viewers", () => {
    const viewerTools = toolsForUser("viewer").map((t) => t.name);
    const adminTools = toolsForUser("admin").map((t) => t.name);
    expect(viewerTools).not.toContain("start_node");
    expect(viewerTools).not.toContain("stop_node");
    expect(viewerTools).not.toContain("restart_node");
    expect(viewerTools).not.toContain("set_plugin_enabled");
    expect(adminTools).toEqual(expect.arrayContaining(["start_node", "stop_node", "restart_node", "set_plugin_enabled"]));
  });

  it("every tool has a unique name and a non-empty description", () => {
    const names = ALL_TOOLS.map((t) => t.name);
    expect(new Set(names).size).toBe(names.length);
    for (const t of ALL_TOOLS) {
      expect(t.description.length).toBeGreaterThan(20);
      expect(t.inputSchema.type).toBe("object");
    }
  });

  it("executeTool blocks admin tools when caller is a viewer", async () => {
    const ctx = makeContext();
    await expect(executeTool("start_node", { node_id: "n1" }, ctx)).rejects.toThrow(/admin role/i);
  });

  it("executeTool runs read-only tool for viewer", async () => {
    const ctx = makeContext();
    const result = (await executeTool("get_network_height", { network: "mainnet" }, ctx)) as { network: string; height: number };
    expect(result).toEqual({ network: "mainnet", height: 12345 });
  });

  it("redacts node paths, sensitive settings, and plugin config for viewer get_node", async () => {
    const node = makeNode();
    const ctx = makeContext({
      deps: { ...makeContext().deps, nodeManager: { ...makeContext().deps.nodeManager, getNode: () => node } as never },
    });

    const result = await executeTool("get_node", { node_id: "node-1" }, ctx);
    const json = JSON.stringify(result);

    expect(result).toMatchObject({
      id: "node-1",
      settings: {
        maxConnections: 40,
        relay: true,
        keyProtection: { mode: "secure-signer" },
      },
      plugins: [{ id: "RpcServer", version: "1.0.0", enabled: true }],
    });
    expect(json).not.toContain("/srv/neo");
    expect(json).not.toContain("secret-password");
    expect(json).not.toContain("ctx-secret");
    expect(json).not.toContain("signer-secret");
    expect(json).not.toContain("plugin-secret");
  });

  it("redacts plugin config for viewer list_plugins", async () => {
    const node = makeNode();
    const ctx = makeContext({
      deps: { ...makeContext().deps, nodeManager: { ...makeContext().deps.nodeManager, getNode: () => node } as never },
    });

    const result = await executeTool("list_plugins", { node_id: "node-1" }, ctx);

    expect(result).toEqual([
      {
        id: "RpcServer",
        version: "1.0.0",
        installedAt: 1710000000000,
        enabled: true,
      },
    ]);
    expect(JSON.stringify(result)).not.toContain("plugin-secret");
  });

  it("redacts integration config and provider errors for viewer list_integrations", async () => {
    const ctx = makeContext({
      deps: {
        ...makeContext().deps,
        integrationManager: {
          listAll: () => [{
            id: "webhook",
            name: "Webhook",
            description: "Send event notifications",
            category: "alerting",
            enabled: true,
            configured: true,
            configSchema: [{ key: "url", label: "URL", type: "url", placeholder: "https://hooks.example", required: true }],
            configValues: { url: "https://hooks.example.test/secret-path" },
            lastTestAt: "2026-05-06T12:00:00Z",
            lastError: "upstream rejected token=super-secret",
          }],
        },
      } as never,
    });

    const result = await executeTool("list_integrations", {}, ctx);
    const json = JSON.stringify(result);

    expect(result).toEqual([{
      id: "webhook",
      name: "Webhook",
      description: "Send event notifications",
      category: "alerting",
      enabled: true,
      configured: true,
      lastTestAt: "2026-05-06T12:00:00Z",
    }]);
    expect(json).not.toContain("secret-path");
    expect(json).not.toContain("super-secret");
    expect(json).not.toContain("configSchema");
    expect(json).not.toContain("configValues");
    expect(json).not.toContain("lastError");
  });

  it("redacts remote server URLs and probe errors for viewer list_remote_servers", async () => {
    const ctx = makeContext({
      deps: {
        ...makeContext().deps,
        remoteServerManager: {
          listServersWithStatus: async () => [{
            profile: {
              id: "srv-1",
              name: "Tokyo",
              baseUrl: "https://tokyo.internal.example",
              description: "Remote control plane",
              enabled: true,
              createdAt: 1710000000000,
              updatedAt: 1710000001000,
            },
            reachable: false,
            error: "connect ECONNREFUSED https://tokyo.internal.example/api/public/status",
          }],
        },
      } as never,
    });

    const result = await executeTool("list_remote_servers", {}, ctx);
    const json = JSON.stringify(result);

    expect(result).toEqual([{
      profile: {
        id: "srv-1",
        name: "Tokyo",
        description: "Remote control plane",
        enabled: true,
        createdAt: 1710000000000,
        updatedAt: 1710000001000,
      },
      reachable: false,
    }]);
    expect(json).not.toContain("baseUrl");
    expect(json).not.toContain("tokyo.internal.example");
    expect(json).not.toContain("ECONNREFUSED");
  });

  it("executeTool routes start_node through nodeManager and audits", async () => {
    const ctx = makeContext({ user: { id: "u1", username: "alice", role: "admin" } });
    const result = await executeTool("start_node", { node_id: "node-abc" }, ctx);
    expect(ctx.deps.nodeManager.startNode).toHaveBeenCalledWith("node-abc");
    expect(ctx.deps.auditLogger.log).toHaveBeenCalledWith(expect.objectContaining({ action: "agent.node.start", resourceId: "node-abc", username: "alice" }));
    expect(result).toEqual({ node_id: "node-abc", requested: "start" });
  });

  it("executeTool rejects unknown tool", async () => {
    const ctx = makeContext();
    await expect(executeTool("nonexistent_tool", {}, ctx)).rejects.toThrow(/unknown tool/i);
  });

  it("get_node_logs clamps count to bounds", async () => {
    const node = makeNode();
    const getNodeLogs = vi.fn(() => [{ timestamp: 0, level: "info", source: "n", message: "" }]);
    const ctx = makeContext({
      deps: { ...makeContext().deps, nodeManager: { ...makeContext().deps.nodeManager, getNode: () => node, getNodeLogs } as never },
    });
    await executeTool("get_node_logs", { node_id: "n1", count: 999999 }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 500);
    await executeTool("get_node_logs", { node_id: "n1", count: -5 }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 1);
    await executeTool("get_node_logs", { node_id: "n1" }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 100);
  });

  it("redacts node paths and secrets for viewer get_node_logs", async () => {
    const node = makeNode();
    const getNodeLogs = vi.fn(() => [{
      timestamp: 0,
      level: "error",
      source: "node",
      message: "Could not load /srv/neo/node-1/wallet.json password=hunter2 token=abc123",
    }]);
    const ctx = makeContext({
      deps: { ...makeContext().deps, nodeManager: { ...makeContext().deps.nodeManager, getNode: () => node, getNodeLogs } as never },
    });

    const result = await executeTool("get_node_logs", { node_id: "node-1" }, ctx);
    const json = JSON.stringify(result);

    expect(json).toContain("[node-path]");
    expect(json).toContain("password=[redacted]");
    expect(json).toContain("token=[redacted]");
    expect(json).not.toContain("/srv/neo");
    expect(json).not.toContain("hunter2");
    expect(json).not.toContain("abc123");
  });
});
