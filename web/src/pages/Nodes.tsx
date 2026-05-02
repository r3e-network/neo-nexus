import {
  Activity,
  AlertCircle,
  DatabaseZap,
  Eye,
  FolderOpen,
  Gauge,
  Plus,
  Play,
  Search,
  Server,
  ShieldCheck,
  Square,
  Trash2,
} from "lucide-react";
import { Link } from "react-router-dom";
import { useMemo, useState } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { EmptyState } from "../components/EmptyState";
import { TableRowSkeleton } from "../components/LoadingSkeleton";
import { NodeProtectionLabel } from "../components/NodeProtectionLabel";
import { SignerStatus } from "../components/SignerStatus";
import { SpinnerButton } from "../components/SpinnerButton";
import { useDeleteNode, useNodes, useStartNode, useStopNode, type Node } from "../hooks/useNodes";

type NodeFilter = "all" | "running" | "needs-attention" | "protected" | "imported";

function ownershipLabel(node: Node) {
  if (!node.settings?.import) return "NeoNexus managed";
  const mode = node.settings.import.ownershipMode ?? "observe-only";
  if (mode === "managed-process") return "Imported · process managed";
  if (mode === "managed-config") return "Imported · config managed";
  return "Imported · observe only";
}

function lifecycleAllowed(node: Node) {
  return !node.settings?.import || node.settings.import.ownershipMode === "managed-process";
}

function filterNodes(nodes: Node[], filter: NodeFilter, searchTerm: string) {
  const normalizedSearch = searchTerm.trim().toLowerCase();
  return nodes.filter((node) => {
    const matchesSearch = !normalizedSearch || [
      node.name,
      node.type,
      node.network,
      node.version,
      node.process.status,
      ownershipLabel(node),
    ].some((field) => field.toLowerCase().includes(normalizedSearch));

    if (!matchesSearch) return false;
    if (filter === "running") return node.process.status === "running";
    if (filter === "needs-attention") return node.process.status === "error" || Boolean(node.process.errorMessage);
    if (filter === "protected") return node.settings?.keyProtection?.mode === "secure-signer";
    if (filter === "imported") return Boolean(node.settings?.import);
    return true;
  });
}

export default function Nodes() {
  const { data: nodes = [], isLoading } = useNodes();
  const startNode = useStartNode();
  const stopNode = useStopNode();
  const deleteNode = useDeleteNode();
  const [searchTerm, setSearchTerm] = useState("");
  const [activeFilter, setActiveFilter] = useState<NodeFilter>("all");
  const [deleting, setDeleting] = useState<string | null>(null);
  const [nodePendingDelete, setNodePendingDelete] = useState<Node | null>(null);

  const filteredNodes = useMemo(() => filterNodes(nodes, activeFilter, searchTerm), [nodes, activeFilter, searchTerm]);
  const runningCount = nodes.filter((node) => node.process.status === "running").length;
  const attentionCount = nodes.filter((node) => node.process.status === "error" || node.process.errorMessage).length;
  const protectedCount = nodes.filter((node) => node.settings?.keyProtection?.mode === "secure-signer").length;
  const importedCount = nodes.filter((node) => node.settings?.import).length;

  const filters: Array<{ id: NodeFilter; label: string; count: number }> = [
    { id: "all", label: "All nodes", count: nodes.length },
    { id: "running", label: "Running", count: runningCount },
    { id: "needs-attention", label: "Needs attention", count: attentionCount },
    { id: "protected", label: "Protected keys", count: protectedCount },
    { id: "imported", label: "Imported", count: importedCount },
  ];

  const confirmDelete = async () => {
    if (!nodePendingDelete) {
      return;
    }

    setDeleting(nodePendingDelete.id);
    try {
      await deleteNode.mutateAsync(nodePendingDelete.id);
      setNodePendingDelete(null);
    } finally {
      setDeleting(null);
    }
  };

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero pb-5">
        <div className="relative z-10 flex flex-col gap-5 xl:flex-row xl:items-center xl:justify-between">
          <div>
            <p className="console-kicker">Node fleet</p>
            <h1 className="mt-2 text-3xl font-semibold text-slate-950">Nodes</h1>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
              Configure networks, lifecycle, plugin readiness, signer protection and imported-node ownership from one operations table.
            </p>
          </div>
          <div className="flex flex-col gap-3 sm:flex-row">
            <Link to="/nodes/import" className="btn btn-secondary justify-center">
              <FolderOpen className="h-4 w-4" /> Import existing
            </Link>
            <Link to="/nodes/create" className="btn btn-primary justify-center">
              <Plus className="h-4 w-4" /> Create node
            </Link>
          </div>
        </div>

        <div className="relative z-10 mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Total</span><Server className="h-4 w-4 text-blue-600" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{nodes.length}</p>
            <p className="mt-1 text-xs text-slate-500">neo-cli and neo-go</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Running</span><Gauge className="h-4 w-4 text-emerald-600" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{runningCount}</p>
            <p className="mt-1 text-xs text-slate-500">processes online</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Protected</span><ShieldCheck className="h-4 w-4 text-cyan-600" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{protectedCount}</p>
            <p className="mt-1 text-xs text-slate-500">secure signer bindings</p>
          </div>
          <div className="stat-tile">
            <div className="flex items-center justify-between"><span className="text-sm text-slate-600">Imported</span><Eye className="h-4 w-4 text-amber-600" /></div>
            <p className="mt-2 text-2xl font-semibold text-slate-950">{importedCount}</p>
            <p className="mt-1 text-xs text-slate-500">safe ownership modes</p>
          </div>
        </div>
      </section>

      <div className="card space-y-4">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-500" />
            <input
              type="text"
              placeholder="Search by name, network, type, status, ownership..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="input pl-10"
            />
          </div>
          <div className="flex flex-wrap gap-2">
            {filters.map((filter) => (
              <button
                key={filter.id}
                type="button"
                onClick={() => setActiveFilter(filter.id)}
                className={`filter-chip ${
                  activeFilter === filter.id
                    ? "filter-chip-active"
                    : ""
                }`}
              >
                {filter.label} <span className="ml-1 text-xs opacity-70">{filter.count}</span>
              </button>
            ))}
          </div>
        </div>

        {isLoading ? (
          <TableRowSkeleton rows={5} />
        ) : filteredNodes.length === 0 ? (
          searchTerm || activeFilter !== "all" ? (
            <div className="py-12 text-center">
              <Activity className="mx-auto mb-4 h-12 w-12 text-slate-600" />
              <p className="text-slate-600">No nodes match this view.</p>
              <button type="button" onClick={() => { setSearchTerm(""); setActiveFilter("all"); }} className="mt-3 text-sm font-medium text-blue-700 hover:text-blue-900">
                Clear filters
              </button>
            </div>
          ) : (
            <EmptyState
              icon={Server}
              title="No nodes configured"
              description="Create a managed node or import an existing native node in observe-only mode."
              actions={[
                { label: "Create Node", href: "/nodes/create", variant: "primary" },
                { label: "Import Existing", href: "/nodes/import", variant: "secondary" },
              ]}
            />
          )
        ) : (
          <div className="table-shell">
            <table className="w-full min-w-[780px]">
              <thead className="bg-slate-50">
                <tr className="border-b border-slate-200 text-left text-xs uppercase tracking-wide text-slate-500">
                  <th className="px-4 py-3 font-semibold">Node</th>
                  <th className="px-4 py-3 font-semibold">Network</th>
                  <th className="px-4 py-3 font-semibold">Sync / Peers</th>
                  <th className="px-4 py-3 font-semibold">Ports</th>
                  <th className="px-4 py-3 font-semibold">Security</th>
                  <th className="sticky right-0 w-[112px] bg-slate-50 px-4 py-3 text-right font-semibold shadow-[-10px_0_16px_-16px_rgba(15,23,42,0.45)]">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-200">
                {filteredNodes.map((node) => {
                  const canControlLifecycle = lifecycleAllowed(node);
                  const isRunning = node.process.status === "running";
                  return (
                    <tr key={node.id} className="group bg-transparent transition-colors hover:bg-slate-50">
                      <td className="px-4 py-4">
                        <Link to={`/nodes/${node.id}`} className="flex items-start gap-3">
                          <div className={`rounded-lg p-2 ${node.type === "neo-cli" ? "bg-blue-50 text-blue-700" : "bg-emerald-50 text-emerald-700"}`}>
                            <Activity className="h-5 w-5" />
                          </div>
                          <div>
                            <div className="flex flex-wrap items-center gap-2">
                              <p className="font-medium text-slate-950">{node.name}</p>
                              <span className={`status-badge status-${node.process.status}`}>{node.process.status}</span>
                              {node.chain === 'x' && (
                                <span className="rounded-full bg-violet-50 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-violet-700 border border-violet-200">Neo X</span>
                              )}
                            </div>
                            <p className="mt-1 text-xs text-slate-500">{node.type} · v{node.version} · {ownershipLabel(node)}</p>
                            {node.process.errorMessage && (
                              <p className="mt-1 inline-flex items-center gap-1 text-xs text-red-700"><AlertCircle className="h-3 w-3" /> {node.process.errorMessage}</p>
                            )}
                          </div>
                        </Link>
                      </td>
                      <td className="px-4 py-4 text-sm text-slate-700">
                        <span className="capitalize">{node.network}</span>
                        <p className="mt-1 text-xs text-slate-500">{node.syncMode} sync</p>
                      </td>
                      <td className="px-4 py-4 text-sm text-slate-700">
                        <p>Block {node.metrics?.blockHeight?.toLocaleString() ?? "—"}</p>
                        <p className="mt-1 text-xs text-slate-500">{node.metrics?.connectedPeers ?? "—"} peers</p>
                      </td>
                      <td className="px-4 py-4 text-sm text-slate-700">
                        <div className="font-mono text-xs leading-5 text-slate-600">
                          <p>RPC {node.ports.rpc}</p>
                          <p>P2P {node.ports.p2p}</p>
                          {node.ports.websocket && <p>WS {node.ports.websocket}</p>}
                        </div>
                      </td>
                      <td className="px-4 py-4">
                        <div className="flex flex-col items-start gap-2">
                          <NodeProtectionLabel node={node} padding="px-2 py-0.5" />
                          {node.settings?.keyProtection?.mode === "secure-signer" ? (
                            <SignerStatus nodeId={node.id} textSize="text-xs" />
                          ) : (
                            <span className="inline-flex items-center gap-1 text-xs text-slate-500"><DatabaseZap className="h-3 w-3" /> local wallet protection</span>
                          )}
                        </div>
                      </td>
                      <td className="sticky right-0 bg-white/95 px-4 py-4 shadow-[-10px_0_16px_-16px_rgba(15,23,42,0.45)] backdrop-blur transition-colors group-hover:bg-slate-50/95">
                        <div className="flex items-center justify-end gap-2">
                          {isRunning ? (
                            <SpinnerButton
                              onClick={() => stopNode.mutate({ id: node.id })}
                              loading={stopNode.isPending}
                              disabled={!canControlLifecycle}
                              className="btn btn-secondary p-2"
                              title={canControlLifecycle ? "Stop" : "Lifecycle locked by imported ownership mode"}
                              aria-label="Stop node"
                            >
                              <Square className="h-4 w-4" />
                            </SpinnerButton>
                          ) : (
                            <SpinnerButton
                              onClick={() => startNode.mutate(node.id)}
                              loading={startNode.isPending}
                              disabled={!canControlLifecycle || node.process.status === "starting"}
                              className="btn btn-success p-2"
                              title={canControlLifecycle ? "Start" : "Lifecycle locked by imported ownership mode"}
                              aria-label="Start node"
                            >
                              <Play className="h-4 w-4" />
                            </SpinnerButton>
                          )}
                          <SpinnerButton
                            onClick={() => setNodePendingDelete(node)}
                            loading={deleting === node.id}
                            className="btn btn-error p-2"
                            title="Delete registration"
                            aria-label="Delete node"
                          >
                            <Trash2 className="h-4 w-4" />
                          </SpinnerButton>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <ConfirmDialog
        open={Boolean(nodePendingDelete)}
        title="Delete node registration?"
        description="Managed node files are removed only when NeoNexus owns the node directory."
        confirmLabel="Delete node"
        isConfirming={deleteNode.isPending}
        onCancel={() => setNodePendingDelete(null)}
        onConfirm={() => void confirmDelete()}
      />
    </div>
  );
}
