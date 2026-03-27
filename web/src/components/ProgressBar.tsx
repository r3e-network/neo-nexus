interface ProgressBarProps {
  value: number;
  color?: string;
  className?: string;
}

export function ProgressBar({ value, color = "bg-blue-500", className = "" }: ProgressBarProps) {
  return (
    <div className={`h-2 bg-slate-700 rounded-full overflow-hidden ${className}`}>
      <div
        className={`progress-bar-fill ${color}`}
        style={{ width: `${Math.min(100, Math.max(0, value))}%` }}
      />
    </div>
  );
}
