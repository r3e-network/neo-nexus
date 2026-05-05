import type { InstalledPlugin, NodeInstance, NodeSettings } from "../../types";

export type ResponseRole = "admin" | "viewer" | undefined;

export type ViewerPlugin = Pick<InstalledPlugin, "id" | "version" | "installedAt" | "enabled">;

export type ViewerNode = Omit<NodeInstance, "paths" | "settings" | "plugins"> & {
  settings: Partial<NodeSettings>;
  plugins?: ViewerPlugin[];
};

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
