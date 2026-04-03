import { Link } from "react-router-dom";

interface EmptyStateAction {
  label: string;
  href?: string;
  onClick?: () => void;
  variant?: "primary" | "secondary";
}

interface EmptyStateProps {
  icon: React.ElementType;
  title: string;
  description?: string;
  action?: EmptyStateAction;
  actions?: EmptyStateAction[];
}

export function EmptyState({ icon: Icon, title, description, action, actions }: EmptyStateProps) {
  const allActions = actions ?? (action ? [action] : []);

  return (
    <div className="text-center py-12 px-6 border-2 border-dashed border-slate-700/50 bg-slate-800/10 backdrop-blur-sm rounded-2xl animate-fade-in transition-all duration-300 hover:border-slate-600/50">
      <div className="w-16 h-16 bg-slate-800/50 rounded-full flex items-center justify-center mx-auto mb-4 border border-slate-700/50 shadow-inner">
        <Icon className="w-8 h-8 text-slate-400" />
      </div>
      <h3 className="text-white font-medium mb-1">{title}</h3>
      {description && <p className="text-sm text-slate-400 mb-6 max-w-md mx-auto">{description}</p>}
      {allActions.length > 0 && (
        <div className="flex items-center justify-center gap-3">
          {allActions.map((a) => {
            const cls = a.variant === "secondary"
              ? "btn btn-secondary inline-flex"
              : "btn btn-primary inline-flex";
            return a.href ? (
              <Link key={a.label} to={a.href} className={cls}>{a.label}</Link>
            ) : (
              <button key={a.label} type="button" className={cls} onClick={a.onClick}>{a.label}</button>
            );
          })}
        </div>
      )}
    </div>
  );
}
