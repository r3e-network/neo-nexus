import { useState } from "react";
import {
  Coins,
  CloudCog,
  Database,
  GitBranch,
  Globe,
  Puzzle,
  ScrollText,
  ShieldCheck,
  Vote,
  Wallet,
  Wrench,
} from "lucide-react";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import { type PluginDefinition, type InstalledPlugin } from "../../hooks/usePlugins";
import { getPluginMeta } from "../../utils/pluginMeta";
import { ConfigField } from "./ConfigField";

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

// ── Plugin card ────────────────────────────────────────────────────────
export interface PluginCardProps {
  plugin: PluginDefinition;
  installed: InstalledPlugin | undefined;
  configValues: Record<string, unknown>;
  onConfigChange: (key: string, value: unknown) => void;
  onToggle: () => void;
  onSaveConfig: () => void;
  isSaving: boolean;
}

export function PluginCard({
  plugin,
  installed,
  configValues,
  onConfigChange,
  onToggle,
  onSaveConfig,
  isSaving,
}: PluginCardProps) {
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
