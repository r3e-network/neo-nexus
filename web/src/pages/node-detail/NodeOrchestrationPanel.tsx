import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  ClipboardList,
  Database,
  History,
  Layers3,
  Loader2,
  Plus,
  RefreshCw,
  Save,
  Sparkles,
  Zap,
} from "lucide-react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import {
  useActivateNodeDataContext,
  useApplyNodeRole,
  useCreateCustomRole,
  useCreateNodeDataContext,
  useDownloadFastSyncSnapshot,
  useFastSyncSnapshots,
  useNodeDataContexts,
  useNodeRoleApplications,
  useNodeRolePlan,
  useNodeRoles,
  useRegisterFastSyncSnapshot,
  useVerifyFastSyncSnapshot,
  type FastSyncSourceType,
  type RoleSyncStrategy,
  type StorageEngine,
} from "../../hooks/useNodeOrchestration";
import type { Node } from "../../hooks/useNodes";
import { ApiRequestError } from "../../utils/api";
import { formatBytes } from "../../utils/format";
import {
  ROLE_PLUGIN_OPTIONS,
  STORAGE_ENGINE_OPTIONS,
  SYNC_STRATEGY_OPTIONS,
  compatibleSnapshots,
  currentStorageEngine,
  currentSyncStrategy,
  defaultRoleLabelTemplate,
  formatStorageEngine,
  formatSyncStrategy,
  planTone,
  roleStorageEngine,
  roleSupportsNode,
  summarizeRole,
} from "../../utils/orchestration";

interface NodeOrchestrationPanelProps {
  node: Node;
}

interface BannerState {
  error?: string;
  suggestion?: string;
  code?: string;
  success?: string;
}

function apiBanner(error: unknown, fallback: string): BannerState {
  if (error instanceof ApiRequestError) {
    return { error: error.message, suggestion: error.suggestion, code: error.code };
  }
  return { error: error instanceof Error ? error.message : fallback };
}

function timestamp(value?: number) {
  return value ? new Date(value).toLocaleString() : "Not recorded";
}

function parsePositiveInteger(value: string, label: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const parsed = Number(trimmed);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${label} must be a positive integer`);
  }
  return parsed;
}

export function NodeOrchestrationPanel({ node }: NodeOrchestrationPanelProps) {
  const stopped = node.process.status === "stopped";
  const rolesQuery = useNodeRoles();
  const contextsQuery = useNodeDataContexts(node.id);
  const applicationsQuery = useNodeRoleApplications(node.id);
  const snapshotsQuery = useFastSyncSnapshots();
  const applyRole = useApplyNodeRole();
  const createCustomRole = useCreateCustomRole();
  const createContext = useCreateNodeDataContext(node.id);
  const activateContext = useActivateNodeDataContext(node.id);
  const registerSnapshot = useRegisterFastSyncSnapshot();
  const verifySnapshot = useVerifyFastSyncSnapshot();
  const downloadSnapshot = useDownloadFastSyncSnapshot();

  const roles = rolesQuery.data ?? [];
  const compatibleRoles = useMemo(
    () => roles.filter((role) => roleSupportsNode(role, node.type)),
    [node.type, roles],
  );
  const selectedDefaultRole = compatibleRoles.find((role) => role.id === node.settings?.role?.id) ?? compatibleRoles[0];
  const [selectedRoleId, setSelectedRoleId] = useState(selectedDefaultRole?.id ?? "");
  const selectedRole = compatibleRoles.find((role) => role.id === selectedRoleId) ?? null;
  const [roleStorage, setRoleStorage] = useState<StorageEngine>(() =>
    selectedDefaultRole ? roleStorageEngine(selectedDefaultRole, currentStorageEngine(node)) : currentStorageEngine(node));
  const rolePlanQuery = useNodeRolePlan(selectedRole?.id, node.id, roleStorage);
  const [applyDialogOpen, setApplyDialogOpen] = useState(false);
  const [banner, setBanner] = useState<BannerState>({});
  const [showCustomRole, setShowCustomRole] = useState(false);
  const [customPluginIds, setCustomPluginIds] = useState<string[]>([]);
  const [customRoleName, setCustomRoleName] = useState("");
  const [customRoleDescription, setCustomRoleDescription] = useState("");
  const [customRoleStorage, setCustomRoleStorage] = useState<StorageEngine>(currentStorageEngine(node));
  const [customRoleSync, setCustomRoleSync] = useState<RoleSyncStrategy>(currentSyncStrategy(node));
  const [customRoleLabel, setCustomRoleLabel] = useState("custom-{network}-{storageEngine}");
  const [customRoleRelay, setCustomRoleRelay] = useState(true);

  const [contextLabel, setContextLabel] = useState(`${node.network}-${currentStorageEngine(node)}`);
  const [contextStorage, setContextStorage] = useState<StorageEngine>(currentStorageEngine(node));
  const [contextSync, setContextSync] = useState<RoleSyncStrategy>(currentSyncStrategy(node));
  const [checkpointHeight, setCheckpointHeight] = useState("");
  const [checkpointHash, setCheckpointHash] = useState("");
  const [contextSnapshotId, setContextSnapshotId] = useState("");

  const [showSnapshotForm, setShowSnapshotForm] = useState(false);
  const [snapshotName, setSnapshotName] = useState(`${node.name} snapshot`);
  const [snapshotSourceType, setSnapshotSourceType] = useState<FastSyncSourceType>("local");
  const [snapshotSource, setSnapshotSource] = useState("");
  const [snapshotHeight, setSnapshotHeight] = useState("");
  const [snapshotSha256, setSnapshotSha256] = useState("");
  const [snapshotBlockHash, setSnapshotBlockHash] = useState("");
  const [snapshotStorage, setSnapshotStorage] = useState<StorageEngine>(currentStorageEngine(node));

  useEffect(() => {
    if (selectedRoleId || !selectedDefaultRole) {
      return;
    }
    setSelectedRoleId(selectedDefaultRole.id);
    setRoleStorage(roleStorageEngine(selectedDefaultRole, currentStorageEngine(node)));
  }, [node, selectedDefaultRole, selectedRoleId]);

  const snapshots = snapshotsQuery.data ?? [];
  const matchingSnapshots = compatibleSnapshots(snapshots, node, contextStorage);
  const currentPlan = rolePlanQuery.data;
  const planState = planTone(currentPlan);
  const applications = applicationsQuery.data ?? [];
  const contexts = contextsQuery.data?.contexts ?? [];
  const activeContext = contextsQuery.data?.activeContext ?? null;
  const canUsePlugins = node.type === "neo-cli";

  const handleRoleChange = (roleId: string) => {
    const role = compatibleRoles.find((candidate) => candidate.id === roleId);
    setSelectedRoleId(roleId);
    setRoleStorage(role ? roleStorageEngine(role, currentStorageEngine(node)) : currentStorageEngine(node));
    setBanner({});
  };

  const handleApplyRole = async () => {
    if (!selectedRole) return;
    setBanner({});
    try {
      await applyRole.mutateAsync({
        roleId: selectedRole.id,
        nodeId: node.id,
        storageEngine: roleStorage,
      });
      setApplyDialogOpen(false);
      setBanner({ success: `${selectedRole.name} was applied to ${node.name}.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to apply node role."));
    }
  };

  const handleCreateCustomRole = async () => {
    setBanner({});
    if (!customRoleName.trim()) {
      setBanner({ error: "Custom role name is required." });
      return;
    }
    try {
      const role = await createCustomRole.mutateAsync({
        name: customRoleName.trim(),
        description: customRoleDescription.trim() || undefined,
        nodeTypes: [node.type],
        profile: {
          storageEngine: customRoleStorage,
          settings: { relay: customRoleRelay },
          ...(canUsePlugins && customPluginIds.length > 0
            ? { plugins: customPluginIds.map((id) => ({ id, enabled: true, config: {} })) }
            : {}),
          dataContext: { mode: "reuse-or-create", labelTemplate: customRoleLabel.trim() || defaultRoleLabelTemplate({ id: "custom", name: customRoleName, kind: "custom", nodeTypes: [node.type], profile: {}, createdAt: 0, updatedAt: 0 }) },
          sync: { strategy: customRoleSync, allowCheckpoint: customRoleSync === "fast-sync" },
        },
      });
      setSelectedRoleId(role.id);
      setRoleStorage(role.profile.storageEngine ?? currentStorageEngine(node));
      setShowCustomRole(false);
      setCustomRoleName("");
      setCustomRoleDescription("");
      setCustomPluginIds([]);
      setBanner({ success: `Custom role ${role.name} was saved and selected.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to create custom role."));
    }
  };

  const handleCreateContext = async () => {
    setBanner({});
    try {
      const height = parsePositiveInteger(checkpointHeight, "Checkpoint height");
      const context = await createContext.mutateAsync({
        label: contextLabel.trim() || `${node.network}-${contextStorage}`,
        storageEngine: contextStorage,
        syncStrategy: contextSync,
        ...(height ? { checkpointHeight: height } : {}),
        ...(checkpointHash.trim() ? { checkpointHash: checkpointHash.trim() } : {}),
        ...(contextSnapshotId ? { snapshotId: contextSnapshotId } : {}),
      });
      setContextLabel(`${node.network}-${contextStorage}`);
      setCheckpointHeight("");
      setCheckpointHash("");
      setContextSnapshotId("");
      setBanner({ success: `Data context ${context.label} was created${context.active ? " and activated" : ""}.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to create data context."));
    }
  };

  const handleActivateContext = async (contextId: string) => {
    setBanner({});
    try {
      const result = await activateContext.mutateAsync(contextId);
      setBanner({ success: `Data context ${result.context.label} is now active.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to activate data context."));
    }
  };

  const handleRegisterSnapshot = async () => {
    setBanner({});
    try {
      const height = parsePositiveInteger(snapshotHeight, "Snapshot height");
      if (!height) {
        setBanner({ error: "Snapshot height is required." });
        return;
      }
      const snapshot = await registerSnapshot.mutateAsync({
        name: snapshotName.trim() || `${node.network} ${node.type} snapshot`,
        sourceType: snapshotSourceType,
        source: snapshotSource.trim(),
        chain: node.chain ?? (node.type === "neox-go" ? "x" : "n3"),
        network: node.network,
        nodeType: node.type,
        storageEngine: snapshotStorage,
        height,
        sha256: snapshotSha256.trim(),
        ...(snapshotBlockHash.trim() ? { blockHash: snapshotBlockHash.trim() } : {}),
      });
      setSnapshotName(`${node.name} snapshot`);
      setSnapshotSource("");
      setSnapshotHeight("");
      setSnapshotSha256("");
      setSnapshotBlockHash("");
      setShowSnapshotForm(false);
      setContextSnapshotId(snapshot.id);
      setContextStorage(snapshot.storageEngine);
      setBanner({ success: `Snapshot manifest ${snapshot.name} was registered and selected for new contexts.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to register snapshot manifest."));
    }
  };

  const handleVerifySnapshot = async (snapshotId: string) => {
    setBanner({});
    try {
      const snapshot = await verifySnapshot.mutateAsync(snapshotId);
      setBanner({ success: `Snapshot ${snapshot.name} verified at height ${snapshot.height.toLocaleString()}.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to verify snapshot."));
    }
  };

  const handleDownloadSnapshot = async (snapshotId: string) => {
    setBanner({});
    try {
      const snapshot = await downloadSnapshot.mutateAsync(snapshotId);
      setBanner({ success: `Snapshot ${snapshot.name} was downloaded and verified at height ${snapshot.height.toLocaleString()}.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to download snapshot."));
    }
  };

  return (
    <section className="card space-y-5">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <p className="console-kicker">Role orchestration</p>
          <h2 className="mt-1 text-lg font-semibold text-slate-950">One-click node identity and isolated state</h2>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
            Apply a saved role, switch storage, isolate blockchain data contexts, and bind fast-sync checkpoints without editing native configs by hand.
          </p>
        </div>
        <span className={`inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium ${
          stopped ? "border-emerald-200 bg-emerald-50 text-emerald-800" : "border-amber-200 bg-amber-50 text-amber-800"
        }`}>
          {stopped ? <CheckCircle2 className="h-4 w-4" /> : <AlertTriangle className="h-4 w-4" />}
          {stopped ? "Ready for configuration" : `Stop node first (${node.process.status})`}
        </span>
      </div>

      <FeedbackBanner {...banner} />

      <div className="grid gap-3 md:grid-cols-4">
        <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
          <p className="text-xs font-medium uppercase text-slate-500">Role</p>
          <p className="mt-1 text-sm font-semibold text-slate-950">{node.settings?.role?.name ?? "Unassigned"}</p>
        </div>
        <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
          <p className="text-xs font-medium uppercase text-slate-500">Storage</p>
          <p className="mt-1 text-sm font-semibold text-slate-950">{formatStorageEngine(currentStorageEngine(node))}</p>
        </div>
        <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
          <p className="text-xs font-medium uppercase text-slate-500">Sync</p>
          <p className="mt-1 text-sm font-semibold text-slate-950">{formatSyncStrategy(currentSyncStrategy(node))}</p>
        </div>
        <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
          <p className="text-xs font-medium uppercase text-slate-500">Data context</p>
          <p className="mt-1 truncate text-sm font-semibold text-slate-950">{activeContext?.label ?? node.settings?.activeDataContextId ?? "Default path"}</p>
        </div>
      </div>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
        <div className="space-y-4">
          <div className="rounded-lg border border-slate-200 bg-white p-4">
            <div className="mb-4 flex items-start justify-between gap-3">
              <div>
                <div className="flex items-center gap-2">
                  <Sparkles className="h-5 w-5 text-teal-700" />
                  <h3 className="font-semibold text-slate-950">Role switcher</h3>
                </div>
                <p className="mt-1 text-sm text-slate-600">Roles remember plugins, config, storage, sync strategy, and data context labels.</p>
              </div>
              <button type="button" onClick={() => setShowCustomRole((value) => !value)} className="btn btn-secondary">
                <Plus className="h-4 w-4" />
                Custom
              </button>
            </div>

            <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_180px]">
              <div>
                <label className="mb-2 block text-xs font-medium text-slate-600">Role</label>
                <select
                  className="input"
                  value={selectedRoleId}
                  onChange={(event) => handleRoleChange(event.target.value)}
                  disabled={rolesQuery.isLoading || compatibleRoles.length === 0}
                >
                  {compatibleRoles.length === 0 ? (
                    <option value="">No compatible roles</option>
                  ) : (
                    compatibleRoles.map((role) => (
                      <option key={role.id} value={role.id}>
                        {role.name} · {role.kind}
                      </option>
                    ))
                  )}
                </select>
              </div>
              <div>
                <label className="mb-2 block text-xs font-medium text-slate-600">Storage override</label>
                <select className="input" value={roleStorage} onChange={(event) => setRoleStorage(event.target.value as StorageEngine)}>
                  {STORAGE_ENGINE_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>{option.label}</option>
                  ))}
                </select>
              </div>
            </div>

            {selectedRole && (
              <div className="mt-4 rounded-lg border border-slate-200 bg-slate-50 p-3">
                <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div>
                    <p className="text-sm font-semibold text-slate-950">{selectedRole.name}</p>
                    <p className="mt-1 text-sm leading-5 text-slate-600">{selectedRole.description ?? summarizeRole(selectedRole)}</p>
                    <p className="mt-2 text-xs text-slate-500">{summarizeRole(selectedRole)}</p>
                  </div>
                  <button
                    type="button"
                    disabled={!stopped || !selectedRole || applyRole.isPending || rolePlanQuery.isError}
                    onClick={() => setApplyDialogOpen(true)}
                    className="btn btn-primary justify-center"
                  >
                    {applyRole.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Zap className="h-4 w-4" />}
                    Apply role
                  </button>
                </div>

                <div className={`mt-4 rounded-lg border p-3 ${
                  planState === "warning"
                    ? "border-amber-200 bg-amber-50"
                    : planState === "success"
                      ? "border-emerald-200 bg-emerald-50"
                      : "border-slate-200 bg-white"
                }`}>
                  <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-950">
                    <ClipboardList className="h-4 w-4" />
                    Plan preview
                    {rolePlanQuery.isFetching && <Loader2 className="h-4 w-4 animate-spin text-slate-500" />}
                  </div>
                  {rolePlanQuery.isError ? (
                    <p className="text-sm text-red-700">{rolePlanQuery.error instanceof Error ? rolePlanQuery.error.message : "Unable to preview role plan."}</p>
                  ) : currentPlan ? (
                    <>
                      {currentPlan.changes.length === 0 ? (
                        <p className="text-sm text-emerald-800">This node already matches the selected role.</p>
                      ) : (
                        <ul className="space-y-1 text-sm text-slate-700">
                          {currentPlan.changes.map((change) => (
                            <li key={`${change.type}-${change.summary}`} className="flex gap-2">
                              <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-teal-600" />
                              <span>{change.summary}</span>
                            </li>
                          ))}
                        </ul>
                      )}
                      {currentPlan.warnings.length > 0 && (
                        <div className="mt-3 space-y-1 text-sm text-amber-800">
                          {currentPlan.warnings.map((warning) => (
                            <p key={warning} className="flex gap-2">
                              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
                              <span>{warning}</span>
                            </p>
                          ))}
                        </div>
                      )}
                    </>
                  ) : (
                    <p className="text-sm text-slate-600">Select a compatible role to preview changes.</p>
                  )}
                </div>
              </div>
            )}

            {showCustomRole && (
              <div className="mt-4 rounded-lg border border-teal-200 bg-teal-50 p-4">
                <div className="mb-3">
                  <h4 className="font-semibold text-teal-950">Save custom identity</h4>
                  <p className="mt-1 text-sm text-teal-900">Create a reusable role for this node type. It can be applied later like a built-in preset.</p>
                </div>
                <div className="grid gap-3 lg:grid-cols-2">
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Name</label>
                    <input className="input" value={customRoleName} onChange={(event) => setCustomRoleName(event.target.value)} placeholder="Archive indexer" />
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Data label template</label>
                    <input className="input" value={customRoleLabel} onChange={(event) => setCustomRoleLabel(event.target.value)} placeholder="archive-{network}-{storageEngine}" />
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Storage</label>
                    <select className="input" value={customRoleStorage} onChange={(event) => setCustomRoleStorage(event.target.value as StorageEngine)}>
                      {STORAGE_ENGINE_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Sync strategy</label>
                    <select className="input" value={customRoleSync} onChange={(event) => setCustomRoleSync(event.target.value as RoleSyncStrategy)}>
                      {SYNC_STRATEGY_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                    </select>
                  </div>
                  <div className="lg:col-span-2">
                    <label className="mb-2 block text-xs font-medium text-slate-700">Description</label>
                    <textarea className="input min-h-20" value={customRoleDescription} onChange={(event) => setCustomRoleDescription(event.target.value)} placeholder="What this role configures and when to use it" />
                  </div>
                </div>
                {canUsePlugins && (
                  <div className="mt-3">
                    <p className="mb-2 text-xs font-medium text-slate-700">Neo CLI plugins</p>
                    <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                      {ROLE_PLUGIN_OPTIONS.map((plugin) => {
                        const checked = customPluginIds.includes(plugin.id);
                        return (
                          <label key={plugin.id} className={`flex items-center gap-2 rounded-lg border px-3 py-2 text-sm ${checked ? "border-teal-300 bg-white text-teal-950" : "border-teal-100 bg-white/70 text-slate-700"}`}>
                            <input
                              type="checkbox"
                              checked={checked}
                              onChange={(event) => {
                                setCustomPluginIds((current) =>
                                  event.target.checked ? [...current, plugin.id] : current.filter((id) => id !== plugin.id));
                              }}
                            />
                            {plugin.label}
                          </label>
                        );
                      })}
                    </div>
                  </div>
                )}
                <label className="mt-3 flex items-center justify-between rounded-lg border border-teal-100 bg-white/70 px-3 py-2">
                  <span>
                    <span className="block text-sm font-medium text-slate-950">Relay transactions</span>
                    <span className="text-xs text-slate-600">Include relay=true in role settings.</span>
                  </span>
                  <input type="checkbox" checked={customRoleRelay} onChange={(event) => setCustomRoleRelay(event.target.checked)} />
                </label>
                <div className="mt-4 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                  <button type="button" className="btn btn-secondary justify-center" onClick={() => setShowCustomRole(false)}>Cancel</button>
                  <button type="button" className="btn btn-primary justify-center" onClick={() => void handleCreateCustomRole()} disabled={createCustomRole.isPending}>
                    {createCustomRole.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
                    Save role
                  </button>
                </div>
              </div>
            )}
          </div>

          <div className="rounded-lg border border-slate-200 bg-white p-4">
            <div className="mb-4 flex items-center gap-2">
              <History className="h-5 w-5 text-slate-600" />
              <h3 className="font-semibold text-slate-950">Role history</h3>
            </div>
            {applications.length === 0 ? (
              <p className="text-sm text-slate-600">No role applications have been recorded for this node yet.</p>
            ) : (
              <div className="space-y-2">
                {applications.slice(0, 4).map((application) => (
                  <div key={application.id} className="rounded-lg border border-slate-200 bg-slate-50 p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <p className="text-sm font-medium text-slate-950">{application.roleName}</p>
                      <span className={`status-badge status-${application.status === "failed" ? "error" : "running"}`}>{application.status}</span>
                    </div>
                    <p className="mt-1 text-xs text-slate-500">{timestamp(application.appliedAt)}</p>
                    {application.errorMessage && <p className="mt-2 text-sm text-red-700">{application.errorMessage}</p>}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-4">
          <div className="rounded-lg border border-slate-200 bg-white p-4">
            <div className="mb-4 flex items-center gap-2">
              <Layers3 className="h-5 w-5 text-teal-700" />
              <h3 className="font-semibold text-slate-950">Data contexts</h3>
            </div>
            <div className="space-y-2">
              {contexts.length === 0 ? (
                <p className="text-sm text-slate-600">No isolated contexts yet. Create one to pin storage, sync strategy, checkpoint, or snapshot metadata.</p>
              ) : (
                contexts.map((context) => (
                  <div key={context.id} className={`rounded-lg border p-3 ${context.active ? "border-teal-300 bg-teal-50" : "border-slate-200 bg-slate-50"}`}>
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="text-sm font-semibold text-slate-950">{context.label}</p>
                          {context.active && <span className="rounded-full bg-teal-700 px-2 py-0.5 text-xs font-medium text-white">Active</span>}
                        </div>
                        <p className="mt-1 text-xs text-slate-600">
                          {formatStorageEngine(context.storageEngine)} · {formatSyncStrategy(context.syncStrategy)}
                          {context.checkpointHeight ? ` · checkpoint ${context.checkpointHeight.toLocaleString()}` : ""}
                        </p>
                        {context.snapshotId && <p className="mt-1 text-xs text-slate-500">Snapshot {context.snapshotId}</p>}
                      </div>
                      <button
                        type="button"
                        className="btn btn-secondary justify-center"
                        disabled={context.active || !stopped || activateContext.isPending}
                        onClick={() => void handleActivateContext(context.id)}
                      >
                        Activate
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>

            <div className="mt-4 rounded-lg border border-slate-200 bg-slate-50 p-3">
              <h4 className="text-sm font-semibold text-slate-950">Create isolated context</h4>
              <div className="mt-3 grid gap-3 sm:grid-cols-2">
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Label</label>
                  <input className="input" value={contextLabel} onChange={(event) => setContextLabel(event.target.value)} />
                </div>
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Snapshot</label>
                  <select className="input" value={contextSnapshotId} onChange={(event) => setContextSnapshotId(event.target.value)}>
                    <option value="">No snapshot</option>
                    {matchingSnapshots.map((snapshot) => (
                      <option key={snapshot.id} value={snapshot.id}>
                        {snapshot.name} · {snapshot.height.toLocaleString()}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Storage</label>
                  <select className="input" value={contextStorage} onChange={(event) => setContextStorage(event.target.value as StorageEngine)}>
                    {STORAGE_ENGINE_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                  </select>
                </div>
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Sync</label>
                  <select className="input" value={contextSync} onChange={(event) => setContextSync(event.target.value as RoleSyncStrategy)}>
                    {SYNC_STRATEGY_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                  </select>
                </div>
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Checkpoint height</label>
                  <input className="input" inputMode="numeric" value={checkpointHeight} onChange={(event) => setCheckpointHeight(event.target.value)} placeholder="Optional" />
                </div>
                <div>
                  <label className="mb-2 block text-xs font-medium text-slate-600">Checkpoint hash</label>
                  <input className="input font-mono text-xs" value={checkpointHash} onChange={(event) => setCheckpointHash(event.target.value)} placeholder="Optional" />
                </div>
              </div>
              <button type="button" className="btn btn-primary mt-3 justify-center" disabled={!stopped || createContext.isPending} onClick={() => void handleCreateContext()}>
                {createContext.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Plus className="h-4 w-4" />}
                Create context
              </button>
            </div>
          </div>

          <div className="rounded-lg border border-slate-200 bg-white p-4">
            <div className="mb-4 flex items-start justify-between gap-3">
              <div>
                <div className="flex items-center gap-2">
                  <Database className="h-5 w-5 text-blue-700" />
                  <h3 className="font-semibold text-slate-950">Fast sync snapshots</h3>
                </div>
                <p className="mt-1 text-sm text-slate-600">Register manifests, verify local archives, then attach them to data contexts.</p>
              </div>
              <button type="button" className="btn btn-secondary" onClick={() => setShowSnapshotForm((value) => !value)}>
                <Plus className="h-4 w-4" />
                Manifest
              </button>
            </div>

            {matchingSnapshots.length === 0 ? (
              <p className="text-sm text-slate-600">No compatible snapshots for {node.network} · {node.type} · {formatStorageEngine(contextStorage)}.</p>
            ) : (
              <div className="space-y-2">
                {matchingSnapshots.slice(0, 4).map((snapshot) => (
                  <div key={snapshot.id} className="rounded-lg border border-slate-200 bg-slate-50 p-3">
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                      <div>
                        <p className="text-sm font-semibold text-slate-950">{snapshot.name}</p>
                        <p className="mt-1 text-xs text-slate-600">
                          {snapshot.sourceType} · height {snapshot.height.toLocaleString()}
                          {snapshot.sizeBytes ? ` · ${formatBytes(snapshot.sizeBytes)}` : ""}
                        </p>
                        <p className="mt-1 text-xs text-slate-500">Verified: {timestamp(snapshot.lastVerifiedAt)}</p>
                      </div>
                      {snapshot.sourceType === "local" ? (
                        <button
                          type="button"
                          className="btn btn-secondary justify-center"
                          disabled={verifySnapshot.isPending}
                          onClick={() => void handleVerifySnapshot(snapshot.id)}
                        >
                          <RefreshCw className="h-4 w-4" />
                          Verify
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="btn btn-secondary justify-center"
                          disabled={downloadSnapshot.isPending}
                          onClick={() => void handleDownloadSnapshot(snapshot.id)}
                        >
                          <RefreshCw className="h-4 w-4" />
                          Download
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {showSnapshotForm && (
              <div className="mt-4 rounded-lg border border-blue-200 bg-blue-50 p-3">
                <div className="grid gap-3 sm:grid-cols-2">
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Name</label>
                    <input className="input" value={snapshotName} onChange={(event) => setSnapshotName(event.target.value)} />
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Source type</label>
                    <select className="input" value={snapshotSourceType} onChange={(event) => setSnapshotSourceType(event.target.value as FastSyncSourceType)}>
                      <option value="local">Local file</option>
                      <option value="url">URL</option>
                      <option value="catalog">Catalog</option>
                    </select>
                  </div>
                  <div className="sm:col-span-2">
                    <label className="mb-2 block text-xs font-medium text-slate-700">Source</label>
                    <input className="input font-mono text-xs" value={snapshotSource} onChange={(event) => setSnapshotSource(event.target.value)} placeholder="/var/snapshots/mainnet.tar.zst or https://..." />
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Height</label>
                    <input className="input" inputMode="numeric" value={snapshotHeight} onChange={(event) => setSnapshotHeight(event.target.value)} />
                  </div>
                  <div>
                    <label className="mb-2 block text-xs font-medium text-slate-700">Storage</label>
                    <select className="input" value={snapshotStorage} onChange={(event) => setSnapshotStorage(event.target.value as StorageEngine)}>
                      {STORAGE_ENGINE_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                    </select>
                  </div>
                  <div className="sm:col-span-2">
                    <label className="mb-2 block text-xs font-medium text-slate-700">SHA-256</label>
                    <input className="input font-mono text-xs" value={snapshotSha256} onChange={(event) => setSnapshotSha256(event.target.value)} />
                  </div>
                  <div className="sm:col-span-2">
                    <label className="mb-2 block text-xs font-medium text-slate-700">Block hash</label>
                    <input className="input font-mono text-xs" value={snapshotBlockHash} onChange={(event) => setSnapshotBlockHash(event.target.value)} placeholder="Optional" />
                  </div>
                </div>
                <button type="button" className="btn btn-primary mt-3 justify-center" disabled={registerSnapshot.isPending} onClick={() => void handleRegisterSnapshot()}>
                  {registerSnapshot.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
                  Register manifest
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={applyDialogOpen}
        title="Apply node role?"
        description={
          <div className="space-y-2">
            <p>This will write plugin/config/storage/data-context changes for {node.name}. The node must remain stopped while the role is applied.</p>
            {currentPlan && currentPlan.changes.length > 0 && (
              <p>{currentPlan.changes.length} change{currentPlan.changes.length === 1 ? "" : "s"} will be applied.</p>
            )}
          </div>
        }
        confirmLabel="Apply role"
        confirmVariant="primary"
        isConfirming={applyRole.isPending}
        onCancel={() => setApplyDialogOpen(false)}
        onConfirm={() => void handleApplyRole()}
      />
    </section>
  );
}
