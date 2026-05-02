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
  const borderColor = isError ? "border-red-200" : "border-emerald-200";
  const bgColor = isError ? "bg-red-50" : "bg-emerald-50";
  const textColor = isError ? "text-red-800" : "text-emerald-800";
  const subtextColor = isError ? "text-red-700" : "text-emerald-700";
  const hoverTextColor = isError ? "hover:text-red-900" : "hover:text-emerald-900";

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
                    className="text-xs font-medium text-blue-700 hover:text-blue-900 underline"
                  >
                    {action.label}
                  </a>
                ) : (
                  <button
                    key={action.label}
                    type="button"
                    onClick={action.onClick}
                    className="text-xs font-medium text-blue-700 hover:text-blue-900 underline"
                  >
                    {action.label}
                  </button>
                ),
              )}

              {suggestion && (
                <button
                  type="button"
                  onClick={() => setCollapsed(!collapsed)}
                  className={`text-xs ${subtextColor} ${hoverTextColor} flex items-center gap-1 ml-auto`}
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
