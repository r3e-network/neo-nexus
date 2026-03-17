export function clampReconcileRunsLimit(value: string | undefined, fallback = 10) {
  const parsed = Number.parseInt(value ?? '', 10);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    return fallback;
  }

  return Math.min(parsed, 50);
}
