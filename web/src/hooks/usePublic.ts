import { useQuery } from '@tanstack/react-query';
import { REFETCH_INTERVALS } from '../config/constants';

const API_BASE = '/api/public';

async function readPublicResponse<T>(path: string, field: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch ${field} (${response.status})`);
  }

  const data = (await response.json()) as Record<string, T>;
  return data[field];
}

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
    queryFn: () => readPublicResponse<SystemStatus>('/status', 'status'),
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicNodes() {
  return useQuery({
    queryKey: ['public', 'nodes'],
    queryFn: () => readPublicResponse<PublicNode[]>('/nodes', 'nodes'),
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicNode(id: string) {
  return useQuery({
    queryKey: ['public', 'nodes', id],
    queryFn: () => readPublicResponse<PublicNode>(`/nodes/${id}`, 'node'),
    enabled: !!id,
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}

export function usePublicSystemMetrics() {
  return useQuery({
    queryKey: ['public', 'metrics', 'system'],
    queryFn: () => readPublicResponse<PublicSystemMetrics>('/metrics/system', 'metrics'),
    refetchInterval: REFETCH_INTERVALS.publicDashboard,
  });
}
