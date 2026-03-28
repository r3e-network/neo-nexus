import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";
import { REFETCH_INTERVALS } from "../config/constants";

export interface RemoteServerProfile {
  id: string;
  name: string;
  baseUrl: string;
  description?: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface RemoteServerStatusSummary {
  totalNodes: number;
  runningNodes: number;
  syncingNodes: number;
  errorNodes: number;
  totalBlocks: number;
  totalPeers: number;
  timestamp: number;
}

export interface RemoteServerNodeSummary {
  id: string;
  name: string;
  type: "neo-cli" | "neo-go";
  network: "mainnet" | "testnet" | "private";
  status: string;
  version: string;
  metrics: {
    blockHeight: number;
    headerHeight?: number;
    connectedPeers: number;
    syncProgress?: number;
  } | null;
  uptime?: number;
  lastUpdate?: number;
}

export interface RemoteServerSystemMetrics {
  cpu: {
    usage: number;
    cores: number;
  };
  memory: {
    percentage: number;
    used: number;
    total: number;
  };
  disk: {
    percentage: number;
    used: number;
    total: number;
  };
}

export interface RemoteServerSummary {
  profile: RemoteServerProfile;
  reachable: boolean;
  status?: RemoteServerStatusSummary;
  nodes?: RemoteServerNodeSummary[];
  systemMetrics?: RemoteServerSystemMetrics;
  error?: string;
}

export function useServers() {
  return useQuery({
    queryKey: ["servers"],
    queryFn: async () => {
      const response = await api.get<{ servers: RemoteServerSummary[] }>("/servers");
      return response.servers;
    },
    refetchInterval: REFETCH_INTERVALS.servers,
  });
}

export function useCreateServer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: {
      name: string;
      baseUrl: string;
      description?: string;
      enabled?: boolean;
    }) => {
      const response = await api.post<{ server: RemoteServerProfile }>("/servers", payload);
      return response.server;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
    },
  });
}

export function useUpdateServer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      id,
      payload,
    }: {
      id: string;
      payload: Partial<RemoteServerProfile>;
    }) => {
      const response = await api.put<{ server: RemoteServerProfile }>(`/servers/${id}`, payload);
      return response.server;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
    },
  });
}

export function useDeleteServer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/servers/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
    },
  });
}
