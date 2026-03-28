import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { Info, Puzzle, Server } from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { useNodes } from "../hooks/useNodes";
import {
  useAvailablePlugins,
  useInstallPlugin,
  useNodePlugins,
  useUninstallPlugin,
  useUpdatePlugin,
} from "../hooks/usePlugins";
import { getPluginMeta } from "../utils/pluginMeta";
import { PluginCard } from "./plugins/PluginCard";

// ── Plugins page ───────────────────────────────────────────────────────
export default function Plugins() {
  const { data: nodes = [] } = useNodes();
  const neoCliNodes = useMemo(() => nodes.filter((node) => node.type === "neo-cli"), [nodes]);
  const [selectedNodeId, setSelectedNodeId] = useState("");
  const [configDrafts, setConfigDrafts] = useState<Record<string, Record<string, unknown>>>({});
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
  const { data: installedPlugins = [] } = useNodePlugins(selectedNodeId || undefined);

  const prevNodeIdRef = useRef(selectedNodeId);
  useEffect(() => {
    if (prevNodeIdRef.current === selectedNodeId && Object.keys(configDrafts).length > 0) {
      prevNodeIdRef.current = selectedNodeId;
      return;
    }
    prevNodeIdRef.current = selectedNodeId;

    if (!selectedNodeId) {
      setConfigDrafts({});
      return;
    }

    const nextDrafts: Record<string, Record<string, unknown>> = {};
    availablePlugins.forEach((plugin) => {
      const installed = installedPlugins.find((entry) => entry.id === plugin.id);
      const meta = getPluginMeta(plugin.id);
      const base: Record<string, unknown> = {};

      // Populate defaults from meta, then overlay installed/default config
      for (const field of meta.configFields) {
        base[field.key] = field.defaultValue;
      }

      const existing = installed?.config || plugin.defaultConfig || {};
      nextDrafts[plugin.id] = { ...base, ...existing };
    });
    setConfigDrafts(nextDrafts);
  }, [availablePlugins, installedPlugins, selectedNodeId]);

  const installPlugin = useInstallPlugin(selectedNodeId || undefined);
  const updatePlugin = useUpdatePlugin(selectedNodeId || undefined);
  const uninstallPlugin = useUninstallPlugin(selectedNodeId || undefined);

  const installedById = new Map(installedPlugins.map((plugin) => [plugin.id, plugin]));

  const handleToggle = async (pluginId: string) => {
    const isInstalled = installedById.has(pluginId);
    try {
      setFeedback(null);
      const meta = getPluginMeta(pluginId);
      if (isInstalled) {
        await uninstallPlugin.mutateAsync(pluginId);
        setFeedback({ type: "success", message: `${meta.featureName} removed from ${selectedNode?.name}.` });
      } else {
        await installPlugin.mutateAsync({
          pluginId,
          config: configDrafts[pluginId] || {},
        });
        setFeedback({ type: "success", message: `${meta.featureName} installed on ${selectedNode?.name}.` });
      }
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Operation failed." });
    }
  };

  const handleSaveConfig = async (pluginId: string) => {
    try {
      setFeedback(null);
      const meta = getPluginMeta(pluginId);
      await updatePlugin.mutateAsync({
        pluginId,
        config: configDrafts[pluginId] || {},
      });
      setFeedback({ type: "success", message: `${meta.featureName} configuration saved.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Failed to save configuration." });
    }
  };

  const handleConfigChange = (pluginId: string, key: string, value: unknown) => {
    setConfigDrafts((current) => ({
      ...current,
      [pluginId]: { ...(current[pluginId] || {}), [key]: value },
    }));
  };

  // ── Group plugins by purpose ──
  const corePlugins = availablePlugins.filter((p) => p.category === "Core");
  const apiPlugins = availablePlugins.filter((p) => p.category === "API");
  const storagePlugins = availablePlugins.filter((p) => p.category === "Storage");
  const toolingPlugins = availablePlugins.filter((p) => p.category === "Tooling");

  const sections = [
    { title: "Network & API", description: "Expose your node to wallets, dApps, and monitoring tools", plugins: apiPlugins },
    { title: "Core Services", description: "Consensus, state tracking, oracles, and execution logging", plugins: corePlugins },
    { title: "Storage Engine", description: "Choose the underlying database for blockchain data", plugins: storagePlugins },
    { title: "Tooling", description: "Wallets, signing, and data migration utilities", plugins: toolingPlugins },
  ].filter((s) => s.plugins.length > 0);

  // ── Empty state ──
  if (neoCliNodes.length === 0) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Node Features</h1>
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
                Create or import a neo-cli node first, then return here to enable features.
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
        <h1 className="text-2xl font-bold text-white">Node Features</h1>
        <p className="text-slate-400 mt-1">
          Enable and configure capabilities for your neo-cli node. Each feature installs and configures the corresponding plugin automatically.
        </p>
      </div>

      {/* Info banner */}
      <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <Info className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <p className="text-blue-300 text-sm">
              Stop your node before toggling features. Changes take effect on the next start.
            </p>
            {selectedNode && (
              <p className="text-xs text-blue-200/80">
                Active node: <span className="font-medium">{selectedNode.name}</span> · {selectedNode.process.status}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Node selector */}
      <div className="card flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Target Node</h2>
          <p className="text-sm text-slate-400">Features are managed per node.</p>
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

      {/* Feature sections */}
      {isLoadingAvailable ? (
        <div className="card text-sm text-slate-400">Loading features...</div>
      ) : (
        sections.map((section) => (
          <div key={section.title} className="space-y-4">
            <div>
              <h2 className="text-lg font-semibold text-white">{section.title}</h2>
              <p className="text-sm text-slate-400">{section.description}</p>
            </div>

            <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
              {section.plugins.map((plugin) => (
                <PluginCard
                  key={plugin.id}
                  plugin={plugin}
                  installed={installedById.get(plugin.id)}
                  configValues={configDrafts[plugin.id] || {}}
                  onConfigChange={(key, value) => handleConfigChange(plugin.id, key, value)}
                  onToggle={() => handleToggle(plugin.id)}
                  onSaveConfig={() => handleSaveConfig(plugin.id)}
                  isSaving={updatePlugin.isPending}
                />
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  );
}
