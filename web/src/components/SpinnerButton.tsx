import React from "react";
import { Loader2 } from "lucide-react";

interface SpinnerButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  loading?: boolean;
  children: React.ReactNode;
}

export function SpinnerButton({ loading, children, disabled, className, ...props }: SpinnerButtonProps) {
  return (
    <button
      {...props}
      disabled={disabled || loading}
      className={className}
    >
      {loading && <Loader2 className="w-4 h-4 animate-spin" />}
      {children}
    </button>
  );
}
