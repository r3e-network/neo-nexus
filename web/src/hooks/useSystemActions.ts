import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";
import type { ConfigurationSnapshot } from "../../../src/types";

export interface CleanLogsResult {
  cleanedFiles: number;
  nodesAffected: number;
  maxAgeDays: number;
}

export type ExportConfigurationResult = ConfigurationSnapshot;

export interface StopAllNodesResult {
  stoppedCount: number;
  alreadyStoppedCount: number;
}

export interface ResetAllDataResult extends StopAllNodesResult {
  deletedNodeCount: number;
  removedDirectoryCount: number;
}

export interface RestoreConfigurationResult {
  restoredCount: number;
  skippedCount: number;
  failedCount: number;
}

function triggerJsonDownload(filename: string, payload: unknown) {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export function useCleanLogs() {
  return useMutation<CleanLogsResult, Error, number>({
    mutationFn: async (maxAgeDays) =>
      api.post<CleanLogsResult>("/system/logs/clean", { maxAgeDays }),
  });
}

export function useExportConfiguration() {
  return useMutation({
    mutationFn: async () => {
      const snapshot = await api.get<ExportConfigurationResult>("/system/export");
      const filename = `neonexus-export-${snapshot.generatedAt}.json`;
      triggerJsonDownload(filename, snapshot);
      return snapshot;
    },
  });
}

export function useStopAllNodes() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => api.post<StopAllNodesResult>("/system/nodes/stop-all"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
    },
  });
}

export function useResetAllData() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => api.post<ResetAllDataResult>("/system/reset"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}

export function useRestoreConfiguration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      snapshot,
      replaceExisting,
    }: {
      snapshot: ConfigurationSnapshot;
      replaceExisting: boolean;
    }) =>
      api.post<RestoreConfigurationResult>("/system/restore", {
        snapshot,
        replaceExisting,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}
