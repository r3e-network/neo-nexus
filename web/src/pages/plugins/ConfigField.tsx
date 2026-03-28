import { useState } from "react";
import { HelpCircle } from "lucide-react";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import { type PluginConfigField } from "../../utils/pluginMeta";

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

// ── Config field renderer ──────────────────────────────────────────────
export interface ConfigFieldProps {
  field: PluginConfigField;
  value: unknown;
  onChange: (key: string, value: unknown) => void;
}

export function ConfigField({ field, value, onChange }: ConfigFieldProps) {
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
