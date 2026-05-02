import type { NodeInstance } from "../types";
import type { ToolContext, ToolDefinition } from "./types";

const READ_TOOLS: ToolDefinition[] = [
  {
    name: "list_nodes",
    description:
      "List all managed Neo nodes (neo-cli and neo-go) with type, network, status, version, ports, ownership, and the most recent metrics snapshot. Use this for a fleet overview.",
    inputSchema: { type: "object", properties: {} },
    requiresAdmin: false,
    async execute(_input, ctx) {
      return ctx.deps.nodeManager.getAllNodes().map((n) => summarizeNode(n));
    },
  },
  {
    name: "get_node",
    description:
      "Fetch full detail for a single node by id, including config paths, ports, secure-signer binding, plugins, and the latest metrics. Use when the user asks about a specific node.",
    inputSchema: {
      type: "object",
      properties: { node_id: { type: "string", description: "Node id (e.g. node-mn80ntf9-phx68)" } },
      required: ["node_id"],
    },
    requiresAdmin: false,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      const node = ctx.deps.nodeManager.getNode(id);
      if (!node) throw new Error(`Node ${id} not found`);
      return node;
    },
  },
  {
    name: "get_node_logs",
    description:
      "Fetch the most recent log lines for a node (default 100, max 500). Returns timestamp, level, and message. Use to investigate errors, sync issues, or recent activity.",
    inputSchema: {
      type: "object",
      properties: {
        node_id: { type: "string" },
        count: { type: "integer", minimum: 1, maximum: 500, default: 100 },
      },
      required: ["node_id"],
    },
    requiresAdmin: false,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      const count = clampInteger(input.count, 1, 500, 100);
      return ctx.deps.nodeManager.getNodeLogs(id, count);
    },
  },
  {
    name: "list_plugins",
    description:
      "List installed plugins on a node, including which are enabled, their version, and config keys. Plugins are only meaningful for neo-cli nodes.",
    inputSchema: {
      type: "object",
      properties: { node_id: { type: "string" } },
      required: ["node_id"],
    },
    requiresAdmin: false,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      const node = ctx.deps.nodeManager.getNode(id);
      if (!node) throw new Error(`Node ${id} not found`);
      return node.plugins ?? [];
    },
  },
  {
    name: "get_system_metrics",
    description:
      "Get current host system metrics: CPU usage, memory percentage and bytes, disk usage, network rx/tx bytes. Use for capacity-planning questions.",
    inputSchema: { type: "object", properties: {} },
    requiresAdmin: false,
    async execute(_input, ctx) {
      return ctx.deps.metricsCollector.collectSystemMetrics();
    },
  },
  {
    name: "get_network_height",
    description:
      "Get the cached Neo network block height for mainnet or testnet from this control plane's seed-node tracker. Use to compare against per-node block height for sync progress.",
    inputSchema: {
      type: "object",
      properties: { network: { type: "string", enum: ["mainnet", "testnet"] } },
      required: ["network"],
    },
    requiresAdmin: false,
    async execute(input, ctx) {
      const network = stringInput(input, "network");
      if (network !== "mainnet" && network !== "testnet") {
        throw new Error("network must be mainnet or testnet");
      }
      return { network, height: ctx.deps.networkHeightTracker.getHeight(network) };
    },
  },
  {
    name: "list_remote_servers",
    description:
      "List federated remote NeoNexus instances and their reachability + status summary. Use to check fleet health across multiple control planes.",
    inputSchema: { type: "object", properties: {} },
    requiresAdmin: false,
    async execute(_input, ctx) {
      return ctx.deps.remoteServerManager.listServersWithStatus();
    },
  },
  {
    name: "list_integrations",
    description:
      "List configured external integrations (metrics, logging, uptime, alerting, errors): which are enabled, last test result, and whether credentials are present. Credentials are redacted.",
    inputSchema: { type: "object", properties: {} },
    requiresAdmin: false,
    async execute(_input, ctx) {
      return ctx.deps.integrationManager.listAll();
    },
  },
];

const CONTROL_TOOLS: ToolDefinition[] = [
  {
    name: "start_node",
    description:
      "Start a node that is currently stopped. Verifies the node exists; emits a status WebSocket event. Admin only.",
    inputSchema: {
      type: "object",
      properties: { node_id: { type: "string" } },
      required: ["node_id"],
    },
    requiresAdmin: true,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      await ctx.deps.nodeManager.startNode(id);
      ctx.deps.auditLogger.log({ action: "agent.node.start", resourceType: "node", resourceId: id, userId: ctx.user.id, username: ctx.user.username });
      return { node_id: id, requested: "start" };
    },
  },
  {
    name: "stop_node",
    description:
      "Stop a running node gracefully. Use force=true only if the node is wedged. Admin only.",
    inputSchema: {
      type: "object",
      properties: {
        node_id: { type: "string" },
        force: { type: "boolean", default: false },
      },
      required: ["node_id"],
    },
    requiresAdmin: true,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      const force = Boolean(input.force);
      await ctx.deps.nodeManager.stopNode(id, force);
      ctx.deps.auditLogger.log({ action: "agent.node.stop", resourceType: "node", resourceId: id, userId: ctx.user.id, username: ctx.user.username, details: JSON.stringify({ force }) });
      return { node_id: id, requested: "stop", force };
    },
  },
  {
    name: "restart_node",
    description:
      "Stop then start a node. Equivalent to stop_node followed by start_node. Admin only.",
    inputSchema: {
      type: "object",
      properties: { node_id: { type: "string" } },
      required: ["node_id"],
    },
    requiresAdmin: true,
    async execute(input, ctx) {
      const id = stringInput(input, "node_id");
      await ctx.deps.nodeManager.restartNode(id);
      ctx.deps.auditLogger.log({ action: "agent.node.restart", resourceType: "node", resourceId: id, userId: ctx.user.id, username: ctx.user.username });
      return { node_id: id, requested: "restart" };
    },
  },
  {
    name: "set_plugin_enabled",
    description:
      "Enable or disable a plugin on a neo-cli node. The node must be stopped before toggling. Admin only.",
    inputSchema: {
      type: "object",
      properties: {
        node_id: { type: "string" },
        plugin_id: { type: "string" },
        enabled: { type: "boolean" },
      },
      required: ["node_id", "plugin_id", "enabled"],
    },
    requiresAdmin: true,
    async execute(input, ctx) {
      const nodeId = stringInput(input, "node_id");
      const pluginId = stringInput(input, "plugin_id");
      const enabled = Boolean(input.enabled);
      // Cast through unknown — the runtime will reject unknown plugin ids upstream.
      await ctx.deps.nodeManager.setPluginEnabled(nodeId, pluginId as never, enabled);
      ctx.deps.auditLogger.log({
        action: "agent.plugin.toggle",
        resourceType: "node",
        resourceId: nodeId,
        userId: ctx.user.id,
        username: ctx.user.username,
        details: JSON.stringify({ pluginId, enabled }),
      });
      return { node_id: nodeId, plugin_id: pluginId, enabled };
    },
  },
];

export const ALL_TOOLS: ToolDefinition[] = [...READ_TOOLS, ...CONTROL_TOOLS];

export function toolsForUser(role: "admin" | "viewer"): ToolDefinition[] {
  return ALL_TOOLS.filter((tool) => !tool.requiresAdmin || role === "admin");
}

export async function executeTool(
  toolName: string,
  input: Record<string, unknown>,
  ctx: ToolContext,
): Promise<unknown> {
  const tool = ALL_TOOLS.find((t) => t.name === toolName);
  if (!tool) {
    throw new Error(`Unknown tool: ${toolName}`);
  }
  if (tool.requiresAdmin && ctx.user.role !== "admin") {
    throw new Error(`Tool ${toolName} requires the admin role; current role is ${ctx.user.role}.`);
  }
  return tool.execute(input, ctx);
}

function stringInput(input: Record<string, unknown>, key: string): string {
  const value = input[key];
  if (typeof value !== "string" || !value) {
    throw new Error(`Missing or invalid '${key}' parameter (expected non-empty string)`);
  }
  return value;
}

function clampInteger(value: unknown, min: number, max: number, fallback: number): number {
  if (typeof value !== "number" || !Number.isInteger(value)) return fallback;
  return Math.min(max, Math.max(min, value));
}

function summarizeNode(node: NodeInstance) {
  return {
    id: node.id,
    name: node.name,
    type: node.type,
    network: node.network,
    status: node.process.status,
    version: node.version,
    ports: node.ports,
    ownership: node.settings.import?.ownershipMode ?? "managed",
    metrics: node.metrics,
  };
}
