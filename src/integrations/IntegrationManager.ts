import type Database from 'better-sqlite3';
import type { SystemMetrics } from '../types/index';
import type {
  IntegrationId,
  IntegrationRow,
  IntegrationStatus,
  IntegrationEvent,
  MetricsProvider,
  LogProvider,
  UptimeProvider,
  NotificationProvider,
  ErrorProvider,
  NodeMetricsWithContext,
  LogEntryWithContext,
} from './types';
import { integrationRegistry, registryMap } from './registry';

const REDACTION_PREFIX = '••••••••...';

export class IntegrationManager {
  private metricsProviders = new Map<string, MetricsProvider>();
  private logProviders = new Map<string, LogProvider>();
  private notificationProviders = new Map<string, NotificationProvider>();
  private errorProviders = new Map<string, ErrorProvider>();
  private lastAlertTimes = new Map<string, number>();

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
      const id = row.id;

      switch (definition.category) {
        case 'metrics':
          this.metricsProviders.set(id, provider as MetricsProvider);
          break;
        case 'logging':
          this.logProviders.set(id, provider as LogProvider);
          break;
        case 'alerting':
          this.notificationProviders.set(id, provider as NotificationProvider);
          break;
        case 'errors':
          this.errorProviders.set(id, provider as ErrorProvider);
          break;
        case 'uptime':
          // Uptime providers auto-register a monitor for the NeoNexus health endpoint
          this.registerUptimeMonitor(id, provider as UptimeProvider);
          break;
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
    const shutdownAndRemove = <T extends { shutdown?(): void }>(map: Map<string, T>): void => {
      const existing = map.get(id);
      if (existing) {
        existing.shutdown?.();
        map.delete(id);
      }
    };

    switch (category) {
      case 'metrics': shutdownAndRemove(this.metricsProviders); break;
      case 'logging': shutdownAndRemove(this.logProviders); break;
      case 'alerting': shutdownAndRemove(this.notificationProviders); break;
      case 'errors': shutdownAndRemove(this.errorProviders); break;
      case 'uptime': this.deregisterUptimeMonitor(id); break;
    }
  }

  private registerUptimeMonitor(id: string, provider: UptimeProvider): void {
    const port = process.env.PORT || '8080';
    const host = process.env.HOST || '0.0.0.0';
    const healthUrl = `http://${host === '0.0.0.0' ? 'localhost' : host}:${port}/api/health`;

    provider.registerMonitor(healthUrl, `NeoNexus (${id})`).then(monitorId => {
      // Store the monitor ID so we can remove it later
      this.db.prepare(
        "UPDATE integrations SET config = json_set(config, '$._monitorId', ?) WHERE id = ?"
      ).run(monitorId, id);
    }).catch(err => {
      this.handleProviderError(id, err);
    });
  }

  private deregisterUptimeMonitor(id: string): void {
    const row = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;
    if (!row) return;

    const config = JSON.parse(row.config) as Record<string, string>;
    const monitorId = config._monitorId;
    if (!monitorId) return;

    const def = registryMap.get(id as IntegrationId);
    if (!def) return;

    try {
      const provider = def.createProvider(config) as UptimeProvider;
      provider.removeMonitor(monitorId).catch(err => {
        this.handleProviderError(id, err);
      });
    } catch {
      // Provider creation failed, can't deregister
    }
  }

  shutdown(): void {
    const allMaps = [this.metricsProviders, this.logProviders, this.notificationProviders, this.errorProviders];
    for (const map of allMaps) {
      for (const p of map.values()) {
        (p as { shutdown?(): void }).shutdown?.();
      }
      map.clear();
    }
  }

  // --- Broadcasting ---

  async broadcastMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void> {
    const providers = [...this.metricsProviders.entries()];
    await Promise.allSettled(
      providers.map(([id, p]) =>
        p.pushMetrics(system, nodes).catch(err =>
          this.handleProviderError(id, err)
        )
      )
    );
  }

  async broadcastLogs(entries: LogEntryWithContext[]): Promise<void> {
    if (entries.length === 0) return;
    const providers = [...this.logProviders.entries()];
    await Promise.allSettled(
      providers.map(([id, p]) =>
        p.pushLogs(entries).catch(err =>
          this.handleProviderError(id, err)
        )
      )
    );
  }

  async broadcastNotification(event: IntegrationEvent): Promise<void> {
    // Debounce repeated alerts of the same type (5 minute cooldown)
    const ALERT_COOLDOWN_MS = 5 * 60 * 1000;
    const cooldownKey = event.type + (event.nodeId ?? '');
    const lastTime = this.lastAlertTimes.get(cooldownKey);
    if (lastTime && event.timestamp - lastTime < ALERT_COOLDOWN_MS) {
      return;
    }
    this.lastAlertTimes.set(cooldownKey, event.timestamp);

    const providers = [...this.notificationProviders.entries()];
    await Promise.allSettled(
      providers.map(([id, p]) =>
        p.notify(event).catch(err =>
          this.handleProviderError(id, err)
        )
      )
    );
  }

  captureError(error: Error, context?: Record<string, unknown>): void {
    for (const p of this.errorProviders.values()) {
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

    // Validate URL-type fields
    for (const field of def.configSchema) {
      const value = config[field.key];
      if (field.type === 'url' && value && !value.startsWith(REDACTION_PREFIX)) {
        try {
          new URL(value);
        } catch {
          throw new Error(`Invalid URL for ${field.label}: ${value}`);
        }
      }
    }

    const existing = this.db.prepare('SELECT * FROM integrations WHERE id = ?').get(id) as IntegrationRow | undefined;

    // Merge with existing config to preserve redacted fields the user didn't change
    let mergedConfig = config;
    if (existing) {
      const existingConfig = JSON.parse(existing.config) as Record<string, string>;
      mergedConfig = { ...existingConfig };
      for (const [key, value] of Object.entries(config)) {
        // Skip if the value looks like a redacted placeholder (starts with our redaction prefix)
        if (value && !value.startsWith(REDACTION_PREFIX)) {
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
      } else if (field.type === 'password' || field.sensitive) {
        redacted[field.key] = value.length > 4
          ? REDACTION_PREFIX + value.slice(-4)
          : REDACTION_PREFIX.slice(0, 4);
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

  private handleProviderError(integrationId: string, error: unknown): void {
    const msg = error instanceof Error ? error.message : String(error);
    const def = registryMap.get(integrationId as IntegrationId);
    console.error(`[integrations] ${def?.name ?? integrationId} error: ${msg}`);
    this.updateLastError(integrationId, msg);
  }
}
