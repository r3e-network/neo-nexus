export interface RealtimeLogEntry {
  timestamp: number;
  level: string;
  message: string;
}

export function mergeNodeLogs(
  baseLogs: RealtimeLogEntry[],
  realtimeLogs: RealtimeLogEntry[],
): RealtimeLogEntry[] {
  const deduped = new Map<string, RealtimeLogEntry>();

  for (const entry of [...baseLogs, ...realtimeLogs]) {
    deduped.set(`${entry.timestamp}:${entry.level}:${entry.message}`, entry);
  }

  return [...deduped.values()].sort((left, right) => left.timestamp - right.timestamp);
}
