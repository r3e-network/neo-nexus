export function calculateDaysUntilFull(freeBytes: number, growthBytesPerHour: number): number {
  if (growthBytesPerHour <= 0) return Infinity;
  return freeBytes / growthBytesPerHour / 24;
}

export function getDiskAlertLevel(usagePercent: number): 'critical' | 'warning' | null {
  if (usagePercent >= 95) return 'critical';
  if (usagePercent >= 90) return 'warning';
  return null;
}

const diskReadings: Array<{ timestamp: number; freeBytes: number }> = [];

export function recordDiskReading(freeBytes: number): void {
  diskReadings.push({ timestamp: Date.now(), freeBytes });
  if (diskReadings.length > 20) diskReadings.shift();
}

export function getGrowthRatePerHour(): number {
  if (diskReadings.length < 2) return 0;
  const oldest = diskReadings[0];
  const newest = diskReadings[diskReadings.length - 1];
  const hours = (newest.timestamp - oldest.timestamp) / (1000 * 60 * 60);
  if (hours <= 0) return 0;
  return (oldest.freeBytes - newest.freeBytes) / hours;
}
