import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";
import type { Node } from "./useNodes";

export type StorageEngine = "leveldb" | "rocksdb";
export type RoleSyncStrategy = "full" | "light" | "fast-sync";
export type NodeRoleKind = "builtin" | "custom";
export type RoleApplicationStatus = "planned" | "applied" | "failed";
export type FastSyncSourceType = "local" | "url" | "catalog";
export type PrivateNetworkTemplate = "single" | "four" | "seven";
export type PrivateNetworkPlanStatus = "draft" | "applied" | "failed";

export interface NodeRolePluginDesiredState {
  id: string;
  enabled: boolean;
  config?: Record<string, unknown>;
}

export interface NodeRoleProfileBody {
  storageEngine?: StorageEngine;
  settings?: Record<string, unknown>;
  plugins?: NodeRolePluginDesiredState[];
  dataContext?: {
    mode: "reuse-or-create" | "always-create";
    labelTemplate: string;
  };
  sync?: {
    strategy: RoleSyncStrategy;
    allowCheckpoint?: boolean;
  };
  warnings?: string[];
  prerequisites?: string[];
}

export interface NodeRoleProfile {
  id: string;
  name: string;
  description?: string;
  kind: NodeRoleKind;
  nodeTypes: Node["type"][];
  profile: NodeRoleProfileBody;
  createdBy?: string;
  createdAt: number;
  updatedAt: number;
}

export interface NodeRoleApplicationPlan {
  nodeId: string;
  roleId: string;
  roleName: string;
  requiresRestart: boolean;
  changes: Array<{
    type: "settings" | "plugin" | "storage" | "data-context" | "fast-sync";
    summary: string;
  }>;
  warnings: string[];
}

export interface NodeRoleApplication {
  id: string;
  nodeId: string;
  roleId: string;
  roleName: string;
  applicationPlan: NodeRoleApplicationPlan;
  previousState?: Record<string, unknown>;
  appliedAt: number;
  appliedBy?: string;
  status: RoleApplicationStatus;
  errorMessage?: string;
}

export interface CreateCustomRoleInput {
  name: string;
  description?: string;
  nodeTypes: Node["type"][];
  profile: NodeRoleProfileBody;
}

export interface NodeDataContext {
  id: string;
  nodeId: string;
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
  active: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface CreateNodeDataContextInput {
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
}

export interface FastSyncSnapshot {
  id: string;
  name: string;
  sourceType: FastSyncSourceType;
  source: string;
  chain: Node["chain"];
  network: Node["network"];
  nodeType: Node["type"];
  storageEngine: StorageEngine;
  height: number;
  blockHash?: string;
  sha256: string;
  sizeBytes?: number;
  signature?: string;
  trusted: boolean;
  createdAt: number;
  lastVerifiedAt?: number;
}

export interface RegisterFastSyncSnapshotInput {
  name: string;
  sourceType: FastSyncSourceType;
  source: string;
  chain: NonNullable<Node["chain"]>;
  network: Node["network"];
  nodeType: Node["type"];
  storageEngine: StorageEngine;
  height: number;
  blockHash?: string;
  sha256: string;
  sizeBytes?: number;
  signature?: string;
}

export interface PrivateNetworkPlanNode {
  name: string;
  type: "neo-cli" | "neo-go";
  roleIds: string[];
  storageEngine: StorageEngine;
  ports: {
    rpc?: number;
    p2p?: number;
    websocket?: number;
    metrics?: number;
  };
  publicKey: string;
  address: string;
}

export interface PrivateNetworkPlan {
  id: string;
  name: string;
  template: PrivateNetworkTemplate;
  networkMagic: number;
  plan: {
    nodes: PrivateNetworkPlanNode[];
    seedList: string[];
    validatorsCount: number;
    standbyCommittee: string[];
  };
  status: PrivateNetworkPlanStatus;
  createdAt: number;
  appliedAt?: number;
}

export interface CreatePrivateNetworkPlanInput {
  name: string;
  template: PrivateNetworkTemplate;
  nodeType: "neo-cli" | "neo-go";
  storageEngine: StorageEngine;
  networkMagic?: number;
  baseRpcPort?: number;
  baseP2pPort?: number;
  baseWebsocketPort?: number;
  baseMetricsPort?: number;
  nodeNamePrefix?: string;
}

function invalidateNodeOrchestration(queryClient: ReturnType<typeof useQueryClient>, nodeId: string) {
  queryClient.invalidateQueries({ queryKey: ["nodes", nodeId] });
  queryClient.invalidateQueries({ queryKey: ["nodes"] });
  queryClient.invalidateQueries({ queryKey: ["node-roles", "applications", nodeId] });
  queryClient.invalidateQueries({ queryKey: ["node-data-contexts", nodeId] });
  queryClient.invalidateQueries({ queryKey: ["plugins", nodeId, "installed"] });
}

export function useNodeRoles() {
  return useQuery({
    queryKey: ["node-roles"],
    queryFn: async () => {
      const response = await api.get<{ roles: NodeRoleProfile[] }>("/node-roles");
      return response.roles;
    },
  });
}

export function useCreateCustomRole() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (input: CreateCustomRoleInput) => {
      const response = await api.post<{ role: NodeRoleProfile }>("/node-roles", input as unknown as Record<string, unknown>);
      return response.role;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["node-roles"] });
    },
  });
}

export function useNodeRolePlan(roleId?: string, nodeId?: string, storageEngine?: StorageEngine) {
  return useQuery({
    queryKey: ["node-roles", "plan", roleId, nodeId, storageEngine ?? ""],
    queryFn: async () => {
      const response = await api.post<{ plan: NodeRoleApplicationPlan }>(`/node-roles/${roleId}/plan`, {
        nodeId,
        ...(storageEngine ? { storageEngine } : {}),
      });
      return response.plan;
    },
    enabled: !!roleId && !!nodeId,
    retry: false,
  });
}

export function useApplyNodeRole() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      roleId,
      nodeId,
      storageEngine,
    }: {
      roleId: string;
      nodeId: string;
      storageEngine?: StorageEngine;
    }) => {
      const response = await api.post<{ node: Node; application: NodeRoleApplication }>(`/node-roles/${roleId}/apply`, {
        nodeId,
        ...(storageEngine ? { storageEngine } : {}),
      });
      return response;
    },
    onSuccess: (_result, variables) => {
      invalidateNodeOrchestration(queryClient, variables.nodeId);
    },
  });
}

export function useNodeRoleApplications(nodeId?: string) {
  return useQuery({
    queryKey: ["node-roles", "applications", nodeId],
    queryFn: async () => {
      const response = await api.get<{ applications: NodeRoleApplication[] }>(`/nodes/${nodeId}/role-applications`);
      return response.applications;
    },
    enabled: !!nodeId,
  });
}

export function useNodeDataContexts(nodeId?: string) {
  return useQuery({
    queryKey: ["node-data-contexts", nodeId],
    queryFn: async () => {
      const response = await api.get<{
        contexts: NodeDataContext[];
        activeContext: NodeDataContext | null;
      }>(`/nodes/${nodeId}/data-contexts`);
      return response;
    },
    enabled: !!nodeId,
  });
}

export function useCreateNodeDataContext(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (input: CreateNodeDataContextInput) => {
      const response = await api.post<{ context: NodeDataContext }>(`/nodes/${nodeId}/data-contexts`, input as unknown as Record<string, unknown>);
      return response.context;
    },
    onSuccess: () => {
      if (nodeId) {
        invalidateNodeOrchestration(queryClient, nodeId);
      }
    },
  });
}

export function useActivateNodeDataContext(nodeId?: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (contextId: string) => {
      const response = await api.post<{ context: NodeDataContext; node: Node }>(`/nodes/${nodeId}/data-contexts/${contextId}/activate`);
      return response;
    },
    onSuccess: () => {
      if (nodeId) {
        invalidateNodeOrchestration(queryClient, nodeId);
      }
    },
  });
}

export function useFastSyncSnapshots() {
  return useQuery({
    queryKey: ["fast-sync", "snapshots"],
    queryFn: async () => {
      const response = await api.get<{ snapshots: FastSyncSnapshot[] }>("/fast-sync/snapshots");
      return response.snapshots;
    },
  });
}

export function useRegisterFastSyncSnapshot() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (input: RegisterFastSyncSnapshotInput) => {
      const response = await api.post<{ snapshot: FastSyncSnapshot }>("/fast-sync/snapshots", input as unknown as Record<string, unknown>);
      return response.snapshot;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["fast-sync", "snapshots"] });
    },
  });
}

export function useVerifyFastSyncSnapshot() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (snapshotId: string) => {
      const response = await api.post<{ snapshot: FastSyncSnapshot }>(`/fast-sync/snapshots/${snapshotId}/verify`);
      return response.snapshot;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["fast-sync", "snapshots"] });
    },
  });
}

export function useDownloadFastSyncSnapshot() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (snapshotId: string) => {
      const response = await api.post<{ snapshot: FastSyncSnapshot }>(`/fast-sync/snapshots/${snapshotId}/download`);
      return response.snapshot;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["fast-sync", "snapshots"] });
    },
  });
}

export function usePrivateNetworkPlans() {
  return useQuery({
    queryKey: ["private-networks", "plans"],
    queryFn: async () => {
      const response = await api.get<{ plans: PrivateNetworkPlan[] }>("/private-networks/plans");
      return response.plans;
    },
  });
}

export function useCreatePrivateNetworkPlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (input: CreatePrivateNetworkPlanInput) => {
      const response = await api.post<{ plan: PrivateNetworkPlan }>("/private-networks/plans", input as unknown as Record<string, unknown>);
      return response.plan;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["private-networks", "plans"] });
    },
  });
}

export function useApplyPrivateNetworkPlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ planId, replaceExisting }: { planId: string; replaceExisting: boolean }) => {
      const response = await api.post<{
        result: { restoredCount: number; skippedCount: number; failedCount: number };
        plan: PrivateNetworkPlan | null;
      }>(`/private-networks/plans/${planId}/apply`, { replaceExisting });
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["private-networks", "plans"] });
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
    },
  });
}
