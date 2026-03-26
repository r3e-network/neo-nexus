import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { Check, Info, Link2, Puzzle, Server, Settings2, Trash2 } from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { useNodes } from "../hooks/useNodes";
import {
  useAvailablePlugins,
  useInstallPlugin,
  useNodePlugins,
  useSetPluginEnabled,
  useUninstallPlugin,
  useUpdatePlugin,
} from "../hooks/usePlugins";

const CATEGORIES: Record<string, string> = {
  Core: "bg-blue-500/10 text-blue-400",
  Storage: "bg-emerald-500/10 text-emerald-400",
  API: "bg-purple-500/10 text-purple-400",
  Tooling: "bg-orange-500/10 text-orange-400",
};

const DEFAULT_PLUGIN_IDS = new Set(["RpcServer", "LevelDBStore"]);

function stringifyConfig(config?: Record<string, unknown>) {
  return JSON.stringify(config || {}, null, 2);
}

export default function Plugins() {
  const { data: nodes = [] } = useNodes();
  const neoCliNodes = useMemo(() => nodes.filter((node) => node.type === "neo-cli"), [nodes]);
  const [selectedNodeId, setSelectedNodeId] = useState("");
  const [configDrafts, setConfigDrafts] = useState<Record<string, string>>({});
  const [feedback, setFeedback] = useState<{ type: "error" | "success"; message: string } | null>(null);
  const selectedNode = neoCliNodes.find((node) => node.id === selectedNodeId) || null;

  useEffect(() => {
    if (!selectedNodeId && neoCliNodes[0]) {
      setSelectedNodeId(neoCliNodes[0].id);
      return;
    }

    if (selectedNodeId && !neoCliNodes.some((node) => node.id === selectedNodeId)) {
      setSelectedNodeId(neoCliNodes[0]?.id || "");
    }
  }, [neoCliNodes, selectedNodeId]);

  const { data: availablePlugins = [], isLoading: isLoadingAvailable } = useAvailablePlugins(selectedNodeId || undefined);
  const { data: installedPlugins = [], isLoading: isLoadingInstalled } = useNodePlugins(selectedNodeId || undefined);

  const prevNodeIdRef = useRef(selectedNodeId);
  useEffect(() => {
    // Only hydrate config drafts when the selected node changes, not on every data refetch
    if (prevNodeIdRef.current === selectedNodeId && Object.keys(configDrafts).length > 0) {
      prevNodeIdRef.current = selectedNodeId;
      return;
    }
    prevNodeIdRef.current = selectedNodeId;

    if (!selectedNodeId) {
      setConfigDrafts({});
      return;
    }

    const nextDrafts: Record<string, string> = {};
    availablePlugins.forEach((plugin) => {
      const installed = installedPlugins.find((entry) => entry.id === plugin.id);
      nextDrafts[plugin.id] = stringifyConfig(installed?.config || plugin.defaultConfig);
    });
    setConfigDrafts(nextDrafts);
  }, [availablePlugins, installedPlugins, selectedNodeId]);

  const installPlugin = useInstallPlugin(selectedNodeId || undefined);
  const updatePlugin = useUpdatePlugin(selectedNodeId || undefined);
  const uninstallPlugin = useUninstallPlugin(selectedNodeId || undefined);
  const setPluginEnabled = useSetPluginEnabled(selectedNodeId || undefined);

  const installedById = new Map(installedPlugins.map((plugin) => [plugin.id, plugin]));

  const parseConfig = (pluginId: string) => {
    const raw = configDrafts[pluginId];

    if (!raw?.trim()) {
      return {};
    }

    try {
      return JSON.parse(raw) as Record<string, unknown>;
    } catch {
      throw new Error(`Invalid JSON config for ${pluginId}.`);
    }
  };

  const handleInstall = async (pluginId: string) => {
    try {
      setFeedback(null);
      await installPlugin.mutateAsync({
        pluginId,
        config: parseConfig(pluginId),
      });
      setFeedback({ type: "success", message: `${pluginId} installed on ${selectedNode?.name}.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Failed to install plugin." });
    }
  };

  const handleUpdate = async (pluginId: string) => {
    try {
      setFeedback(null);
      await updatePlugin.mutateAsync({
        pluginId,
        config: parseConfig(pluginId),
      });
      setFeedback({ type: "success", message: `${pluginId} configuration updated.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Failed to update plugin." });
    }
  };

  const handleUninstall = async (pluginId: string) => {
    try {
      setFeedback(null);
      await uninstallPlugin.mutateAsync(pluginId);
      setFeedback({ type: "success", message: `${pluginId} removed from ${selectedNode?.name}.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Failed to uninstall plugin." });
    }
  };

  const handleToggleEnabled = async (pluginId: string, enabled: boolean) => {
    try {
      setFeedback(null);
      await setPluginEnabled.mutateAsync({ pluginId, enabled });
      setFeedback({ type: "success", message: `${pluginId} ${enabled ? "enabled" : "disabled"}.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Failed to update plugin state." });
    }
  };

  if (neoCliNodes.length === 0) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Plugins</h1>
          <p className="text-slate-400 mt-1">Plugin management is available for neo-cli nodes.</p>
        </div>

        <div className="card">
          <div className="flex items-start gap-4">
            <div className="rounded-xl bg-blue-500/10 p-3">
              <Puzzle className="h-6 w-6 text-blue-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-lg font-semibold text-white">No neo-cli nodes available</h2>
              <p className="text-sm text-slate-400">
                Create or import a neo-cli node first, then return here to install API, storage, and tooling plugins.
              </p>
              <Link to="/nodes/create" className="btn btn-primary">
                <Server className="w-4 h-4" />
                Create Node
              </Link>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Plugins</h1>
        <p className="text-slate-400 mt-1">Install, configure, enable, and remove neo-cli plugins per node.</p>
      </div>

      <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <Info className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <p className="text-blue-300 text-sm">
              Select a neo-cli node, stop it before changing plugin binaries, then install or update the plugins below.
            </p>
            {selectedNode && (
              <p className="text-xs text-blue-200/80">
                Active node: <span className="font-medium">{selectedNode.name}</span> · status {selectedNode.process.status}
              </p>
            )}
          </div>
        </div>
      </div>

      <div className="card flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Target Node</h2>
          <p className="text-sm text-slate-400">Installed plugin state is scoped to the selected neo-cli node.</p>
        </div>
        <div className="w-full max-w-md">
          <select
            className="select"
            value={selectedNodeId}
            onChange={(event) => setSelectedNodeId(event.target.value)}
          >
            {neoCliNodes.map((node) => (
              <option key={node.id} value={node.id}>
                {node.name} · {node.network} · {node.process.status}
              </option>
            ))}
          </select>
        </div>
      </div>

      <FeedbackBanner
        error={feedback?.type === "error" ? feedback.message : undefined}
        success={feedback?.type === "success" ? feedback.message : undefined}
      />

      <div className="card">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold text-white">Installed On {selectedNode?.name}</h2>
            <p className="text-sm text-slate-400">Quick summary of the plugin set currently attached to this node.</p>
          </div>
          <div className="text-sm text-slate-400">{installedPlugins.length} installed</div>
        </div>

        {isLoadingInstalled ? (
          <div className="mt-4 text-sm text-slate-400">Loading installed plugins...</div>
        ) : installedPlugins.length === 0 ? (
          <div className="mt-4 rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3 text-sm text-slate-400">
            No plugins installed yet.
          </div>
        ) : (
          <div className="mt-4 flex flex-wrap gap-2">
            {installedPlugins.map((plugin) => (
              <span key={plugin.id} className="inline-flex items-center gap-2 rounded-full bg-slate-800 px-3 py-1 text-sm text-slate-200">
                <Link2 className="h-3.5 w-3.5 text-blue-400" />
                {plugin.id}
                <span className={plugin.enabled ? "text-emerald-400" : "text-slate-500"}>
                  {plugin.enabled ? "enabled" : "disabled"}
                </span>
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="card">
        <h2 className="text-lg font-semibold text-white mb-4">Available Plugins</h2>

        {isLoadingAvailable ? (
          <div className="text-sm text-slate-400">Loading plugin catalog...</div>
        ) : (
          <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
            {availablePlugins.map((plugin) => {
              const installed = installedById.get(plugin.id);

              return (
                <div key={plugin.id} className="rounded-xl border border-slate-700/60 bg-slate-900/70 p-5 space-y-4">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex items-start gap-4">
                      <div className="w-10 h-10 rounded-lg bg-slate-700 flex items-center justify-center shrink-0">
                        <Puzzle className="w-5 h-5 text-slate-300" />
                      </div>
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <h3 className="font-medium text-white">{plugin.name}</h3>
                          <span className={`text-xs px-2 py-0.5 rounded-full ${CATEGORIES[plugin.category]}`}>
                            {plugin.category}
                          </span>
                          {DEFAULT_PLUGIN_IDS.has(plugin.id) && (
                            <span className="rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-300">Default</span>
                          )}
                          {installed && (
                            <span className="rounded-full bg-blue-500/10 px-2 py-0.5 text-xs text-blue-300">Installed</span>
                          )}
                        </div>
                        <p className="text-sm text-slate-400 mt-1">{plugin.description}</p>
                      </div>
                    </div>

                    {installed && (
                      <button
                        className={`rounded-full px-3 py-1 text-xs font-medium ${
                          installed.enabled
                            ? "bg-emerald-500/10 text-emerald-300"
                            : "bg-slate-700 text-slate-300"
                        }`}
                        onClick={() => handleToggleEnabled(plugin.id, !installed.enabled)}
                        type="button"
                      >
                        {installed.enabled ? "Disable" : "Enable"}
                      </button>
                    )}
                  </div>

                  <div className="grid grid-cols-1 gap-3 md:grid-cols-2 text-sm text-slate-400">
                    <div className="rounded-lg bg-slate-800/70 px-3 py-2">
                      <span className="text-slate-500">Requires Config</span>
                      <p className="mt-1 text-white">{plugin.requiresConfig ? "Yes" : "No"}</p>
                    </div>
                    <div className="rounded-lg bg-slate-800/70 px-3 py-2">
                      <span className="text-slate-500">Installed Version</span>
                      <p className="mt-1 text-white">{installed?.version || "Not installed"}</p>
                    </div>
                  </div>

                  <div>
                    <label className="mb-2 block text-sm font-medium text-slate-300">
                      {plugin.requiresConfig ? "Plugin Config (JSON)" : "Optional Config Override (JSON)"}
                    </label>
                    <textarea
                      value={configDrafts[plugin.id] || "{}"}
                      onChange={(event) =>
                        setConfigDrafts((current) => ({
                          ...current,
                          [plugin.id]: event.target.value,
                        }))
                      }
                      className="input min-h-40 font-mono text-xs"
                      spellCheck={false}
                    />
                  </div>

                  <div className="flex flex-wrap gap-3">
                    {installed ? (
                      <>
                        <button className="btn btn-primary" onClick={() => handleUpdate(plugin.id)} type="button">
                          <Settings2 className="w-4 h-4" />
                          Save Config
                        </button>
                        <button className="btn btn-error" onClick={() => handleUninstall(plugin.id)} type="button">
                          <Trash2 className="w-4 h-4" />
                          Uninstall
                        </button>
                      </>
                    ) : (
                      <button className="btn btn-success" onClick={() => handleInstall(plugin.id)} type="button">
                        <Check className="w-4 h-4" />
                        Install On Node
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
