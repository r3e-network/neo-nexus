import type { NodeType } from '../types/index';

export interface ResourceLimits {
  maxMemoryMB?: number;
}

export function buildResourceEnv(nodeType: NodeType, limits: ResourceLimits): Record<string, string> {
  const env: Record<string, string> = {};
  if (!limits.maxMemoryMB) return env;
  if (nodeType === 'neo-go') {
    env.GOMEMLIMIT = `${limits.maxMemoryMB}MiB`;
  } else {
    env.DOTNET_GCHeapHardLimit = (limits.maxMemoryMB * 1024 * 1024).toString(16);
  }
  return env;
}
