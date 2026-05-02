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
    <div className="text-center py-10 px-6 border border-dashed border-slate-300 bg-slate-50 rounded-lg animate-fade-in">
      <div className="w-12 h-12 bg-white rounded-lg flex items-center justify-center mx-auto mb-4 border border-slate-200 shadow-sm">
        <Icon className="w-6 h-6 text-slate-500" />
      </div>
      <h3 className="text-slate-950 font-medium mb-1">{title}</h3>
      {description && <p className="text-sm text-slate-600 mb-6 max-w-md mx-auto">{description}</p>}
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
