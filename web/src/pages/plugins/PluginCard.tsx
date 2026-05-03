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

function stableConfigString(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(stableConfigString).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.entries(value as Record<string, unknown>)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, entryValue]) => `${JSON.stringify(key)}:${stableConfigString(entryValue)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

export function hasPluginConfigChanges(
  savedConfig: Record<string, unknown> | undefined,
  draftConfig: Record<string, unknown>,
): boolean {
  return stableConfigString(savedConfig ?? {}) !== stableConfigString(draftConfig);
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
  disabledReason?: string;
}

export function PluginCard({
  plugin,
  installed,
  configValues,
  onConfigChange,
  onToggle,
  onSaveConfig,
  isSaving,
  disabledReason,
}: PluginCardProps) {
  const meta = getPluginMeta(plugin.id, plugin.name);
  const isActive = !!installed;
  const [showAdvanced, setShowAdvanced] = useState(false);

  const basicFields = meta.configFields.filter((f) => !f.advanced);
  const advancedFields = meta.configFields.filter((f) => f.advanced);
  const hasConfig = meta.configFields.length > 0;
  const configChanged = isActive && hasConfig && hasPluginConfigChanges(installed?.config, configValues);

  return (
    <div
      className={`rounded-lg border p-5 transition-colors ${
        isActive
          ? "border-emerald-200 bg-emerald-50"
          : "border-slate-200 bg-white hover:border-slate-300 hover:bg-slate-50"
      }`}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-3 sm:gap-5">
        <div className="flex min-w-0 flex-1 items-start gap-3.5">
          <div
            className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ring-1 ${
              isActive ? "bg-emerald-100 ring-emerald-200" : "bg-slate-50 ring-slate-200"
            }`}
          >
            <PluginIcon
              icon={meta.icon}
              className={`w-5 h-5 ${isActive ? "text-emerald-700" : "text-slate-500"}`}
            />
          </div>
          <div className="min-w-0">
            <h3 className="break-words font-semibold text-slate-950">{meta.featureName}</h3>
            <p className="mt-0.5 break-words text-sm text-slate-600">{meta.summary}</p>
          </div>
        </div>

        <div className="shrink-0">
          <ToggleSwitch
            checked={isActive}
            onChange={onToggle}
            label={`Toggle ${meta.featureName}`}
            disabled={Boolean(disabledReason)}
          />
        </div>
      </div>

      {/* Install note */}
      {disabledReason && (
        <div className="mt-3 rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs leading-5 text-amber-800">
          {disabledReason}
        </div>
      )}

      {!isActive && (
        <div className="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs leading-5 text-slate-600">
          <span className="text-slate-700 font-medium">When enabled:</span> {meta.installNote}
        </div>
      )}

      {/* Active: show config & status */}
      {isActive && (
        <div className="mt-4 space-y-4">
          {/* Enable/Disable within installed */}
          {installed && !installed.enabled && (
            <div className="rounded-lg bg-amber-50 border border-amber-200 px-3 py-2 text-xs text-amber-800">
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
                    className="text-xs font-medium text-slate-600 hover:text-slate-950 transition-colors"
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
                    disabled={isSaving || Boolean(disabledReason)}
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
