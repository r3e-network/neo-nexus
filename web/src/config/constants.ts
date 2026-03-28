/** React Query refetch intervals (ms) */
export const REFETCH_INTERVALS = {
  dashboard: 5_000,
  nodeDetail: 5_000,
  plugins: 5_000,
  signerHealth: 10_000,
  servers: 15_000,
  publicDashboard: 5_000,
} as const;

/** UI limits */
export const UI_LIMITS = {
  maxNotificationToasts: 3,
  notificationDismissMs: 8_000,
  maxLogEntries: 50,
  maxUnreadBadge: 9,
} as const;
