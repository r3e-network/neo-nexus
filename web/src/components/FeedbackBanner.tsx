import { AlertCircle, CheckCircle } from "lucide-react";

interface FeedbackBannerProps {
  error?: string;
  success?: string;
}

export function FeedbackBanner({ error, success }: FeedbackBannerProps) {
  if (!error && !success) return null;

  const isError = !!error;
  const message = error || success;

  return (
    <div
      className={`flex items-center gap-2 rounded-lg border px-4 py-3 text-sm ${
        isError
          ? "border-red-500/20 bg-red-500/10 text-red-300"
          : "border-emerald-500/20 bg-emerald-500/10 text-emerald-300"
      }`}
    >
      {isError ? (
        <AlertCircle className="h-4 w-4 shrink-0" />
      ) : (
        <CheckCircle className="h-4 w-4 shrink-0" />
      )}
      <span>{message}</span>
    </div>
  );
}
