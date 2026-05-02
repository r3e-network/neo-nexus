import { describe, expect, it, vi } from "vitest";
import { ALL_TOOLS, executeTool, toolsForUser } from "../../src/agent/tools";
import type { ToolContext } from "../../src/agent/types";

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
    const getNodeLogs = vi.fn(() => [{ timestamp: 0, level: "info", source: "n", message: "" }]);
    const ctx = makeContext({
      deps: { ...makeContext().deps, nodeManager: { ...makeContext().deps.nodeManager, getNodeLogs } as never },
    });
    await executeTool("get_node_logs", { node_id: "n1", count: 999999 }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 500);
    await executeTool("get_node_logs", { node_id: "n1", count: -5 }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 1);
    await executeTool("get_node_logs", { node_id: "n1" }, ctx);
    expect(getNodeLogs).toHaveBeenCalledWith("n1", 100);
  });
});
