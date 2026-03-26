import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import {
  Coins,
  CloudCog,
  Database,
  GitBranch,
  Globe,
  HelpCircle,
  Info,
  Puzzle,
  ScrollText,
  Server,
  ShieldCheck,
  Vote,
  Wallet,
  Wrench,
} from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { useNodes } from "../hooks/useNodes";
import {
  useAvailablePlugins,
  useInstallPlugin,
  useNodePlugins,
  useUninstallPlugin,
  useUpdatePlugin,
  type PluginDefinition,
  type InstalledPlugin,
} from "../hooks/usePlugins";
import { getPluginMeta, type PluginConfigField } from "../utils/pluginMeta";

// ── Icon resolver ──────────────────────────────────────────────────────
const ICON_COMPONENTS: Record<string, React.ElementType> = {
  api: Globe,
  logs: ScrollText,
  consensus: Vote,
  state: GitBranch,
  oracle: CloudCog,
  tokens: Coins,
  storage: Database,
  wallet: Wallet,
  tools: Wrench,
  security: ShieldCheck,
  plugin: Puzzle,
};

function PluginIcon({ icon, className }: { icon: string; className?: string }) {
  const Component = ICON_COMPONENTS[icon] || Puzzle;
  return <Component className={className} />;
}

// ── Tooltip ────────────────────────────────────────────────────────────
function HelpTooltip({ text }: { text: string }) {
  const [open, setOpen] = useState(false);

  return (
    <span className="relative inline-flex">
      <button
        type="button"
        className="text-slate-500 hover:text-slate-300 transition-colors"
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        onClick={() => setOpen((v) => !v)}
        aria-label="Help"
      >
        <HelpCircle className="w-3.5 h-3.5" />
      </button>
      {open && (
        <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-64 rounded-lg bg-slate-800 border border-slate-700 p-3 text-xs text-slate-300 shadow-xl z-50 animate-slide-in-right">
          {text}
          <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-px border-4 border-transparent border-t-slate-700" />
        </div>
      )}
    </span>
  );
}

// ── Toggle switch ──────────────────────────────────────────────────────
function ToggleSwitch({
  checked,
  onChange,
  disabled,
  label,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
  label?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400 ${
        disabled
          ? "opacity-50 cursor-not-allowed"
          : "cursor-pointer"
      } ${checked ? "bg-emerald-500" : "bg-slate-600"}`}
    >
      <span
        className={`inline-block h-4 w-4 rounded-full bg-white transition-transform ${
          checked ? "translate-x-6" : "translate-x-1"
        }`}
      />
    </button>
  );
}

// ── Config field renderer ──────────────────────────────────────────────
function ConfigField({
  field,
  value,
  onChange,
}: {
  field: PluginConfigField;
  value: unknown;
  onChange: (key: string, value: unknown) => void;
}) {
  const id = `config-${field.key}`;

  return (
    <div>
      <div className="flex items-center gap-1.5 mb-1.5">
        <label htmlFor={id} className="text-sm font-medium text-slate-300">
          {field.label}
        </label>
        <HelpTooltip text={field.help} />
      </div>

      {field.type === "boolean" ? (
        <div className="flex items-center gap-3 rounded-lg bg-slate-800/60 px-4 py-2.5">
          <ToggleSwitch
            checked={value === true || value === "true"}
            onChange={(v) => onChange(field.key, v)}
            label={field.label}
          />
          <span className="text-sm text-slate-400">
            {value === true || value === "true" ? "Enabled" : "Disabled"}
          </span>
        </div>
      ) : field.type === "select" ? (
        <select
          id={id}
          value={String(value ?? field.defaultValue ?? "")}
          onChange={(e) => onChange(field.key, e.target.value)}
          className="input"
        >
          {field.options?.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      ) : (
        <input
          id={id}
          type={field.type === "number" ? "number" : "text"}
          value={String(value ?? field.defaultValue ?? "")}
          onChange={(e) =>
            onChange(
              field.key,
              field.type === "number"
                ? e.target.value === ""
                  ? undefined
                  : Number(e.target.value)
                : e.target.value,
            )
          }
          placeholder={field.placeholder || String(field.defaultValue ?? "")}
          className="input"
        />
      )}
    </div>
  );
}

// ── Plugin card ────────────────────────────────────────────────────────
function PluginCard({
  plugin,
  installed,
  configValues,
  onConfigChange,
  onToggle,
  onSaveConfig,
  isSaving,
}: {
  plugin: PluginDefinition;
  installed: InstalledPlugin | undefined;
  configValues: Record<string, unknown>;
  onConfigChange: (key: string, value: unknown) => void;
  onToggle: () => void;
  onSaveConfig: () => void;
  isSaving: boolean;
}) {
  const meta = getPluginMeta(plugin.id, plugin.name);
  const isActive = !!installed;
  const [showAdvanced, setShowAdvanced] = useState(false);

  const basicFields = meta.configFields.filter((f) => !f.advanced);
  const advancedFields = meta.configFields.filter((f) => f.advanced);
  const hasConfig = meta.configFields.length > 0;
  const configChanged = isActive && hasConfig;

  return (
    <div
      className={`rounded-xl border p-5 transition-colors ${
        isActive
          ? "border-emerald-500/30 bg-emerald-500/5"
          : "border-slate-700/60 bg-slate-900/70"
      }`}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-3.5">
          <div
            className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${
              isActive ? "bg-emerald-500/10" : "bg-slate-700/80"
            }`}
          >
            <PluginIcon
              icon={meta.icon}
              className={`w-5 h-5 ${isActive ? "text-emerald-400" : "text-slate-400"}`}
            />
          </div>
          <div>
            <h3 className="font-semibold text-white">{meta.featureName}</h3>
            <p className="text-sm text-slate-400 mt-0.5">{meta.summary}</p>
          </div>
        </div>

        <ToggleSwitch
          checked={isActive}
          onChange={onToggle}
          label={`Toggle ${meta.featureName}`}
        />
      </div>

      {/* Install note */}
      {!isActive && (
        <div className="mt-3 rounded-lg bg-slate-800/50 px-3 py-2 text-xs text-slate-500">
          <span className="text-slate-400 font-medium">When enabled:</span> {meta.installNote}
        </div>
      )}

      {/* Active: show config & status */}
      {isActive && (
        <div className="mt-4 space-y-4">
          {/* Enable/Disable within installed */}
          {installed && !installed.enabled && (
            <div className="rounded-lg bg-amber-500/10 border border-amber-500/20 px-3 py-2 text-xs text-amber-300">
              Plugin is installed but currently disabled. The toggle above will uninstall it. Use the node configuration to re-enable.
            </div>
          )}

          {/* Config fields */}
          {hasConfig && (
            <div className="space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {basicFields.map((field) => (
                  <ConfigField
                    key={field.key}
                    field={field}
                    value={configValues[field.key]}
                    onChange={onConfigChange}
                  />
                ))}
              </div>

              {advancedFields.length > 0 && (
                <>
                  <button
                    type="button"
                    onClick={() => setShowAdvanced(!showAdvanced)}
                    className="text-xs text-slate-500 hover:text-slate-300 transition-colors"
                  >
                    {showAdvanced ? "Hide" : "Show"} advanced settings ({advancedFields.length})
                  </button>

                  {showAdvanced && (
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
                      {advancedFields.map((field) => (
                        <ConfigField
                          key={field.key}
                          field={field}
                          value={configValues[field.key]}
                          onChange={onConfigChange}
                        />
                      ))}
                    </div>
                  )}
                </>
              )}

              {configChanged && (
                <div className="flex justify-end">
                  <button
                    type="button"
                    className="btn btn-primary"
                    onClick={onSaveConfig}
                    disabled={isSaving}
                  >
                    {isSaving ? "Saving..." : "Save Configuration"}
                  </button>
                </div>
              )}
            </div>
          )}

          {!hasConfig && (
            <p className="text-xs text-slate-500">No configuration needed — this plugin works out of the box.</p>
          )}
        </div>
      )}
    </div>
  );
}

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
