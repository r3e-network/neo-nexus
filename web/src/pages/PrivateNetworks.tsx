import { useMemo, useState, type FormEvent } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  ClipboardList,
  Database,
  KeyRound,
  Loader2,
  Network,
  Plus,
  RotateCw,
  Server,
} from "lucide-react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { FeedbackBanner } from "../components/FeedbackBanner";
import {
  useApplyPrivateNetworkPlan,
  useCreatePrivateNetworkPlan,
  usePrivateNetworkPlans,
  type PrivateNetworkPlan,
  type PrivateNetworkTemplate,
  type StorageEngine,
} from "../hooks/useNodeOrchestration";
import { ApiRequestError } from "../utils/api";
import {
  PRIVATE_NETWORK_TEMPLATE_OPTIONS,
  STORAGE_ENGINE_OPTIONS,
  defaultPrivateNetworkName,
  formatStorageEngine,
  privateNetworkTemplateNodeCount,
} from "../utils/orchestration";

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

function optionalInteger(value: string, label: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const parsed = Number(trimmed);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${label} must be a positive integer`);
  }
  return parsed;
}

function PlanStatus({ status }: { status: PrivateNetworkPlan["status"] }) {
  if (status === "applied") {
    return <span className="status-badge status-running"><CheckCircle2 className="h-3.5 w-3.5" /> applied</span>;
  }
  if (status === "failed") {
    return <span className="status-badge status-error">failed</span>;
  }
  return <span className="status-badge status-stopped">draft</span>;
}

export default function PrivateNetworks() {
  const plansQuery = usePrivateNetworkPlans();
  const createPlan = useCreatePrivateNetworkPlan();
  const applyPlan = useApplyPrivateNetworkPlan();
  const plans = useMemo(() => plansQuery.data ?? [], [plansQuery.data]);
  const [banner, setBanner] = useState<BannerState>({});
  const [template, setTemplate] = useState<PrivateNetworkTemplate>("single");
  const [name, setName] = useState(defaultPrivateNetworkName("single"));
  const [nodeType, setNodeType] = useState<"neo-cli" | "neo-go">("neo-cli");
  const [storageEngine, setStorageEngine] = useState<StorageEngine>("leveldb");
  const [networkMagic, setNetworkMagic] = useState("");
  const [nodeNamePrefix, setNodeNamePrefix] = useState("");
  const [baseRpcPort, setBaseRpcPort] = useState("20332");
  const [baseP2pPort, setBaseP2pPort] = useState("20333");
  const [baseWebsocketPort, setBaseWebsocketPort] = useState("20334");
  const [baseMetricsPort, setBaseMetricsPort] = useState("22112");
  const [selectedPlanId, setSelectedPlanId] = useState("");
  const [applyDialogOpen, setApplyDialogOpen] = useState(false);
  const [replaceExisting, setReplaceExisting] = useState(false);

  const selectedPlan = useMemo(
    () => plans.find((plan) => plan.id === selectedPlanId) ?? plans[0],
    [plans, selectedPlanId],
  );
  const nodeCount = privateNetworkTemplateNodeCount(template);
  const appliedCount = plans.filter((plan) => plan.status === "applied").length;

  const handleTemplateChange = (nextTemplate: PrivateNetworkTemplate) => {
    setTemplate(nextTemplate);
    setName((current) =>
      PRIVATE_NETWORK_TEMPLATE_OPTIONS.some((option) => current === defaultPrivateNetworkName(option.value))
        ? defaultPrivateNetworkName(nextTemplate)
        : current);
  };

  const handleCreatePlan = async (event: FormEvent) => {
    event.preventDefault();
    setBanner({});
    try {
      const parsedNetworkMagic = optionalInteger(networkMagic, "Network magic");
      const parsedRpcPort = optionalInteger(baseRpcPort, "Base RPC port");
      const parsedP2pPort = optionalInteger(baseP2pPort, "Base P2P port");
      const parsedWebsocketPort = optionalInteger(baseWebsocketPort, "Base WebSocket port");
      const parsedMetricsPort = optionalInteger(baseMetricsPort, "Base metrics port");
      const plan = await createPlan.mutateAsync({
        name: name.trim(),
        template,
        nodeType,
        storageEngine,
        ...(parsedNetworkMagic ? { networkMagic: parsedNetworkMagic } : {}),
        ...(parsedRpcPort ? { baseRpcPort: parsedRpcPort } : {}),
        ...(parsedP2pPort ? { baseP2pPort: parsedP2pPort } : {}),
        ...(parsedWebsocketPort ? { baseWebsocketPort: parsedWebsocketPort } : {}),
        ...(parsedMetricsPort ? { baseMetricsPort: parsedMetricsPort } : {}),
        ...(nodeNamePrefix.trim() ? { nodeNamePrefix: nodeNamePrefix.trim() } : {}),
      });
      setSelectedPlanId(plan.id);
      setBanner({ success: `${plan.name} was planned with ${plan.plan.nodes.length} generated node${plan.plan.nodes.length === 1 ? "" : "s"}.` });
    } catch (error) {
      setBanner(apiBanner(error, "Failed to create private network plan."));
    }
  };

  const handleApplyPlan = async () => {
    if (!selectedPlan) return;
    setBanner({});
    try {
      const response = await applyPlan.mutateAsync({ planId: selectedPlan.id, replaceExisting });
      setApplyDialogOpen(false);
      if (response.plan) {
        setBanner({ success: `${selectedPlan.name} was applied. ${response.result.restoredCount} node${response.result.restoredCount === 1 ? "" : "s"} restored.` });
      } else {
        setBanner({
          error: "Private network plan was not fully applied.",
          suggestion: `${response.result.restoredCount} restored, ${response.result.skippedCount} skipped, ${response.result.failedCount} failed.`,
        });
      }
    } catch (error) {
      setBanner(apiBanner(error, "Failed to apply private network plan."));
    }
  };

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero pb-5">
        <div className="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
          <div>
            <p className="console-kicker">Private networks</p>
            <h1 className="mt-2 text-3xl font-semibold text-slate-950">One-click local N3 networks</h1>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
              Plan single, 4-node, or 7-node private networks with generated addresses, committee keys, seed lists, ports, storage engines, and role-aware plugin defaults.
            </p>
          </div>
          <button type="button" className="btn btn-secondary" onClick={() => void plansQuery.refetch()}>
            <RotateCw className="h-4 w-4" />
            Refresh
          </button>
        </div>

        <div className="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Plans</span><ClipboardList className="h-4 w-4 text-blue-700" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{plans.length}</p>
            <p className="mt-1 text-xs text-slate-500">draft and applied</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Applied</span><CheckCircle2 className="h-4 w-4 text-emerald-700" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{appliedCount}</p>
            <p className="mt-1 text-xs text-slate-500">restored to nodes</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Template</span><Server className="h-4 w-4 text-teal-700" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{nodeCount}</p>
            <p className="mt-1 text-xs text-slate-500">nodes in new plan</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Storage</span><Database className="h-4 w-4 text-amber-700" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{formatStorageEngine(storageEngine)}</p>
            <p className="mt-1 text-xs text-slate-500">for generated nodes</p>
          </div>
        </div>
      </section>

      <FeedbackBanner {...banner} />

      <div className="grid gap-6 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
        <form onSubmit={handleCreatePlan} className="card space-y-5">
          <div>
            <p className="console-kicker">Plan generator</p>
            <h2 className="mt-1 text-lg font-semibold text-slate-950">Create private network plan</h2>
            <p className="mt-2 text-sm leading-6 text-slate-600">Plans are saved first, then applied through the same guarded restore path as configuration snapshots.</p>
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-700">Name</label>
            <input className="input" value={name} onChange={(event) => setName(event.target.value)} placeholder="Local validator rehearsal" />
          </div>

          <div>
            <p className="mb-2 text-sm font-medium text-slate-700">Template</p>
            <div className="grid gap-3 sm:grid-cols-3">
              {PRIVATE_NETWORK_TEMPLATE_OPTIONS.map((option) => (
                <label key={option.value} className={`cursor-pointer rounded-lg border p-4 transition-colors ${
                  template === option.value ? "border-teal-500 bg-teal-50" : "border-slate-200 bg-white hover:border-slate-300"
                }`}>
                  <input
                    type="radio"
                    name="template"
                    value={option.value}
                    checked={template === option.value}
                    onChange={() => handleTemplateChange(option.value)}
                    className="sr-only"
                  />
                  <p className="font-semibold text-slate-950">{option.label}</p>
                  <p className="mt-1 text-xs leading-5 text-slate-600">{option.description}</p>
                </label>
              ))}
            </div>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-700">Node implementation</label>
              <select className="input" value={nodeType} onChange={(event) => setNodeType(event.target.value as "neo-cli" | "neo-go")}>
                <option value="neo-cli">Neo CLI</option>
                <option value="neo-go">Neo Go</option>
              </select>
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-700">Storage engine</label>
              <select className="input" value={storageEngine} onChange={(event) => setStorageEngine(event.target.value as StorageEngine)}>
                {STORAGE_ENGINE_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
              </select>
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-700">Network magic</label>
              <input className="input" inputMode="numeric" value={networkMagic} onChange={(event) => setNetworkMagic(event.target.value)} placeholder="Random if empty" />
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-700">Node name prefix</label>
              <input className="input" value={nodeNamePrefix} onChange={(event) => setNodeNamePrefix(event.target.value)} placeholder="Derived from plan name" />
            </div>
          </div>

          <div className="rounded-lg border border-slate-200 bg-slate-50 p-4">
            <div className="mb-3 flex items-center gap-2">
              <Network className="h-4 w-4 text-slate-600" />
              <h3 className="text-sm font-semibold text-slate-950">Base ports</h3>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <label className="block">
                <span className="mb-2 block text-xs font-medium text-slate-600">RPC</span>
                <input className="input" inputMode="numeric" value={baseRpcPort} onChange={(event) => setBaseRpcPort(event.target.value)} />
              </label>
              <label className="block">
                <span className="mb-2 block text-xs font-medium text-slate-600">P2P</span>
                <input className="input" inputMode="numeric" value={baseP2pPort} onChange={(event) => setBaseP2pPort(event.target.value)} />
              </label>
              <label className="block">
                <span className="mb-2 block text-xs font-medium text-slate-600">WebSocket</span>
                <input className="input" inputMode="numeric" value={baseWebsocketPort} onChange={(event) => setBaseWebsocketPort(event.target.value)} />
              </label>
              <label className="block">
                <span className="mb-2 block text-xs font-medium text-slate-600">Metrics</span>
                <input className="input" inputMode="numeric" value={baseMetricsPort} onChange={(event) => setBaseMetricsPort(event.target.value)} />
              </label>
            </div>
          </div>

          <button type="submit" className="btn btn-primary w-full justify-center" disabled={createPlan.isPending}>
            {createPlan.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Plus className="h-4 w-4" />}
            Create plan
          </button>
        </form>

        <div className="space-y-6">
          <div className="card">
            <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div>
                <p className="console-kicker">Saved plans</p>
                <h2 className="mt-1 text-lg font-semibold text-slate-950">Review and apply</h2>
              </div>
              {selectedPlan && (
                <button type="button" className="btn btn-primary justify-center" onClick={() => setApplyDialogOpen(true)} disabled={applyPlan.isPending}>
                  {applyPlan.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <CheckCircle2 className="h-4 w-4" />}
                  Apply selected
                </button>
              )}
            </div>

            {plans.length === 0 ? (
              <div className="rounded-lg border border-dashed border-slate-300 bg-slate-50 p-8 text-center">
                <Server className="mx-auto mb-3 h-10 w-10 text-slate-500" />
                <p className="font-medium text-slate-950">No private network plans yet</p>
                <p className="mt-1 text-sm text-slate-600">Create a plan to preview generated ports, addresses, validators, and seed list.</p>
              </div>
            ) : (
              <div className="grid gap-3">
                {plans.map((plan) => (
                  <button
                    key={plan.id}
                    type="button"
                    onClick={() => setSelectedPlanId(plan.id)}
                    className={`rounded-lg border p-4 text-left transition-colors ${
                      selectedPlan?.id === plan.id ? "border-teal-400 bg-teal-50" : "border-slate-200 bg-white hover:bg-slate-50"
                    }`}
                  >
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                      <div>
                        <p className="font-semibold text-slate-950">{plan.name}</p>
                        <p className="mt-1 text-sm text-slate-600">
                          {plan.template} · magic {plan.networkMagic} · {plan.plan.nodes.length} node{plan.plan.nodes.length === 1 ? "" : "s"}
                        </p>
                      </div>
                      <PlanStatus status={plan.status} />
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>

          {selectedPlan && (
            <div className="card space-y-5">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <div>
                  <p className="console-kicker">Plan preview</p>
                  <h2 className="mt-1 text-lg font-semibold text-slate-950">{selectedPlan.name}</h2>
                  <p className="mt-2 text-sm text-slate-600">
                    Validators {selectedPlan.plan.validatorsCount} · seed list {selectedPlan.plan.seedList.length} · standby committee {selectedPlan.plan.standbyCommittee.length}
                  </p>
                </div>
                <PlanStatus status={selectedPlan.status} />
              </div>

              <div className="rounded-lg border border-slate-200 bg-slate-50 p-4">
                <div className="mb-3 flex items-center gap-2">
                  <Network className="h-4 w-4 text-slate-600" />
                  <h3 className="text-sm font-semibold text-slate-950">Seed list</h3>
                </div>
                <div className="flex flex-wrap gap-2">
                  {selectedPlan.plan.seedList.map((seed) => (
                    <span key={seed} className="rounded-md border border-slate-200 bg-white px-2 py-1 font-mono text-xs text-slate-700">{seed}</span>
                  ))}
                </div>
              </div>

              <div className="space-y-3">
                {selectedPlan.plan.nodes.map((node) => (
                  <article key={node.name} className="rounded-lg border border-slate-200 bg-white p-4">
                    <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <h3 className="font-semibold text-slate-950">{node.name}</h3>
                          <span className="rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 text-xs font-medium text-slate-600">{node.type}</span>
                          <span className="rounded-full border border-teal-200 bg-teal-50 px-2 py-0.5 text-xs font-medium text-teal-800">{formatStorageEngine(node.storageEngine)}</span>
                        </div>
                        <p className="mt-2 flex items-center gap-2 text-sm text-slate-700">
                          <KeyRound className="h-4 w-4 text-teal-700" />
                          <span className="font-mono text-xs">{node.address}</span>
                        </p>
                      </div>
                      <div className="grid min-w-[220px] grid-cols-2 gap-2 text-xs text-slate-600">
                        <span className="rounded-md bg-slate-50 px-2 py-1">RPC {node.ports.rpc}</span>
                        <span className="rounded-md bg-slate-50 px-2 py-1">P2P {node.ports.p2p}</span>
                        <span className="rounded-md bg-slate-50 px-2 py-1">WS {node.ports.websocket}</span>
                        <span className="rounded-md bg-slate-50 px-2 py-1">Metrics {node.ports.metrics}</span>
                      </div>
                    </div>
                    <div className="mt-3 flex flex-wrap gap-2">
                      {node.roleIds.map((roleId) => (
                        <span key={roleId} className="rounded-md border border-blue-200 bg-blue-50 px-2 py-1 text-xs font-medium text-blue-800">{roleId.replace("builtin-", "")}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      <ConfirmDialog
        open={applyDialogOpen}
        title="Apply private network plan?"
        description={
          <div className="space-y-3">
            <p>This restores the generated node configuration snapshot and creates the planned private-network nodes.</p>
            <label className="flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 p-3 text-left">
              <input className="mt-1" type="checkbox" checked={replaceExisting} onChange={(event) => setReplaceExisting(event.target.checked)} />
              <span>
                <span className="block font-medium text-amber-900">Replace existing managed nodes</span>
                <span className="block text-sm text-amber-800">Use only when intentionally rebuilding this local private network from scratch.</span>
              </span>
            </label>
            {replaceExisting && (
              <p className="flex gap-2 text-sm text-red-700">
                <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
                Existing managed node registrations can be overwritten.
              </p>
            )}
          </div>
        }
        confirmLabel="Apply plan"
        confirmVariant={replaceExisting ? "danger" : "primary"}
        isConfirming={applyPlan.isPending}
        onCancel={() => setApplyDialogOpen(false)}
        onConfirm={() => void handleApplyPlan()}
      />
    </div>
  );
}
