export function normalizeBatchIds(ids: number[]) {
  const normalized = Array.from(new Set(
    ids.filter((id) => Number.isInteger(id) && id > 0),
  ));
  normalized.sort((left, right) => left - right);
  return normalized;
}
