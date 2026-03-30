import { AlertCircle, CheckCircle, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";

interface BannerAction {
  label: string;
  href?: string;
  onClick?: () => void;
}

interface FeedbackBannerProps {
  error?: string;
  suggestion?: string;
  code?: string;
  success?: string;
  actions?: BannerAction[];
}

export function FeedbackBanner({ error, suggestion, code, success, actions }: FeedbackBannerProps) {
  const [collapsed, setCollapsed] = useState(false);

  if (!error && !success) return null;

  const isError = !!error;
  const Icon = isError ? AlertCircle : CheckCircle;
  const borderColor = isError ? "border-red-500/20" : "border-emerald-500/20";
  const bgColor = isError ? "bg-red-500/10" : "bg-emerald-500/10";
  const textColor = isError ? "text-red-300" : "text-emerald-300";
  const subtextColor = isError ? "text-red-400/70" : "text-emerald-400/70";

  return (
    <div className={`${borderColor} ${bgColor} border rounded-lg p-4 mb-4 animate-fade-in`}>
      <div className="flex items-start gap-3">
        <Icon className={`w-5 h-5 ${textColor} shrink-0 mt-0.5`} />
        <div className="flex-1 min-w-0">
          <p className={`text-sm ${textColor}`}>{error || success}</p>

          {suggestion && !collapsed && (
            <p className={`text-sm ${subtextColor} mt-1`}>{suggestion}</p>
          )}

          {(suggestion || (actions && actions.length > 0)) && (
            <div className="flex items-center gap-3 mt-2 flex-wrap">
              {actions?.map((action) =>
                action.href ? (
                  <a
                    key={action.label}
                    href={action.href}
                    className="text-xs font-medium text-blue-400 hover:text-blue-300 underline"
                  >
                    {action.label}
                  </a>
                ) : (
                  <button
                    key={action.label}
                    type="button"
                    onClick={action.onClick}
                    className="text-xs font-medium text-blue-400 hover:text-blue-300 underline"
                  >
                    {action.label}
                  </button>
                ),
              )}

              {suggestion && (
                <button
                  type="button"
                  onClick={() => setCollapsed(!collapsed)}
                  className={`text-xs ${subtextColor} hover:${textColor} flex items-center gap-1 ml-auto`}
                >
                  {collapsed ? (
                    <>Show hint <ChevronDown className="w-3 h-3" /></>
                  ) : (
                    <>Hide hint <ChevronUp className="w-3 h-3" /></>
                  )}
                </button>
              )}
            </div>
          )}
        </div>
      </div>
      {code && (
        <p className="text-[10px] text-slate-600 mt-2 ml-8 font-mono select-all">{code}</p>
      )}
    </div>
  );
}
