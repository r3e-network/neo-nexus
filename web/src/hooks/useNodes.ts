import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../utils/api';
import { REFETCH_INTERVALS } from '../config/constants';

export type ImportedNodeOwnershipMode = 'observe-only' | 'managed-config' | 'managed-process';

export interface Node {
  id: string;
  name: string;
  type: 'neo-cli' | 'neo-go';
  network: 'mainnet' | 'testnet' | 'private';
  syncMode: 'full' | 'light';
  version: string;
  ports: {
    rpc: number;
    p2p: number;
    websocket?: number;
    metrics?: number;
  };
  process: {
    status: 'stopped' | 'starting' | 'running' | 'stopping' | 'error' | 'syncing';
    pid?: number;
    errorMessage?: string;
    uptime?: number;
  };
  metrics?: {
    blockHeight: number;
    headerHeight: number;
    connectedPeers: number;
    memoryUsage: number;
    cpuUsage: number;
  };
  settings: {
    maxConnections?: number;
    minPeers?: number;
    maxPeers?: number;
    relay?: boolean;
    debugMode?: boolean;
    customConfig?: Record<string, unknown>;
    keyProtection?: {
      mode: 'standard' | 'secure-signer';
      signerProfileId?: string;
      signerName?: string;
      signerMode?: 'software' | 'sgx' | 'nitro' | 'custom';
      signerEndpoint?: string;
      accountPublicKey?: string;
      accountAddress?: string;
      walletPath?: string;
      unlockMode?: 'manual' | 'interactive-passphrase' | 'recipient-attestation';
    };
    import?: {
      importedAt?: number;
      ownershipMode?: ImportedNodeOwnershipMode;
      sourcePath?: string;
      attachedProcessId?: number;
    };
  };
}

export function useNodes() {
  return useQuery({
    queryKey: ['nodes'],
    queryFn: async (): Promise<Node[]> => {
      const data = await api.get<{ nodes: Node[] }>('/nodes');
      return data.nodes;
    },
  });
}

export function useNode(id: string) {
  return useQuery({
    queryKey: ['nodes', id],
    queryFn: async (): Promise<Node> => {
      const data = await api.get<{ node: Node }>(`/nodes/${id}`);
      return data.node;
    },
    enabled: !!id,
  });
}

export function useCreateNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (nodeData: Partial<Node>): Promise<Node> => {
      const data = await api.post<{ node: Node }>('/nodes', nodeData as Record<string, unknown>);
      return data.node;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useUpdateNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      id,
      payload,
    }: {
      id: string;
      payload: Partial<Node>;
    }): Promise<Node> => {
      const data = await api.put<{ node: Node }>(`/nodes/${id}`, payload as Record<string, unknown>);
      return data.node;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['nodes', variables.id] });
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useStartNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string): Promise<Node> => {
      const data = await api.post<{ node: Node }>(`/nodes/${id}/start`);
      return data.node;
    },
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ['nodes', id] });
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useStopNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, force = false }: { id: string; force?: boolean }): Promise<Node> => {
      const data = await api.post<{ node: Node }>(`/nodes/${id}/stop`, { force });
      return data.node;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['nodes', variables.id] });
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useUpdateNodeOwnership() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, ownershipMode }: { id: string; ownershipMode: ImportedNodeOwnershipMode }) => {
      const result = await api.post<{ node: Node }>(`/nodes/${id}/ownership`, { ownershipMode });
      return result.node;
    },
    onSuccess: (_node, variables) => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
      queryClient.invalidateQueries({ queryKey: ['node', variables.id] });
    },
  });
}

export function useDeleteNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string): Promise<void> => {
      await api.delete(`/nodes/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useNodeLogs(id: string, count = 100) {
  return useQuery({
    queryKey: ['nodes', id, 'logs', count],
    queryFn: async (): Promise<Array<{ timestamp: number; level: string; message: string }>> => {
      const data = await api.get<{ logs: Array<{ timestamp: number; level: string; message: string }> }>(
        `/nodes/${id}/logs?count=${count}`,
      );
      return data.logs;
    },
    enabled: !!id,
    refetchInterval: REFETCH_INTERVALS.nodeDetail,
  });
}

export function useNodeSignerHealth(id: string) {
  return useQuery({
    queryKey: ['nodes', id, 'signer-health'],
    queryFn: async () => {
      const data = await api.get<{
        signerHealth: {
          nodeId: string;
          profile: {
            id: string;
            name: string;
            mode: 'software' | 'sgx' | 'nitro' | 'custom' | string;
            endpoint: string;
          };
          readiness: {
            ok: boolean;
            status: 'reachable' | 'unreachable' | 'warning';
            message: string;
            source: 'probe' | 'secure-sign-tools' | 'vsock-format' | string;
            accountStatus?: string;
            checkedAt: number;
          };
        } | null;
      }>(`/nodes/${id}/signer-health`);
      return data.signerHealth;
    },
    enabled: !!id,
    refetchInterval: REFETCH_INTERVALS.signerHealth,
  });
}

export interface SystemMetrics {
  cpu: { usage: number; cores: number };
  memory: { percentage: number; used: number; total: number };
  disk: { percentage: number; used: number; total: number };
  timestamp?: number;
}

export function useSystemMetrics() {
  return useQuery({
    queryKey: ['metrics', 'system'],
    queryFn: async (): Promise<SystemMetrics> => {
      const data = await api.get<{ metrics: SystemMetrics }>('/metrics/system');
      return data.metrics;
    },
    refetchInterval: REFETCH_INTERVALS.dashboard,
  });
}

export function useNetworkHeight() {
  return useQuery({
    queryKey: ['metrics', 'network'],
    queryFn: async () => {
      const data = await api.get<{ mainnet: number | null; testnet: number | null; timestamp: number }>('/metrics/network');
      return data;
    },
    refetchInterval: REFETCH_INTERVALS.dashboard,
  });
}
