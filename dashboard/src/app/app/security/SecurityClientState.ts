import type { ApiKeyType } from './SecurityClient';

export function mergeCreatedApiKey(
  current: ApiKeyType[],
  created: ApiKeyType,
) {
  const merged = [...current, created];
  merged.sort((left, right) => left.createdAt.getTime() - right.createdAt.getTime());
  return merged;
}
