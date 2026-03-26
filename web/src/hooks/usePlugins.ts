import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";

export interface PluginDefinition {
  id: string;
  name: string;
  description: string;
  category: "API" | "Core" | "Storage" | "Tooling";
  requiresConfig: boolean;
  dependencies?: string[];
  defaultConfig?: Record<string, unknown>;
}

export interface InstalledPlugin {
  id: string;
  version: string;
  config: Record<string, unknown>;
  installedAt: number;
  enabled: boolean;
}

function assertNodeId(nodeId?: string) {
  if (!nodeId) {
    throw new Error("Select a neo-cli node first.");
  }
}

export function useAvailablePlugins(nodeId?: string) {
  return useQuery({
    queryKey: ["plugins", nodeId, "available"],
    queryFn: async () => {
      assertNodeId(nodeId);
      const response = await api.get<{ plugins: PluginDefinition[] }>(`/nodes/${nodeId}/plugins/available`);
      return response.plugins;
    },
    enabled: !!nodeId,
  });
}

export function useNodePlugins(nodeId?: string) {
  return useQuery({
    queryKey: ["plugins", nodeId, "installed"],
    queryFn: async () => {
      assertNodeId(nodeId);
      const response = await api.get<{ plugins: InstalledPlugin[] }>(`/nodes/${nodeId}/plugins`);
      return response.plugins;
    },
    enabled: !!nodeId,
  });
}

export function useInstallPlugin(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      pluginId,
      config,
    }: {
      pluginId: string;
      config?: Record<string, unknown>;
    }) => {
      assertNodeId(nodeId);
      return api.post(`/nodes/${nodeId}/plugins`, { pluginId, config });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", nodeId, "installed"] });
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
    },
  });
}

export function useUpdatePlugin(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      pluginId,
      config,
    }: {
      pluginId: string;
      config: Record<string, unknown>;
    }) => {
      assertNodeId(nodeId);
      return api.put(`/nodes/${nodeId}/plugins/${pluginId}`, { config });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", nodeId, "installed"] });
    },
  });
}

export function useUninstallPlugin(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (pluginId: string) => {
      assertNodeId(nodeId);
      return api.delete(`/nodes/${nodeId}/plugins/${pluginId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", nodeId, "installed"] });
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
    },
  });
}

export function useSetPluginEnabled(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      pluginId,
      enabled,
    }: {
      pluginId: string;
      enabled: boolean;
    }) => {
      assertNodeId(nodeId);
      return api.post(`/nodes/${nodeId}/plugins/${pluginId}/${enabled ? "enable" : "disable"}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", nodeId, "installed"] });
    },
  });
}
