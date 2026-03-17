export function clampOperationsItemsLimit(
  value: string | undefined,
  fallback = 25,
) {
  const parsed = Number.parseInt(value ?? '', 10);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    return fallback;
  }

  return Math.min(Math.max(parsed, 10), 200);
}
