import { useQuery } from '@tanstack/react-query';
import { api } from '../utils/api';

export interface AuditEntry {
  id: number;
  action: string;
  resourceType: string;
  resourceId: string;
  userId?: string;
  timestamp: number;
}

export function useAuditLog(limit = 50, offset = 0) {
  return useQuery({
    queryKey: ['audit-log', limit, offset],
    queryFn: async (): Promise<AuditEntry[]> => {
      const data = await api.get<{ entries: AuditEntry[] }>(
        `/system/audit-log?limit=${limit}&offset=${offset}`,
      );
      return data.entries;
    },
  });
}
