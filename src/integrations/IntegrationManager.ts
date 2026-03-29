import type Database from 'better-sqlite3';
import type { SystemMetrics } from '../types/index';
import type {
  IntegrationId,
  IntegrationRow,
  IntegrationStatus,
  IntegrationEvent,
  MetricsProvider,
  LogProvider,
  NotificationProvider,
  ErrorProvider,
  NodeMetricsWithContext,
  LogEntryWithContext,
} from './types';
import { integrationRegistry, registryMap } from './registry';

const SENSITIVE_FIELD_TYPES = new Set(['password']);

export class IntegrationManager {
  private metricsProviders: MetricsProvider[] = [];
  private logProviders: LogProvider[] = [];
  private notificationProviders: NotificationProvider[] = [];
  private errorProviders: ErrorProvider[] = [];

  constructor(private db: Database.Database) {
    this.loadEnabledProviders();
  }

  // --- Provider lifecycle ---

  private loadEnabledProviders(): void {
    const rows = this.db.prepare(
      'SELECT * FROM integrations WHERE enabled = 1'
    ).all() as IntegrationRow[];

    for (const row of rows) {
      this.instantiateProvider(row);
    }
  }

  private instantiateProvider(row: IntegrationRow): void {
    const definition = registryMap.get(row.id as IntegrationId);
    if (!definition) return;

    const config = JSON.parse(row.config) as Record<string, string>;

    // Check that all required fields are present
    const missingFields = definition.configSchema
      .filter(f => f.required && !config[f.key])
      .map(f => f.key);
    if (missingFields.length > 0) return;

    try {
      const provider = definition.createProvider(config);

      switch (definition.category) {
        case 'metrics':
          this.metricsProviders.push(provider as MetricsProvider);
          break;
        case 'logging':
          this.logProviders.push(provider as LogProvider);
          break;
        case 'alerting':
          this.notificationProviders.push(provider as NotificationProvider);
          break;
        case 'errors':
          this.errorProviders.push(provider as ErrorProvider);
          break;
        // uptime providers act on enable/disable, don't need to be stored
      }
    } catch (error) {
      this.updateLastError(row.id, error instanceof Error ? error.message : 'Failed to initialize');
    }
  }

  reloadProvider(id: string): void {
    const definition = registryMap.get(id as IntegrationId);
    if (!definition) return;

    this.removeProviderInstances(id, definition.category);

    const row = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;
    if (row && row.enabled) {
      this.instantiateProvider(row);
    }
  }

  private removeProviderInstances(id: string, category: string): void {
    const definition = registryMap.get(id as IntegrationId);
    if (!definition) return;
    const name = definition.name;

    const shutdownAndFilter = <T extends { readonly name: string; shutdown?(): void }>(list: T[]): T[] => {
      const removed = list.filter(p => p.name === name);
      for (const p of removed) p.shutdown?.();
      return list.filter(p => p.name !== name);
    };

    switch (category) {
      case 'metrics':
        this.metricsProviders = shutdownAndFilter(this.metricsProviders);
        break;
      case 'logging':
        this.logProviders = shutdownAndFilter(this.logProviders);
        break;
      case 'alerting':
        this.notificationProviders = shutdownAndFilter(this.notificationProviders);
        break;
      case 'errors':
        this.errorProviders = shutdownAndFilter(this.errorProviders);
        break;
    }
  }

  shutdown(): void {
    for (const p of [...this.metricsProviders, ...this.logProviders, ...this.notificationProviders, ...this.errorProviders]) {
      (p as { shutdown?(): void }).shutdown?.();
    }
    this.metricsProviders = [];
    this.logProviders = [];
    this.notificationProviders = [];
    this.errorProviders = [];
  }

  // --- Broadcasting ---

  async broadcastMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void> {
    await Promise.allSettled(
      this.metricsProviders.map(p =>
        p.pushMetrics(system, nodes).catch(err =>
          this.handleProviderError(p.name, err)
        )
      )
    );
  }

  async broadcastLogs(entries: LogEntryWithContext[]): Promise<void> {
    if (entries.length === 0) return;
    await Promise.allSettled(
      this.logProviders.map(p =>
        p.pushLogs(entries).catch(err =>
          this.handleProviderError(p.name, err)
        )
      )
    );
  }

  async broadcastNotification(event: IntegrationEvent): Promise<void> {
    await Promise.allSettled(
      this.notificationProviders.map(p =>
        p.notify(event).catch(err =>
          this.handleProviderError(p.name, err)
        )
      )
    );
  }

  captureError(error: Error, context?: Record<string, unknown>): void {
    for (const p of this.errorProviders) {
      try {
        p.captureError(error, context);
      } catch {
        // Silently fail
      }
    }
  }

  // --- CRUD ---

  listAll(): IntegrationStatus[] {
    return integrationRegistry.map(def => {
      const row = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(def.id) as IntegrationRow | undefined;

      const rawConfig = row ? JSON.parse(row.config) as Record<string, string> : {};
      const configValues = this.redactSensitiveFields(def.id, rawConfig);

      return {
        id: def.id,
        name: def.name,
        description: def.description,
        category: def.category,
        enabled: row?.enabled === 1,
        configured: this.isConfigured(def.id, rawConfig),
        configSchema: def.configSchema,
        configValues,
        lastTestAt: row?.last_test_at ?? null,
        lastError: row?.last_error ?? null,
      };
    });
  }

  getOne(id: string): IntegrationStatus | null {
    const def = registryMap.get(id as IntegrationId);
    if (!def) return null;

    const row = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;
    const rawConfig = row ? JSON.parse(row.config) as Record<string, string> : {};

    return {
      id: def.id,
      name: def.name,
      description: def.description,
      category: def.category,
      enabled: row?.enabled === 1,
      configured: this.isConfigured(def.id, rawConfig),
      configSchema: def.configSchema,
      configValues: this.redactSensitiveFields(def.id, rawConfig),
      lastTestAt: row?.last_test_at ?? null,
      lastError: row?.last_error ?? null,
    };
  }

  saveConfig(id: string, config: Record<string, string>, enabled: boolean): void {
    const def = registryMap.get(id as IntegrationId);
    if (!def) throw new Error(`Unknown integration: ${id}`);

    const existing = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;

    // Merge with existing config to preserve fields not sent (redacted ones)
    let mergedConfig = config;
    if (existing) {
      const existingConfig = JSON.parse(existing.config) as Record<string, string>;
      mergedConfig = { ...existingConfig };
      for (const [key, value] of Object.entries(config)) {
        if (value && !value.match(/^\*+\.{3}\w{4}$/)) {
          mergedConfig[key] = value;
        }
      }
    }

    if (existing) {
      this.db.prepare(
        "UPDATE integrations SET config = ?, enabled = ?, last_error = NULL, updated_at = datetime('now') WHERE id = ?"
      ).run(JSON.stringify(mergedConfig), enabled ? 1 : 0, id);
    } else {
      this.db.prepare(
        'INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)'
      ).run(id, def.category, JSON.stringify(mergedConfig), enabled ? 1 : 0);
    }

    this.reloadProvider(id);
  }

  deleteConfig(id: string): void {
    const def = registryMap.get(id as IntegrationId);
    if (!def) throw new Error(`Unknown integration: ${id}`);

    this.removeProviderInstances(id, def.category);
    this.db.prepare('DELETE FROM integrations WHERE id = ?').run(id);
  }

  async testProvider(id: string): Promise<{ success: boolean; error?: string }> {
    const def = registryMap.get(id as IntegrationId);
    if (!def) return { success: false, error: 'Unknown integration' };

    const row = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;
    if (!row) return { success: false, error: 'Not configured' };

    const config = JSON.parse(row.config) as Record<string, string>;

    try {
      const provider = def.createProvider(config);
      const ok = await provider.testConnection();
      if (ok) {
        this.db.prepare(
          "UPDATE integrations SET last_test_at = datetime('now'), last_error = NULL WHERE id = ?"
        ).run(id);
        return { success: true };
      }
      const errMsg = 'Connection test returned false';
      this.updateLastError(id, errMsg);
      return { success: false, error: errMsg };
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : 'Test failed';
      this.updateLastError(id, errMsg);
      return { success: false, error: errMsg };
    }
  }

  // --- Helpers ---

  private isConfigured(id: IntegrationId, config: Record<string, string>): boolean {
    const def = registryMap.get(id);
    if (!def) return false;
    return def.configSchema
      .filter(f => f.required)
      .every(f => !!config[f.key]);
  }

  private redactSensitiveFields(id: IntegrationId, config: Record<string, string>): Record<string, string> {
    const def = registryMap.get(id);
    if (!def) return config;

    const redacted: Record<string, string> = {};
    for (const field of def.configSchema) {
      const value = config[field.key];
      if (!value) {
        redacted[field.key] = '';
      } else if (SENSITIVE_FIELD_TYPES.has(field.type)) {
        redacted[field.key] = value.length > 4
          ? '*'.repeat(Math.min(value.length - 4, 8)) + '...' + value.slice(-4)
          : '****';
      } else {
        redacted[field.key] = value;
      }
    }
    return redacted;
  }

  private updateLastError(id: string, error: string): void {
    this.db.prepare(
      "UPDATE integrations SET last_error = ?, updated_at = datetime('now') WHERE id = ?"
    ).run(error, id);
  }

  private handleProviderError(providerName: string, error: unknown): void {
    const msg = error instanceof Error ? error.message : String(error);
    console.error(`[integrations] ${providerName} error: ${msg}`);

    for (const def of integrationRegistry) {
      if (def.name === providerName) {
        this.updateLastError(def.id, msg);
        break;
      }
    }
  }
}
