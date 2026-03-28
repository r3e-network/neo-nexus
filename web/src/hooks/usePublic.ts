import { useQuery } from '@tanstack/react-query';
import { REFETCH_INTERVALS } from '../config/constants';

const API_BASE = '/api/public';

export interface PublicNode {
  id: string;
  name: string;
  type: 'neo-cli' | 'neo-go';
  network: 'mainnet' | 'testnet' | 'private';
  status: string;
  version: string;
  metrics: {
    blockHeight: number;
    headerHeight: number;
    connectedPeers: number;
    syncProgress: number;
  } | null;
  uptime?: number;
  lastUpdate?: number;
}

export interface SystemStatus {
  totalNodes: number;
  runningNodes: number;
  syncingNodes: number;
  errorNodes: number;
  totalBlocks: number;
  totalPeers: number;
  timestamp: number;
}

export interface PublicSystemMetrics {
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
  timestamp: number;
}

// Public API - no authentication required
export function usePublicStatus() {
  return useQuery({
    queryKey: ['public', 'status'],
    queryFn: async (): Promise<SystemStatus> => {
      const response = await fetch(`${API_BASE}/status`);
      if (!response.ok) throw new Error('Failed to fetch status');
      const data = await response.json();
      return data.status;
    },
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicNodes() {
  return useQuery({
    queryKey: ['public', 'nodes'],
    queryFn: async (): Promise<PublicNode[]> => {
      const response = await fetch(`${API_BASE}/nodes`);
      if (!response.ok) throw new Error('Failed to fetch nodes');
      const data = await response.json();
      return data.nodes;
    },
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicNode(id: string) {
  return useQuery({
    queryKey: ['public', 'nodes', id],
    queryFn: async (): Promise<PublicNode> => {
      const response = await fetch(`${API_BASE}/nodes/${id}`);
      if (!response.ok) throw new Error('Failed to fetch node');
      const data = await response.json();
      return data.node;
    },
    enabled: !!id,
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicSystemMetrics() {
  return useQuery({
    queryKey: ['public', 'metrics', 'system'],
    queryFn: async (): Promise<PublicSystemMetrics> => {
      const response = await fetch(`${API_BASE}/metrics/system`);
      if (!response.ok) throw new Error('Failed to fetch metrics');
      const data = await response.json();
      return data.metrics;
    },
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

