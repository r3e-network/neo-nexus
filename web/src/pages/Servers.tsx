import { useEffect, useMemo, useState } from "react";
import { AlertCircle, Globe, Loader2, Network, Plus, RefreshCw, Server, Trash2 } from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { CardSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import { useCreateServer, useDeleteServer, useServers, useUpdateServer, type RemoteServerSummary } from "../hooks/useServers";

interface ServerFormState {
  name: string;
  baseUrl: string;
  description: string;
  enabled: boolean;
}

const EMPTY_FORM: ServerFormState = {
  name: "",
  baseUrl: "",
  description: "",
  enabled: true,
};

function formatNumber(value?: number) {
  return typeof value === "number" ? value.toLocaleString() : "0";
}

export default function Servers() {
  const { data: servers = [], isLoading, refetch, isFetching } = useServers();
  const createServer = useCreateServer();
  const updateServer = useUpdateServer();
  const deleteServer = useDeleteServer();
  const [selectedServerId, setSelectedServerId] = useState<string>("");
  const [form, setForm] = useState<ServerFormState>(EMPTY_FORM);
  const [formMode, setFormMode] = useState<"create" | "edit">("create");
  const [feedback, setFeedback] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!selectedServerId && servers[0]) {
      setSelectedServerId(servers[0].profile.id);
    }
  }, [servers, selectedServerId]);

  const selectedServer = useMemo(
    () => servers.find((server) => server.profile.id === selectedServerId) || null,
    [servers, selectedServerId],
  );

  const hydrateForm = (server: RemoteServerSummary | null) => {
    if (!server) {
      setForm(EMPTY_FORM);
      setFormMode("create");
      return;
    }

    setForm({
      name: server.profile.name,
      baseUrl: server.profile.baseUrl,
      description: server.profile.description || "",
      enabled: server.profile.enabled,
    });
    setFormMode("edit");
  };

  useEffect(() => {
    if (selectedServer) {
      hydrateForm(selectedServer);
    }
  }, [selectedServer, selectedServerId]);

  const handleSubmit = async () => {
    setFeedback("");
    setError("");

    if (!form.name.trim() || !form.baseUrl.trim()) {
      setError("Name and base URL are required.");
      return;
    }

    try {
      if (formMode === "create") {
        const created = await createServer.mutateAsync({
          name: form.name,
          baseUrl: form.baseUrl,
          description: form.description,
          enabled: form.enabled,
        });
        setSelectedServerId(created.id);
        setFeedback("Remote server profile created.");
      } else if (selectedServer) {
        await updateServer.mutateAsync({
          id: selectedServer.profile.id,
          payload: {
            name: form.name,
            baseUrl: form.baseUrl,
            description: form.description,
            enabled: form.enabled,
          },
        });
        setFeedback("Remote server profile updated.");
      }
    } catch (mutationError) {
      setError(mutationError instanceof Error ? mutationError.message : "Failed to save remote server profile.");
    }
  };

  const handleDelete = async () => {
    if (!selectedServer) {
      return;
    }

    if (!window.confirm(`Delete remote server profile "${selectedServer.profile.name}"?`)) {
      return;
    }

    setFeedback("");
    setError("");

    try {
      await deleteServer.mutateAsync(selectedServer.profile.id);
      setSelectedServerId("");
      setForm(EMPTY_FORM);
      setFormMode("create");
      setFeedback("Remote server profile deleted.");
    } catch (mutationError) {
      setError(mutationError instanceof Error ? mutationError.message : "Failed to delete remote server profile.");
    }
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white">Servers</h1>
          <p className="text-slate-400 mt-1">Monitor other NeoNexus instances through their public status endpoints.</p>
        </div>
        <button
          className="btn btn-secondary"
          onClick={() => refetch()}
          type="button"
        >
          <RefreshCw className={`w-4 h-4 ${isFetching ? "animate-spin" : ""}`} />
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-[360px_minmax(0,1fr)] gap-6">
        <div className="space-y-6">
          <div className="card space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-lg font-semibold text-white">{formMode === "create" ? "Add Server" : "Edit Server"}</h2>
                <p className="text-sm text-slate-400">Use the remote NeoNexus base URL that exposes `/api/public/*`.</p>
              </div>
              {formMode === "edit" && (
                <button
                  className="btn btn-secondary"
                  onClick={() => {
                    setSelectedServerId("");
                    setForm(EMPTY_FORM);
                    setFormMode("create");
                    setFeedback("");
                    setError("");
                  }}
                  type="button"
                >
                  <Plus className="w-4 h-4" />
                  New
                </button>
              )}
            </div>

            <FeedbackBanner error={error} success={feedback} />

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Server Name</label>
                <input
                  className="input"
                  value={form.name}
                  onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                  placeholder="Tokyo Node Manager"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Base URL</label>
                <input
                  className="input"
                  value={form.baseUrl}
                  onChange={(event) => setForm((current) => ({ ...current, baseUrl: event.target.value }))}
                  placeholder="https://tokyo.example.com"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Description</label>
                <textarea
                  className="input min-h-24"
                  value={form.description}
                  onChange={(event) => setForm((current) => ({ ...current, description: event.target.value }))}
                  placeholder="Region, purpose, or access notes"
                />
              </div>

              <label className="flex items-center gap-3 text-sm text-slate-300">
                <input
                  type="checkbox"
                  className="h-4 w-4"
                  checked={form.enabled}
                  onChange={(event) => setForm((current) => ({ ...current, enabled: event.target.checked }))}
                />
                Enable monitoring for this server
              </label>
            </div>

            <div className="flex gap-3">
              <button
                className="btn btn-primary flex-1 justify-center"
                disabled={createServer.isPending || updateServer.isPending}
                onClick={handleSubmit}
                type="button"
              >
                {(createServer.isPending || updateServer.isPending) && (
                  <Loader2 className="w-4 h-4 animate-spin" />
                )}
                {formMode === "create" ? "Create Profile" : "Save Changes"}
              </button>

              {formMode === "edit" && (
                <button
                  className="btn btn-error"
                  disabled={deleteServer.isPending}
                  onClick={handleDelete}
                  type="button"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>

          <div className="card">
            <h2 className="text-lg font-semibold text-white mb-4">Server Profiles</h2>
            {isLoading ? (
              <CardSkeleton count={2} />
            ) : servers.length === 0 ? (
              <EmptyState
                icon={Network}
                title="No remote servers"
                description="Add remote NeoNexus instances to monitor from here"
              />
            ) : (
              <div className="space-y-3">
                {servers.map((server) => (
                  <button
                    key={server.profile.id}
                    className={`w-full text-left rounded-lg border px-4 py-3 transition-all duration-200 ${
                      server.profile.id === selectedServerId
                        ? "border-blue-500 bg-blue-500/10 shadow-sm shadow-blue-500/10"
                        : "border-slate-700 bg-slate-800/50 hover:bg-slate-800 hover:border-slate-600"
                    }`}
                    onClick={() => {
                      setSelectedServerId(server.profile.id);
                      hydrateForm(server);
                    }}
                    type="button"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <p className="font-medium text-white">{server.profile.name}</p>
                        <p className="text-xs text-slate-400 mt-1">{server.profile.baseUrl}</p>
                      </div>
                      <span className={`status-badge ${server.reachable ? "status-running" : "status-error"}`}>
                        {server.reachable ? "reachable" : "offline"}
                      </span>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-6">
          {selectedServer ? (
            <>
              <div className="card">
                <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
                  <div>
                    <h2 className="text-xl font-semibold text-white">{selectedServer.profile.name}</h2>
                    <p className="text-sm text-slate-400 mt-1">{selectedServer.profile.description || selectedServer.profile.baseUrl}</p>
                  </div>
                  <a
                    href={selectedServer.profile.baseUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="btn btn-secondary"
                  >
                    <Globe className="w-4 h-4" />
                    Open Remote
                  </a>
                </div>

                {!selectedServer.reachable && (
                  <div className="mt-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-300 flex items-center gap-2">
                    <AlertCircle className="w-4 h-4 shrink-0" />
                    <span>{selectedServer.error || "Failed to reach remote server."}</span>
                  </div>
                )}
              </div>

              {selectedServer.reachable && selectedServer.status && (
                <div className="grid grid-cols-2 xl:grid-cols-4 gap-4">
                  <div className="card">
                    <p className="text-sm text-slate-400">Total Nodes</p>
                    <p className="text-2xl font-bold text-white mt-1">{selectedServer.status.totalNodes}</p>
                  </div>
                  <div className="card">
                    <p className="text-sm text-slate-400">Running</p>
                    <p className="text-2xl font-bold text-white mt-1">{selectedServer.status.runningNodes}</p>
                  </div>
                  <div className="card">
                    <p className="text-sm text-slate-400">Errors</p>
                    <p className="text-2xl font-bold text-white mt-1">{selectedServer.status.errorNodes}</p>
                  </div>
                  <div className="card">
                    <p className="text-sm text-slate-400">Blocks</p>
                    <p className="text-2xl font-bold text-white mt-1">{formatNumber(selectedServer.status.totalBlocks)}</p>
                  </div>
                </div>
              )}

              {selectedServer.systemMetrics && (
                <div className="card">
                  <h3 className="text-lg font-semibold text-white mb-4">Remote System Metrics</h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                      <p className="text-sm text-slate-400">CPU</p>
                      <p className="text-xl font-semibold text-white mt-1">{selectedServer.systemMetrics.cpu.usage.toFixed(1)}%</p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-400">Memory</p>
                      <p className="text-xl font-semibold text-white mt-1">{selectedServer.systemMetrics.memory.percentage.toFixed(1)}%</p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-400">Disk</p>
                      <p className="text-xl font-semibold text-white mt-1">{selectedServer.systemMetrics.disk.percentage.toFixed(1)}%</p>
                    </div>
                  </div>
                </div>
              )}

              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-4">Remote Nodes</h3>
                {!selectedServer.nodes || selectedServer.nodes.length === 0 ? (
                  <p className="text-sm text-slate-400">No remote nodes reported.</p>
                ) : (
                  <div className="space-y-3">
                    {selectedServer.nodes.map((node) => (
                      <div key={node.id} className="rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3">
                        <div className="flex items-center justify-between gap-3">
                          <div className="flex items-center gap-3">
                            <div className="w-9 h-9 rounded-lg bg-blue-500/10 flex items-center justify-center">
                              <Server className="w-4 h-4 text-blue-400" />
                            </div>
                            <div>
                              <p className="font-medium text-white">{node.name}</p>
                              <p className="text-xs text-slate-400">{node.type} • {node.network} • v{node.version}</p>
                            </div>
                          </div>
                          <span className={`status-badge status-${node.status === "running" ? "running" : node.status === "error" ? "error" : "stopped"}`}>
                            {node.status}
                          </span>
                        </div>

                        {node.metrics && (
                          <div className="mt-3 grid grid-cols-2 md:grid-cols-3 gap-3 text-sm">
                            <div>
                              <p className="text-slate-400">Block Height</p>
                              <p className="text-white">{formatNumber(node.metrics.blockHeight)}</p>
                            </div>
                            <div>
                              <p className="text-slate-400">Peers</p>
                              <p className="text-white">{node.metrics.connectedPeers}</p>
                            </div>
                            <div>
                              <p className="text-slate-400">Sync</p>
                              <p className="text-white">{node.metrics.syncProgress?.toFixed(1) ?? "0.0"}%</p>
                            </div>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="card">
              <p className="text-sm text-slate-400">Select a remote server profile to inspect its status.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
