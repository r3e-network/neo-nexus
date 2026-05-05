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
  Trash2,
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
  onInstall: () => void;
  onRemove: () => void;
  onSetEnabled: (enabled: boolean) => void;
  onSaveConfig: () => void;
  isInstalling: boolean;
  isRemoving: boolean;
  isSettingEnabled: boolean;
  isSaving: boolean;
  disabledReason?: string;
}

export function PluginCard({
  plugin,
  installed,
  configValues,
  onConfigChange,
  onInstall,
  onRemove,
  onSetEnabled,
  onSaveConfig,
  isInstalling,
  isRemoving,
  isSettingEnabled,
  isSaving,
  disabledReason,
}: PluginCardProps) {
  const meta = getPluginMeta(plugin.id, plugin.name);
  const isInstalled = !!installed;
  const isEnabled = installed?.enabled ?? false;
  const [showAdvanced, setShowAdvanced] = useState(false);

  const basicFields = meta.configFields.filter((f) => !f.advanced);
  const advancedFields = meta.configFields.filter((f) => f.advanced);
  const hasConfig = meta.configFields.length > 0;
  const configChanged = isInstalled && hasConfig && hasPluginConfigChanges(installed?.config, configValues);

  return (
    <div
      className={`rounded-lg border p-5 transition-colors ${
        isInstalled
          ? "border-emerald-200 bg-emerald-50"
          : "border-slate-200 bg-white hover:border-slate-300 hover:bg-slate-50"
      }`}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-3 sm:gap-5">
        <div className="flex min-w-0 flex-1 items-start gap-3.5">
          <div
            className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ring-1 ${
              isInstalled ? "bg-emerald-100 ring-emerald-200" : "bg-slate-50 ring-slate-200"
            }`}
          >
            <PluginIcon
              icon={meta.icon}
              className={`w-5 h-5 ${isInstalled ? "text-emerald-700" : "text-slate-500"}`}
            />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className="break-words font-semibold text-slate-950">{meta.featureName}</h3>
              {installed?.version && (
                <span className="rounded border border-slate-200 bg-white px-1.5 py-0.5 text-[11px] font-mono text-slate-600">
                  {installed.version}
                </span>
              )}
              {isInstalled && (
                <span className={`rounded px-1.5 py-0.5 text-[11px] font-medium ${isEnabled ? "bg-emerald-100 text-emerald-800" : "bg-amber-100 text-amber-800"}`}>
                  {isEnabled ? "Enabled" : "Disabled"}
                </span>
              )}
            </div>
            <p className="mt-0.5 break-words text-sm text-slate-600">{meta.summary}</p>
          </div>
        </div>

        <div className="shrink-0">
          {isInstalled ? (
            <div className="flex items-center gap-2">
              <ToggleSwitch
                checked={isEnabled}
                onChange={onSetEnabled}
                label={`${isEnabled ? "Disable" : "Enable"} ${meta.featureName}`}
                disabled={Boolean(disabledReason) || isSettingEnabled}
              />
              <button
                type="button"
                onClick={onRemove}
                disabled={Boolean(disabledReason) || isRemoving}
                className="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-500 transition hover:bg-red-50 hover:text-red-700 disabled:cursor-not-allowed disabled:opacity-50"
                aria-label={`Remove ${meta.featureName}`}
                title={`Remove ${meta.featureName}`}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ) : (
            <button
              type="button"
              onClick={onInstall}
              disabled={Boolean(disabledReason) || isInstalling}
              className="inline-flex items-center gap-1.5 rounded-lg bg-emerald-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-slate-300"
            >
              <Wrench className="h-4 w-4" />
              {isInstalling ? "Installing..." : "Install"}
            </button>
          )}
        </div>
      </div>

      {/* Install note */}
      {disabledReason && (
        <div className="mt-3 rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs leading-5 text-amber-800">
          {disabledReason}
        </div>
      )}

      {!isInstalled && (
        <div className="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs leading-5 text-slate-600">
          <span className="text-slate-700 font-medium">When enabled:</span> {meta.installNote}
        </div>
      )}

      {/* Active: show config & status */}
      {isInstalled && (
        <div className="mt-4 space-y-4">
          {/* Enable/Disable within installed */}
          {installed && !installed.enabled && (
            <div className="rounded-lg bg-amber-50 border border-amber-200 px-3 py-2 text-xs text-amber-800">
              Installed but disabled. Enable it to include this plugin in the node config on the next start.
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
