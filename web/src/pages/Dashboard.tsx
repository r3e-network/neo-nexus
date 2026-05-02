import {
  Activity,
  AlertCircle,
  AlertTriangle,
  ArrowRight,
  Cpu,
  FolderOpen,
  Gauge,
  KeyRound,
  PlugZap,
  Plus,
  Server,
  Settings,
  ShieldCheck,
} from "lucide-react";
import { Link } from "react-router-dom";
import { EmptyState } from "../components/EmptyState";
import { StatSkeleton } from "../components/LoadingSkeleton";
import { NodeProtectionLabel } from "../components/NodeProtectionLabel";
import { ProgressBar } from "../components/ProgressBar";
import { SignerStatus } from "../components/SignerStatus";
import { useAuth } from "../hooks/useAuth";
import { useNodes, useSystemMetrics, type Node } from "../hooks/useNodes";
import { formatBytes } from "../utils/format";
import { countProtectedNodes } from "../utils/signerVisibility";

function nodeOwnershipLabel(node: Node) {
  if (!node.settings?.import) return "NeoNexus managed";
  return node.settings.import.ownershipMode === "managed-process"
    ? "Imported · managed process"
    : node.settings.import.ownershipMode === "managed-config"
      ? "Imported · managed config"
      : "Imported · observe only";
}

function healthTone(value: number) {
  if (value >= 90) return "text-emerald-700";
  if (value >= 65) return "text-amber-700";
  return "text-red-700";
}

export default function Dashboard() {
  const { data: nodes = [], isLoading } = useNodes();
  const { data: systemMetrics } = useSystemMetrics();
  const { user } = useAuth();

  const isDefaultPassword = user?.usingDefaultPassword === true;
  const runningNodes = nodes.filter((n) => n.process.status === "running");
  const errorNodes = nodes.filter((n) => n.process.status === "error");
  const transitionalNodes = nodes.filter((n) => ["starting", "stopping", "syncing"].includes(n.process.status));
  const importedNodes = nodes.filter((n) => n.settings?.import);
  const protectedNodes = countProtectedNodes(nodes);
  const fleetHealth = nodes.length === 0
    ? 0
    : Math.max(0, Math.round(((runningNodes.length + transitionalNodes.length * 0.5) / nodes.length) * 100));
  const fleetHealthTone = nodes.length === 0 ? "text-slate-700" : healthTone(fleetHealth);
  const fleetHealthBadge = nodes.length === 0
    ? { className: "status-stopped", label: "No nodes" }
    : errorNodes.length
      ? { className: "status-error", label: "Needs attention" }
      : runningNodes.length === 0
        // Don't claim "Healthy" when nothing is actually running — that reads as
        // a contradiction next to the 0% headline.
        ? { className: "status-stopped", label: "All stopped" }
        : runningNodes.length < nodes.length
          ? { className: "status-syncing", label: "Partial" }
          : { className: "status-running", label: "Healthy" };

  const stats = [
    { label: "Total nodes", value: nodes.length, detail: `${importedNodes.length} imported`, icon: Server, tone: "text-blue-700", bg: "bg-blue-50" },
    { label: "Running", value: runningNodes.length, detail: `${transitionalNodes.length} changing state`, icon: Activity, tone: "text-emerald-700", bg: "bg-emerald-50" },
    { label: "Fleet health", value: `${fleetHealth}%`, detail: nodes.length === 0 ? "Add a node to begin" : errorNodes.length ? `${errorNodes.length} action required` : "No blocking errors", icon: Gauge, tone: fleetHealthTone, bg: "bg-teal-50" },
    { label: "Protected keys", value: protectedNodes, detail: "TEE / secure signer", icon: ShieldCheck, tone: protectedNodes ? "text-teal-700" : "text-slate-500", bg: "bg-teal-50" },
  ];

  const quickActions = [
    { title: "Create native node", description: "Provision neo-cli or neo-go with safe defaults.", href: "/nodes/create", icon: Plus, accent: "text-blue-700" },
    { title: "Import existing node", description: "Attach observe-only first, then adopt safely.", href: "/nodes/import", icon: FolderOpen, accent: "text-amber-700" },
    { title: "Configure plugins", description: "Enable RPC, storage, monitoring and tooling.", href: "/plugins", icon: PlugZap, accent: "text-teal-700" },
    { title: "Protect private keys", description: "Register TEE, Nitro, SGX or HSM signer profiles.", href: "/settings#secure-signers", icon: KeyRound, accent: "text-teal-700" },
  ];

  const topNodes = nodes.slice(0, 6);

  return (
    <div className="space-y-6 animate-fade-in">
      <section className="page-hero pb-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="console-kicker">Overview</p>
            <div className="mt-2 flex flex-wrap items-center gap-3">
              <h1 className="text-2xl font-semibold text-slate-950 lg:text-3xl">Operations dashboard</h1>
              {errorNodes.length > 0 && (
                <span className="inline-flex items-center gap-2 rounded-md border border-red-200 bg-red-50 px-2.5 py-1 text-sm font-medium text-red-700">
                  <AlertCircle className="h-4 w-4" /> {errorNodes.length} issue{errorNodes.length === 1 ? "" : "s"}
                </span>
              )}
            </div>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
              Fleet health, node lifecycle, plugins, monitoring, and signer posture in one clear workspace.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link to="/nodes/create" className="btn btn-primary">
              <Plus className="h-4 w-4" /> Create node
            </Link>
            <Link to="/nodes/import" className="btn btn-secondary">
              <FolderOpen className="h-4 w-4" /> Import
            </Link>
            <Link to="/settings#secure-signers" className="btn btn-secondary">
              <KeyRound className="h-4 w-4" /> Signers
            </Link>
          </div>
        </div>
      </section>

      {(isDefaultPassword || protectedNodes > 0) && (
        <div className="grid gap-3 lg:grid-cols-2">
          {isDefaultPassword && (
            <div className="rounded-lg border border-amber-200 bg-amber-50 p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-700" />
                <div>
                  <h3 className="text-sm font-semibold text-amber-900">Legacy default password in use</h3>
                  <p className="mt-1 text-sm text-amber-800">Change this admin password before exposing the console beyond localhost.</p>
                  <Link to="/settings" className="mt-3 inline-flex items-center gap-2 text-sm font-medium text-amber-800 hover:text-amber-950">
                    <Settings className="h-4 w-4" /> Open security settings
                  </Link>
                </div>
              </div>
            </div>
          )}
          {protectedNodes > 0 && (
            <div className="rounded-lg border border-teal-200 bg-teal-50 p-4">
              <div className="flex items-start gap-3">
                <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-teal-700" />
                <div>
                  <h3 className="text-sm font-semibold text-teal-950">Secure signer protection active</h3>
                  <p className="mt-1 text-sm text-teal-900">{protectedNodes} node{protectedNodes === 1 ? "" : "s"} use external signing; signer policy defaults to deny-by-default.</p>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      <section className="grid items-start gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="min-w-0 space-y-6">
          <div className="card overflow-hidden p-0">
            <div className="grid lg:grid-cols-[280px_minmax(0,1fr)]">
              <div className="border-b border-slate-200 bg-slate-50 p-5 lg:border-b-0 lg:border-r">
                <p className="console-kicker">Fleet health</p>
                <div className="mt-5 flex items-end justify-between gap-4">
                  <p className={`text-5xl font-semibold leading-none ${fleetHealthTone}`}>{fleetHealth}%</p>
                  <span className={`status-badge ${fleetHealthBadge.className}`}>
                    {fleetHealthBadge.label}
                  </span>
                </div>
                <div className="mt-5">
                  <ProgressBar
                    value={fleetHealth}
                    color={nodes.length === 0 ? "bg-slate-400" : fleetHealth >= 90 ? "bg-emerald-500" : fleetHealth >= 65 ? "bg-amber-500" : "bg-red-500"}
                  />
                </div>
                <p className="mt-4 text-sm leading-6 text-slate-600">
                  {runningNodes.length} running, {transitionalNodes.length} changing state, {errorNodes.length} with errors.
                </p>
              </div>

              <div className="p-5">
                {isLoading ? (
                  <StatSkeleton />
                ) : (
                  <div className="grid gap-3 sm:grid-cols-2">
                    {stats.map((stat) => (
                      <div key={stat.label} className="rounded-lg border border-slate-200 bg-white p-4">
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="text-sm font-medium text-slate-600">{stat.label}</p>
                            <p className="mt-2 text-3xl font-semibold text-slate-950">{stat.value}</p>
                          </div>
                          <div className={`rounded-lg p-2.5 ${stat.bg} ${stat.tone}`}>
                            <stat.icon className="h-5 w-5" />
                          </div>
                        </div>
                        <p className="mt-4 text-sm text-slate-500">{stat.detail}</p>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>

          <div className="card">
            <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="console-kicker">Fleet inventory</p>
                <h2 className="mt-1 text-lg font-semibold text-slate-950">Nodes</h2>
              </div>
              <Link to="/nodes" className="inline-flex items-center gap-2 text-sm font-medium text-blue-700 hover:text-blue-900">
                Open node console <ArrowRight className="h-4 w-4" />
              </Link>
            </div>

            {isLoading ? (
              <div className="space-y-3">
                {[...Array(3)].map((_, i) => <div key={i} className="skeleton h-16" />)}
              </div>
            ) : nodes.length === 0 ? (
              <EmptyState
                icon={Server}
                title="No nodes yet"
                description="Deploy a new Neo node or import an existing installation. Imports start observe-only so native nodes are not modified accidentally."
                actions={[
                  { label: "Create Node", href: "/nodes/create", variant: "primary" },
                  { label: "Import Existing", href: "/nodes/import", variant: "secondary" },
                ]}
              />
            ) : (
              <div className="overflow-hidden rounded-lg border border-slate-200">
                <div className="divide-y divide-slate-200">
                  {topNodes.map((node) => (
                    <Link key={node.id} to={`/nodes/${node.id}`} className="flex flex-col gap-3 bg-white p-4 transition-colors hover:bg-slate-50 md:flex-row md:items-center md:justify-between">
                      <div className="flex items-start gap-3">
                        <div className={`rounded-lg p-2 ${node.type === "neo-cli" ? "bg-blue-50 text-blue-700" : "bg-emerald-50 text-emerald-700"}`}>
                          <Activity className="h-5 w-5" />
                        </div>
                        <div>
                          <div className="flex flex-wrap items-center gap-2">
                            <h3 className="font-medium text-slate-950">{node.name}</h3>
                            <span className={`status-badge status-${node.process.status}`}>{node.process.status}</span>
                          </div>
                          <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-slate-500">
                            <span>{node.type}</span>
                            <span>•</span>
                            <span className="capitalize">{node.network}</span>
                            <span>•</span>
                            <span>{nodeOwnershipLabel(node)}</span>
                            <NodeProtectionLabel node={node} padding="px-2 py-0.5" />
                          </div>
                        </div>
                      </div>
                      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-slate-600">
                        <span>Block {node.metrics?.blockHeight?.toLocaleString() ?? "—"}</span>
                        <span>{node.metrics?.connectedPeers ?? "—"} peers</span>
                        {node.settings?.keyProtection?.mode === "secure-signer" && <SignerStatus nodeId={node.id} showPrefix textSize="text-[11px]" />}
                      </div>
                    </Link>
                  ))}
                </div>
                {nodes.length > topNodes.length && (
                  <div className="border-t border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                    Showing {topNodes.length} of {nodes.length} nodes.
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-6">
          <div className="card">
            <div className="mb-4">
              <p className="console-kicker">Quick actions</p>
              <h2 className="mt-1 text-lg font-semibold text-slate-950">Operate faster</h2>
            </div>
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
              {quickActions.map((action) => (
                <Link key={action.title} to={action.href} className="group rounded-lg border border-slate-200 bg-white p-4 transition-colors hover:border-teal-200 hover:bg-teal-50/50">
                  <div className="flex items-start gap-3">
                    <span className="rounded-lg border border-slate-200 bg-slate-50 p-2">
                      <action.icon className={`h-5 w-5 ${action.accent}`} />
                    </span>
                    <div>
                      <p className="font-medium text-slate-950">{action.title}</p>
                      <p className="mt-1 text-sm leading-5 text-slate-600">{action.description}</p>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>

          {systemMetrics && (
            <div className="card">
              <div className="mb-4 flex items-center gap-2">
                <Cpu className="h-5 w-5 text-blue-700" />
                <h2 className="text-lg font-semibold text-slate-950">Host resources</h2>
              </div>
              <div className="space-y-5">
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-600">CPU</span><span className="font-medium text-slate-950">{systemMetrics.cpu.usage.toFixed(1)}%</span></div>
                  <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" />
                </div>
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-600">Memory</span><span className="font-medium text-slate-950">{formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}</span></div>
                  <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" />
                </div>
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-600">Disk</span><span className="font-medium text-slate-950">{formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}</span></div>
                  <ProgressBar value={systemMetrics.disk.percentage} color="bg-amber-500" />
                </div>
              </div>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
