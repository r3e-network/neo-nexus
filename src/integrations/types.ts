import type { SystemMetrics, NodeMetrics, LogEntry } from '../types/index';

// --- Provider category interfaces ---

export interface MetricsProvider {
  readonly name: string;
  pushMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface LogProvider {
  readonly name: string;
  pushLogs(entries: LogEntryWithContext[]): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface UptimeProvider {
  readonly name: string;
  registerMonitor(url: string, name: string): Promise<string>;
  removeMonitor(monitorId: string): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface NotificationProvider {
  readonly name: string;
  notify(event: IntegrationEvent): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface ErrorProvider {
  readonly name: string;
  captureError(error: Error, context?: Record<string, unknown>): void;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

// --- Shared data types ---

export interface NodeMetricsWithContext {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  network: string;
  metrics: NodeMetrics;
}

export interface LogEntryWithContext extends LogEntry {
  nodeId: string;
  nodeName: string;
}

export type IntegrationEventType =
  | 'node.started' | 'node.stopped' | 'node.crashed' | 'node.error'
  | 'node.synced' | 'node.behind'
  | 'watchdog.restart' | 'watchdog.exhausted'
  | 'disk.warning' | 'disk.critical'
  | 'security.login_failed' | 'security.default_password';

export type IntegrationEventSeverity = 'info' | 'warning' | 'critical';

export interface IntegrationEvent {
  type: IntegrationEventType;
  severity: IntegrationEventSeverity;
  title: string;
  message: string;
  nodeId?: string;
  nodeName?: string;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

// --- Config schema (drives frontend form rendering) ---

export type ConfigFieldType = 'text' | 'password' | 'url';

export interface ConfigField {
  key: string;
  label: string;
  type: ConfigFieldType;
  placeholder: string;
  required: boolean;
  sensitive?: boolean;
}

export type IntegrationCategory = 'metrics' | 'logging' | 'uptime' | 'alerting' | 'errors';

export type IntegrationId =
  | 'grafana_metrics' | 'datadog'
  | 'betterstack_logging' | 'grafana_loki'
  | 'betterstack_uptime' | 'uptimerobot'
  | 'webhook' | 'slack' | 'discord' | 'telegram'
  | 'sentry';

export interface IntegrationDefinition {
  id: IntegrationId;
  name: string;
  description: string;
  category: IntegrationCategory;
  configSchema: ConfigField[];
  createProvider(config: Record<string, string>): MetricsProvider | LogProvider | UptimeProvider | NotificationProvider | ErrorProvider;
}

// --- Database row ---

export interface IntegrationRow {
  id: string;
  category: string;
  enabled: number;
  config: string;
  last_test_at: string | null;
  last_error: string | null;
  created_at: string;
  updated_at: string;
}

// --- API response types ---

export interface IntegrationStatus {
  id: IntegrationId;
  name: string;
  description: string;
  category: IntegrationCategory;
  enabled: boolean;
  configured: boolean;
  configSchema: ConfigField[];
  configValues: Record<string, string>;
  lastTestAt: string | null;
  lastError: string | null;
}
