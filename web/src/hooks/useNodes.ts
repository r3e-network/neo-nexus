import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

const API_BASE = '/api';

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
}

// Helper to get auth headers
function getHeaders() {
  const token = localStorage.getItem('token');
  return {
    'Content-Type': 'application/json',
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

export function useNodes() {
  return useQuery({
    queryKey: ['nodes'],
    queryFn: async (): Promise<Node[]> => {
      const response = await fetch(`${API_BASE}/nodes`, {
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to fetch nodes');
      const data = await response.json();
      return data.nodes;
    },
  });
}

export function useNode(id: string) {
  return useQuery({
    queryKey: ['nodes', id],
    queryFn: async (): Promise<Node> => {
      const response = await fetch(`${API_BASE}/nodes/${id}`, {
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to fetch node');
      const data = await response.json();
      return data.node;
    },
    enabled: !!id,
  });
}

export function useCreateNode() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (nodeData: Partial<Node>): Promise<Node> => {
      const response = await fetch(`${API_BASE}/nodes`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify(nodeData),
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create node');
      }
      const data = await response.json();
      return data.node;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useStartNode() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (id: string): Promise<Node> => {
      const response = await fetch(`${API_BASE}/nodes/${id}/start`, {
        method: 'POST',
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to start node');
      const data = await response.json();
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
      const response = await fetch(`${API_BASE}/nodes/${id}/stop`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ force }),
      });
      if (!response.ok) throw new Error('Failed to stop node');
      const data = await response.json();
      return data.node;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['nodes', variables.id] });
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

export function useDeleteNode() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (id: string): Promise<void> => {
      const response = await fetch(`${API_BASE}/nodes/${id}`, {
        method: 'DELETE',
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to delete node');
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
      const response = await fetch(`${API_BASE}/nodes/${id}/logs?count=${count}`, {
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to fetch logs');
      const data = await response.json();
      return data.logs;
    },
    enabled: !!id,
    refetchInterval: 2000,
  });
}

export function useSystemMetrics() {
  return useQuery({
    queryKey: ['metrics', 'system'],
    queryFn: async () => {
      const response = await fetch(`${API_BASE}/metrics/system`, {
        headers: getHeaders(),
      });
      if (!response.ok) throw new Error('Failed to fetch metrics');
      const data = await response.json();
      return data.metrics;
    },
    refetchInterval: 5000,
  });
}
