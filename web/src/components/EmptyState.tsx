import React from "react";
import { Link } from "react-router-dom";

interface EmptyStateProps {
  icon: React.ElementType;
  title: string;
  description?: string;
  action?: { label: string; href?: string; onClick?: () => void };
}

export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="py-12 text-center">
      <Icon className="w-12 h-12 text-slate-600 mx-auto mb-4" />
      <p className="text-white font-medium">{title}</p>
      {description && <p className="text-slate-400 text-sm mt-1">{description}</p>}
      {action && (
        <div className="mt-4">
          {action.href ? (
            <Link to={action.href} className="btn btn-primary inline-flex">
              {action.label}
            </Link>
          ) : (
            <button type="button" className="btn btn-primary" onClick={action.onClick}>
              {action.label}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
