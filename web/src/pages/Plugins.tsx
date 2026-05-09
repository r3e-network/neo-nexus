import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { AlertTriangle, Info, Puzzle, Server, ShieldCheck } from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { CardSkeleton } from "../components/LoadingSkeleton";
import { type Node, useNodes } from "../hooks/useNodes";
import { nodeOwnershipLabel } from "../utils/nodeKind";
import {
  type InstalledPlugin,
  type PluginDefinition,
  useAvailablePlugins,
  useInstallPlugin,
  useNodePlugins,
  useSetPluginEnabled,
  useUninstallPlugin,
  useUpdatePlugin,
} from "../hooks/usePlugins";
import { getPluginMeta } from "../utils/pluginMeta";
import { PluginCard } from "./plugins/PluginCard";
import { useAuth } from "../hooks/useAuth";

const EMPTY_NODES: Node[] = [];
const EMPTY_PLUGINS: PluginDefinition[] = [];
const EMPTY_INSTALLED_PLUGINS: InstalledPlugin[] = [];

// ── Plugins page ───────────────────────────────────────────────────────
export default function Plugins() {
  const { user } = useAuth();
  const { data: nodesData } = useNodes();
  const nodes = nodesData ?? EMPTY_NODES;
  const neoCliNodes = useMemo(() => nodes.filter((node) => node.type === "neo-cli"), [nodes]);
  const [selectedNodeId, setSelectedNodeId] = useState("");
  const [configDrafts, setConfigDrafts] = useState<Record<string, Record<string, unknown>>>({});
  const [feedback, setFeedback] = useState<{ type: "error" | "success"; message: string } | null>(null);
  const isAdmin = user?.role === "admin";
  const selectedNode = neoCliNodes.find((node) => node.id === selectedNodeId) || null;
  const selectedNodeOwnership = selectedNode?.settings?.import?.ownershipMode;
  const pluginMutationDisabledReason = !isAdmin
    ? "Admin access is required to change node plugins."
    : selectedNodeOwnership === "observe-only"
      ? "This imported node is observe-only. Plugin changes are blocked until ownership is explicitly upgraded."
      : undefined;

  useEffect(() => {
    if (!selectedNodeId && neoCliNodes[0]) {
      setSelectedNodeId(neoCliNodes[0].id);
      return;
    }
    if (selectedNodeId && !neoCliNodes.some((node) => node.id === selectedNodeId)) {
      setSelectedNodeId(neoCliNodes[0]?.id || "");
    }
  }, [neoCliNodes, selectedNodeId]);

  const { data: availablePluginsData, isLoading: isLoadingAvailable } = useAvailablePlugins(selectedNodeId || undefined);
  const { data: installedPluginsData } = useNodePlugins(selectedNodeId || undefined);
  const availablePlugins = availablePluginsData ?? EMPTY_PLUGINS;
  const installedPlugins = installedPluginsData ?? EMPTY_INSTALLED_PLUGINS;

  const prevNodeIdRef = useRef(selectedNodeId);
  useEffect(() => {
    if (prevNodeIdRef.current === selectedNodeId && Object.keys(configDrafts).length > 0) {
      prevNodeIdRef.current = selectedNodeId;
      return;
    }
    prevNodeIdRef.current = selectedNodeId;

    if (!selectedNodeId) {
      setConfigDrafts((current) => (Object.keys(current).length === 0 ? current : {}));
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [availablePlugins, installedPlugins, selectedNodeId]);

  const installPlugin = useInstallPlugin(selectedNodeId || undefined);
  const updatePlugin = useUpdatePlugin(selectedNodeId || undefined);
  const uninstallPlugin = useUninstallPlugin(selectedNodeId || undefined);
  const setPluginEnabled = useSetPluginEnabled(selectedNodeId || undefined);

  const installedById = new Map(installedPlugins.map((plugin) => [plugin.id, plugin]));

  const handleInstall = async (pluginId: string) => {
    if (pluginMutationDisabledReason) {
      setFeedback({ type: "error", message: pluginMutationDisabledReason });
      return;
    }

    try {
      setFeedback(null);
      const meta = getPluginMeta(pluginId);
      await installPlugin.mutateAsync({
        pluginId,
        config: configDrafts[pluginId] || {},
      });
      setFeedback({ type: "success", message: `${meta.featureName} installed on ${selectedNode?.name}.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Operation failed." });
    }
  };

  const handleRemove = async (pluginId: string) => {
    if (pluginMutationDisabledReason) {
      setFeedback({ type: "error", message: pluginMutationDisabledReason });
      return;
    }

    try {
      setFeedback(null);
      const meta = getPluginMeta(pluginId);
      await uninstallPlugin.mutateAsync(pluginId);
      setFeedback({ type: "success", message: `${meta.featureName} removed from ${selectedNode?.name}.` });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Operation failed." });
    }
  };

  const handleSetEnabled = async (pluginId: string, enabled: boolean) => {
    if (pluginMutationDisabledReason) {
      setFeedback({ type: "error", message: pluginMutationDisabledReason });
      return;
    }

    try {
      setFeedback(null);
      const meta = getPluginMeta(pluginId);
      await setPluginEnabled.mutateAsync({ pluginId, enabled });
      setFeedback({
        type: "success",
        message: `${meta.featureName} ${enabled ? "enabled" : "disabled"} on ${selectedNode?.name}.`,
      });
    } catch (error) {
      setFeedback({ type: "error", message: error instanceof Error ? error.message : "Operation failed." });
    }
  };

  const handleSaveConfig = async (pluginId: string) => {
    if (pluginMutationDisabledReason) {
      setFeedback({ type: "error", message: pluginMutationDisabledReason });
      return;
    }

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
      <div className="space-y-7">
        <div>
          <h1 className="text-2xl font-bold text-slate-950">Node Features</h1>
          <p className="text-slate-600 mt-1">Plugin management is available for neo-cli nodes.</p>
        </div>
        <div className="card">
          <div className="flex items-start gap-4">
            <div className="rounded-lg bg-teal-500/10 p-3">
              <Puzzle className="h-6 w-6 text-blue-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-lg font-semibold text-slate-950">No neo-cli nodes available</h2>
              <p className="text-sm text-slate-600">
                Create or import a neo-cli node first, then return here to enable features.
              </p>
              {isAdmin && (
                <Link to="/nodes/create" className="btn btn-primary">
                  <Server className="w-4 h-4" />
                  Create Node
                </Link>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-7">
      <section className="page-hero pb-5">
        <div>
        <p className="console-kicker">Plugin catalog</p>
        <h1 className="mt-2 text-3xl font-semibold text-slate-950">Node plugins</h1>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
          Enable RPC surfaces, storage engines, monitoring, wallets and tooling per neo-cli node. Ownership guards prevent accidental mutations on observe-only imports.
        </p>
        </div>
      </section>

      {/* Info banner */}
      <div className="rounded-lg border border-blue-200 bg-blue-50 p-4">
        <div className="flex items-start gap-3">
          <Info className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <p className="text-blue-800 text-sm">
              Stop your node before toggling features. Changes take effect on the next start.
            </p>
            {selectedNode && (
              <p className="text-xs text-blue-700">
                Active node: <span className="font-medium">{selectedNode.name}</span> · {selectedNode.process.status}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Node selector */}
      <div className="card flex flex-col gap-5 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-slate-950">Target Node</h2>
          <p className="text-sm text-slate-600">Features are managed per node.</p>
          {selectedNode && (
            <p className="mt-2 inline-flex items-center gap-2 text-xs text-slate-500">
              <ShieldCheck className="h-3.5 w-3.5 text-emerald-600" />
              Ownership: {nodeOwnershipLabel(selectedNode)}
            </p>
          )}
        </div>
        <div className="flex w-full max-w-md items-center gap-2">
          <select
            className="select flex-1"
            value={selectedNodeId}
            onChange={(event) => setSelectedNodeId(event.target.value)}
          >
            {neoCliNodes.map((node) => (
              <option key={node.id} value={node.id}>
                {node.name} · {node.network} · {node.process.status}
              </option>
            ))}
          </select>
          {selectedNodeId && (
            <Link
              to={`/nodes/${selectedNodeId}`}
              className="text-sm font-medium text-teal-700 hover:text-teal-800 whitespace-nowrap"
              title="Open this node's detail page"
            >
              View node →
            </Link>
          )}
        </div>
      </div>

      {pluginMutationDisabledReason && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800">
          <div className="flex items-start gap-3">
            <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-600" />
            <p>{pluginMutationDisabledReason}</p>
          </div>
        </div>
      )}

      <FeedbackBanner
        error={feedback?.type === "error" ? feedback.message : undefined}
        success={feedback?.type === "success" ? feedback.message : undefined}
      />

      {/* Feature sections */}
      {isLoadingAvailable ? (
        <CardSkeleton count={4} />
      ) : (
        sections.map((section) => (
          <div key={section.title} className="space-y-4">
            <div>
              <h2 className="text-lg font-semibold text-slate-950">{section.title}</h2>
              <p className="text-sm text-slate-600">{section.description}</p>
            </div>

            <div className="grid grid-cols-1 2xl:grid-cols-2 gap-4">
              {section.plugins.map((plugin) => (
                <PluginCard
                  key={plugin.id}
                  plugin={plugin}
                  installed={installedById.get(plugin.id)}
                  configValues={configDrafts[plugin.id] || {}}
                  onConfigChange={(key, value) => handleConfigChange(plugin.id, key, value)}
                  onInstall={() => handleInstall(plugin.id)}
                  onRemove={() => handleRemove(plugin.id)}
                  onSetEnabled={(enabled) => handleSetEnabled(plugin.id, enabled)}
                  onSaveConfig={() => handleSaveConfig(plugin.id)}
                  isInstalling={installPlugin.isPending}
                  isRemoving={uninstallPlugin.isPending}
                  isSettingEnabled={setPluginEnabled.isPending}
                  isSaving={updatePlugin.isPending}
                  disabledReason={pluginMutationDisabledReason}
                />
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  );
}
