import {
  Activity,
  AlertCircle,
  AlertTriangle,
  ArrowRight,
  BellRing,
  Cpu,
  FolderOpen,
  Gauge,
  KeyRound,
  PlugZap,
  Plus,
  Server,
  Settings,
  ShieldCheck,
  TerminalSquare,
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
  if (value >= 90) return "text-emerald-300";
  if (value >= 65) return "text-amber-300";
  return "text-red-300";
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
  const avgPeers = runningNodes.length > 0
    ? Math.round(runningNodes.reduce((sum, node) => sum + (node.metrics?.connectedPeers ?? 0), 0) / runningNodes.length)
    : 0;
  const fleetHealth = nodes.length === 0
    ? 0
    : Math.max(0, Math.round(((runningNodes.length + transitionalNodes.length * 0.5) / nodes.length) * 100));

  const stats = [
    { label: "Total nodes", value: nodes.length, detail: `${importedNodes.length} imported`, icon: Server, tone: "text-blue-300", bg: "bg-blue-500/10" },
    { label: "Running", value: runningNodes.length, detail: `${transitionalNodes.length} changing state`, icon: Activity, tone: "text-emerald-300", bg: "bg-emerald-500/10" },
    { label: "Fleet health", value: `${fleetHealth}%`, detail: errorNodes.length ? `${errorNodes.length} action required` : "No blocking errors", icon: Gauge, tone: healthTone(fleetHealth), bg: "bg-cyan-500/10" },
    { label: "Protected keys", value: protectedNodes, detail: "TEE / secure signer", icon: ShieldCheck, tone: protectedNodes ? "text-cyan-300" : "text-slate-400", bg: "bg-cyan-500/10" },
  ];

  const quickActions = [
    { title: "Create native node", description: "Provision neo-cli or neo-go with safe defaults.", href: "/nodes/create", icon: Plus, accent: "text-blue-300" },
    { title: "Import existing node", description: "Attach observe-only first, then adopt safely.", href: "/nodes/import", icon: FolderOpen, accent: "text-amber-300" },
    { title: "Configure plugins", description: "Enable RPC, storage, monitoring and tooling.", href: "/plugins", icon: PlugZap, accent: "text-purple-300" },
    { title: "Protect private keys", description: "Register TEE, Nitro, SGX or HSM signer profiles.", href: "/settings#secure-signers", icon: KeyRound, accent: "text-cyan-300" },
  ];

  const topNodes = nodes.slice(0, 6);

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="premium-panel rounded-[1.35rem] p-7 lg:p-10">
        <div className="relative z-10 grid gap-8 2xl:grid-cols-[1.25fr_0.75fr] 2xl:items-center">
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border border-indigo-200/15 bg-indigo-300/10 px-3 py-1 text-xs font-medium text-indigo-100 shadow-[0_14px_40px_-30px_rgba(113,112,255,0.9)]">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-300 shadow-[0_0_12px_rgba(52,211,153,0.9)]" />
              Native node operations console
            </div>
            <div className="mt-4 flex flex-wrap items-center gap-3">
              <h1 className="max-w-4xl text-3xl font-semibold leading-[1.02] tracking-[-0.045em] text-white lg:text-4xl">
                Professional control plane for native Neo infrastructure.
              </h1>
              {errorNodes.length > 0 && (
                <span className="inline-flex items-center gap-2 rounded-xl border border-red-300/25 bg-red-500/10 px-3 py-1 text-sm font-medium text-red-100">
                  <AlertCircle className="h-4 w-4" /> {errorNodes.length} issue{errorNodes.length === 1 ? "" : "s"}
                </span>
              )}
            </div>
            <p className="mt-5 max-w-3xl text-sm leading-6 text-slate-300 lg:text-base">
              Deploy, import, observe, configure and protect native Neo nodes from one polished operator cockpit. Safe imports start observe-only, ownership upgrades are deliberate, and signer policy stays fail-closed.
            </p>
            <div className="mt-7 flex flex-wrap gap-3">
              <Link to="/nodes/create" className="btn btn-primary">
                <Plus className="h-4 w-4" /> Create node
              </Link>
              <Link to="/nodes/import" className="btn btn-secondary">
                <FolderOpen className="h-4 w-4" /> Import existing
              </Link>
              <Link to="/settings#secure-signers" className="btn btn-secondary">
                <KeyRound className="h-4 w-4" /> Private key protection
              </Link>
            </div>
          </div>

          <div className="console-surface p-6">
            <div className="mb-4 flex items-center justify-between">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500">Fleet readiness</p>
                <p className={`mt-1 text-5xl font-semibold tracking-[-0.055em] ${healthTone(fleetHealth)}`}>{fleetHealth}%</p>
              </div>
              <div className="rounded-2xl border border-indigo-200/15 bg-indigo-300/10 p-3 text-indigo-100">
                <TerminalSquare className="h-6 w-6" />
              </div>
            </div>
            <div className="space-y-4">
              <div>
                <div className="mb-2 flex justify-between text-xs text-slate-400">
                  <span>Running capacity</span>
                  <span>{nodes.length ? Math.round((runningNodes.length / nodes.length) * 100) : 0}%</span>
                </div>
                <ProgressBar value={nodes.length ? (runningNodes.length / nodes.length) * 100 : 0} color="bg-emerald-500" />
              </div>
              <div className="soft-divider" />
              <div className="grid grid-cols-3 gap-3 text-center text-sm">
                <div className="rounded-xl border border-white/[0.06] bg-white/[0.035] p-3">
                  <p className="text-xl font-semibold text-white">{avgPeers}</p>
                  <p className="text-xs text-slate-500">avg peers</p>
                </div>
                <div className="rounded-xl border border-white/[0.06] bg-white/[0.035] p-3">
                  <p className="text-xl font-semibold text-white">{protectedNodes}</p>
                  <p className="text-xs text-slate-500">signers</p>
                </div>
                <div className="rounded-xl border border-white/[0.06] bg-white/[0.035] p-3">
                  <p className="text-xl font-semibold text-white">{errorNodes.length}</p>
                  <p className="text-xs text-slate-500">errors</p>
                </div>
              </div>
              <div className="rounded-xl border border-emerald-300/10 bg-emerald-300/[0.035] p-3 text-xs text-slate-400">
                <span className="font-mono text-emerald-300">policy:</span> imported nodes require explicit ownership before mutation.
              </div>
            </div>
          </div>
        </div>
      </section>

      {(isDefaultPassword || protectedNodes > 0) && (
        <div className="grid gap-3 lg:grid-cols-2">
          {isDefaultPassword && (
            <div className="rounded-xl border border-amber-400/20 bg-amber-500/10 p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-400" />
                <div>
                  <h3 className="text-sm font-semibold text-amber-200">Default password in use</h3>
                  <p className="mt-1 text-sm text-amber-100/80">Change the default admin password before exposing this console beyond localhost.</p>
                  <Link to="/settings" className="mt-3 inline-flex items-center gap-2 text-sm font-medium text-amber-200 hover:text-amber-100">
                    <Settings className="h-4 w-4" /> Open security settings
                  </Link>
                </div>
              </div>
            </div>
          )}
          {protectedNodes > 0 && (
            <div className="rounded-xl border border-cyan-400/20 bg-cyan-500/10 p-4">
              <div className="flex items-start gap-3">
                <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-cyan-300" />
                <div>
                  <h3 className="text-sm font-semibold text-cyan-200">Secure signer protection active</h3>
                  <p className="mt-1 text-sm text-cyan-100/80">{protectedNodes} node{protectedNodes === 1 ? "" : "s"} use external signing; signer policy defaults to deny-by-default.</p>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {isLoading ? (
        <StatSkeleton />
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
          {stats.map((stat, i) => (
            <div key={stat.label} className={`card-interactive group animate-fade-in-up stagger-${i + 1} cursor-default p-5`}>
              <div className="relative z-10 flex items-start justify-between gap-4">
                <div>
                  <p className="text-sm font-medium text-slate-400">{stat.label}</p>
                  <p className="mt-2 text-3xl font-semibold tracking-[-0.04em] text-white">{stat.value}</p>
                </div>
                <div className={`rounded-2xl border border-white/[0.07] bg-white/[0.045] p-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.035)] ${stat.tone}`}>
                  <stat.icon className="h-5 w-5" />
                </div>
              </div>
              <p className="relative z-10 mt-4 text-sm text-slate-500">{stat.detail}</p>
            </div>
          ))}
        </div>
      )}

      <section className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
        <div className="card">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p className="console-kicker">Fleet inventory</p>
              <h2 className="mt-1 text-lg font-semibold text-white">Nodes</h2>
            </div>
            <Link to="/nodes" className="inline-flex items-center gap-2 text-sm font-medium text-blue-300 hover:text-blue-200">
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
            <div className="divide-y divide-slate-800/70 overflow-hidden rounded-xl border border-slate-800/70">
              {topNodes.map((node) => (
                <Link key={node.id} to={`/nodes/${node.id}`} className="flex flex-col gap-3 bg-slate-950/20 p-4 transition-colors hover:bg-slate-800/40 md:flex-row md:items-center md:justify-between">
                  <div className="flex items-start gap-3">
                    <div className={`rounded-lg p-2 ${node.type === "neo-cli" ? "bg-blue-500/10 text-blue-300" : "bg-emerald-500/10 text-emerald-300"}`}>
                      <Activity className="h-5 w-5" />
                    </div>
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        <h3 className="font-medium text-white">{node.name}</h3>
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
                  <div className="flex items-center gap-4 text-sm text-slate-400">
                    <span>Block {node.metrics?.blockHeight?.toLocaleString() ?? "—"}</span>
                    <span>{node.metrics?.connectedPeers ?? "—"} peers</span>
                    {node.settings?.keyProtection?.mode === "secure-signer" && <SignerStatus nodeId={node.id} showPrefix textSize="text-[11px]" />}
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-6">
          <div className="card">
            <div className="mb-4 flex items-center justify-between">
              <div>
                <p className="console-kicker">Quick actions</p>
                <h2 className="mt-1 text-lg font-semibold text-white">Operate faster</h2>
              </div>
              <BellRing className="h-5 w-5 text-slate-500" />
            </div>
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
              {quickActions.map((action) => (
                <Link key={action.title} to={action.href} className="group rounded-2xl border border-white/[0.07] bg-white/[0.035] p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.035)] transition-all hover:-translate-y-0.5 hover:border-indigo-200/25 hover:bg-white/[0.055] hover:shadow-[0_18px_48px_-32px_rgba(113,112,255,0.55)]">
                  <div className="flex items-start gap-3">
                    <span className="rounded-xl border border-white/[0.06] bg-slate-950/45 p-2">
                      <action.icon className={`h-5 w-5 ${action.accent}`} />
                    </span>
                    <div>
                      <p className="font-medium text-white group-hover:text-indigo-100">{action.title}</p>
                      <p className="mt-1 text-sm leading-5 text-slate-400">{action.description}</p>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>

          {systemMetrics && (
            <div className="card">
              <div className="mb-4 flex items-center gap-2">
                <Cpu className="h-5 w-5 text-blue-300" />
                <h2 className="text-lg font-semibold text-white">Host resources</h2>
              </div>
              <div className="space-y-5">
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-400">CPU</span><span className="text-white">{systemMetrics.cpu.usage.toFixed(1)}%</span></div>
                  <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" />
                </div>
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-400">Memory</span><span className="text-white">{formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}</span></div>
                  <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" />
                </div>
                <div>
                  <div className="mb-2 flex justify-between text-sm"><span className="text-slate-400">Disk</span><span className="text-white">{formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}</span></div>
                  <ProgressBar value={systemMetrics.disk.percentage} color="bg-purple-500" />
                </div>
              </div>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
