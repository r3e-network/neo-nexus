import { CheckCircle2, ShieldCheck } from "lucide-react";
import {
  evaluatePasswordStrength,
  passwordStrengthBar,
  passwordStrengthTone,
} from "../utils/passwordStrength";

interface PasswordStrengthMeterProps {
  password: string;
  compact?: boolean;
}

export function PasswordStrengthMeter({ password, compact = false }: PasswordStrengthMeterProps) {
  const strength = evaluatePasswordStrength(password);
  const isEmpty = password.length === 0;
  const displayLabel = isEmpty ? "Not started" : strength.label;
  const progress = password.length === 0 ? 0 : Math.max(12, (strength.score + 1) * 20);
  const tone = isEmpty ? "text-slate-500" : passwordStrengthTone(strength.label);
  const barClass = isEmpty ? "bg-slate-400" : passwordStrengthBar(strength.label);

  return (
    <div className={`rounded-lg border border-slate-200 bg-slate-50 ${compact ? "px-3 py-2" : "p-3"}`} aria-live="polite">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-xs font-medium text-slate-600">
          <ShieldCheck className="h-3.5 w-3.5" />
          <span>Password strength</span>
        </div>
        <span className={`text-xs font-semibold ${tone}`}>{displayLabel}</span>
      </div>

      <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-slate-200">
        <div
          className={`h-full rounded-full transition-all duration-300 ${barClass}`}
          style={{ width: `${progress}%` }}
        />
      </div>

      {strength.feedback.length > 0 ? (
        <ul className="mt-2 space-y-1 text-xs leading-5 text-slate-600">
          {strength.feedback.slice(0, compact ? 2 : 4).map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : (
        <p className="mt-2 flex items-center gap-1 text-xs font-medium text-emerald-700">
          <CheckCircle2 className="h-3.5 w-3.5" />
          Ready for admin access.
        </p>
      )}
    </div>
  );
}
