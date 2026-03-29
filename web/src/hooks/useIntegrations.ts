import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api } from '../utils/api';

export type IntegrationCategory = 'metrics' | 'logging' | 'uptime' | 'alerting' | 'errors';

export interface ConfigField {
  key: string;
  label: string;
  type: 'text' | 'password' | 'url';
  placeholder: string;
  required: boolean;
}

export interface IntegrationStatus {
  id: string;
  name: string;
  description: string;
  category: IntegrationCategory;
  enabled: boolean;
  configured: boolean;
  configSchema: ConfigField[];
  configValues: Record<string, string>;
  lastTestAt: string | null;
  lastError: string | null;
}

export function useIntegrations() {
  return useQuery({
    queryKey: ['integrations'],
    queryFn: async () => {
      const response = await api.get<{ integrations: IntegrationStatus[] }>('/integrations');
      return response.integrations;
    },
  });
}

export function useIntegration(id: string) {
  return useQuery({
    queryKey: ['integrations', id],
    queryFn: async () => {
      const response = await api.get<{ integration: IntegrationStatus }>(`/integrations/${id}`);
      return response.integration;
    },
    enabled: !!id,
  });
}

export function useSaveIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, config, enabled }: { id: string; config: Record<string, string>; enabled: boolean }) => {
      const response = await api.put<{ integration: IntegrationStatus }>(`/integrations/${id}`, { config, enabled } as unknown as Record<string, unknown>);
      return response.integration;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}

export function useTestIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      const response = await api.post<{ success: boolean; error?: string }>(`/integrations/${id}/test`);
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}

export function useDeleteIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/integrations/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}
