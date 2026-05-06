import type { InstalledPlugin, NodeInstance, NodeSettings } from "../../types";

export type ResponseRole = "admin" | "viewer" | undefined;

export type ViewerPlugin = Pick<InstalledPlugin, "id" | "version" | "installedAt" | "enabled">;

export type ViewerNode = Omit<NodeInstance, "paths" | "settings" | "plugins"> & {
  settings: Partial<NodeSettings>;
  plugins?: ViewerPlugin[];
};

export interface StoredLogEntry {
  timestamp: number;
  level: string;
  message: string;
  source?: string;
}

export function sanitizeNodeSettingsForViewer(settings: NodeSettings): Partial<NodeSettings> {
  return {
    ...(settings.maxConnections !== undefined ? { maxConnections: settings.maxConnections } : {}),
    ...(settings.minPeers !== undefined ? { minPeers: settings.minPeers } : {}),
    ...(settings.maxPeers !== undefined ? { maxPeers: settings.maxPeers } : {}),
    ...(settings.relay !== undefined ? { relay: settings.relay } : {}),
    ...(settings.debugMode !== undefined ? { debugMode: settings.debugMode } : {}),
    ...(settings.resourceLimits ? { resourceLimits: settings.resourceLimits } : {}),
    ...(settings.keyProtection ? { keyProtection: { mode: settings.keyProtection.mode } } : {}),
    ...(settings.import ? {
      import: {
        imported: settings.import.imported,
        ownershipMode: settings.import.ownershipMode,
      },
    } : {}),
  };
}

export function sanitizePluginForViewer(plugin: InstalledPlugin): ViewerPlugin {
  const { id, version, installedAt, enabled } = plugin;
  return { id, version, installedAt, enabled };
}

export function sanitizeNodeForViewer(node: NodeInstance): ViewerNode {
  const { paths: _paths, settings, plugins, ...safeNode } = node;
  return {
    ...safeNode,
    settings: sanitizeNodeSettingsForViewer(settings),
    plugins: plugins?.map(sanitizePluginForViewer),
  };
}

export function nodeResponseForRole(role: ResponseRole, node: NodeInstance): NodeInstance | ViewerNode {
  return role === "viewer" ? sanitizeNodeForViewer(node) : node;
}

export function pluginResponseForRole(role: ResponseRole, plugins: InstalledPlugin[]): InstalledPlugin[] | ViewerPlugin[] {
  return role === "viewer" ? plugins.map(sanitizePluginForViewer) : plugins;
}

export function sanitizeLogForViewer<T extends StoredLogEntry>(entry: T, node?: NodeInstance | null): T {
  return {
    ...entry,
    ...(entry.source !== undefined ? { source: "node" } : {}),
    message: redactSensitiveLogText(entry.message, node),
  } as T;
}

export function logResponseForRole<T extends StoredLogEntry>(
  role: ResponseRole,
  logs: T[],
  node?: NodeInstance | null,
): T[] {
  return role === "viewer" ? logs.map((entry) => sanitizeLogForViewer(entry, node)) : logs;
}

function redactSensitiveLogText(text: string, node?: NodeInstance | null): string {
  let redacted = text;

  const paths = node
    ? Object.values(node.paths).filter((value): value is string => typeof value === "string" && value.length > 0)
    : [];
  for (const path of paths.sort((a, b) => b.length - a.length)) {
    redacted = redacted.replace(new RegExp(escapeRegExp(path), "g"), "[node-path]");
  }

  redacted = redacted
    .replace(
      /("(?:api[_-]?key|authorization|bearer|password|passwd|pwd|secret|source[_-]?token|token|walletPassword)"\s*:\s*)"[^"]*"/gi,
      '$1"[redacted]"',
    )
    .replace(
      /\b(api[_-]?key|authorization|bearer|password|passwd|pwd|secret|source[_-]?token|token|walletPassword)(\s*[=:]\s*)([^\s,;]+)/gi,
      "$1$2[redacted]",
    )
    .replace(/:\/\/[^/?#\s:@]+:[^/?#\s@]+@/g, "://[redacted]@")
    .replace(/(?<!:)\/(?:Users|home|var|opt|srv|tmp|etc|root)\/[^\s'",)]+/g, "[path]")
    .replace(/[A-Za-z]:\\[^\s'",)]+/g, "[path]");

  return redacted;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
