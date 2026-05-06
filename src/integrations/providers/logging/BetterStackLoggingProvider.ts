// src/integrations/providers/logging/BetterStackLoggingProvider.ts
import type { LogProvider, LogEntryWithContext, ConfigField } from '../../types';
import { safeIntegrationFetch } from '../../safeFetch';

export const betterStackLoggingSchema: ConfigField[] = [
  { key: 'sourceToken', label: 'Source Token', type: 'password', placeholder: 'your-logtail-source-token', required: true },
  { key: 'ingestingUrl', label: 'Ingesting URL', type: 'url', placeholder: 'https://in.logs.betterstack.com', required: false },
];

const DEFAULT_BETTER_STACK_LOGS_URL = 'https://in.logs.betterstack.com';

export class BetterStackLoggingProvider implements LogProvider {
  readonly name = 'Better Stack';

  constructor(private config: { sourceToken: string; ingestingUrl?: string }) {}

  async pushLogs(entries: LogEntryWithContext[]): Promise<void> {
    if (entries.length === 0) return;

    const payload = entries.map(entry => ({
      dt: new Date(entry.timestamp).toISOString(),
      level: entry.level,
      message: entry.message,
      source: entry.source,
      node_id: entry.nodeId,
      node_name: entry.nodeName,
    }));

    const response = await safeIntegrationFetch(this.ingestingUrl(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.config.sourceToken}`,
      },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`Better Stack push failed: ${response.status} ${response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const response = await safeIntegrationFetch(this.ingestingUrl(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.config.sourceToken}`,
      },
      body: JSON.stringify([{ dt: new Date().toISOString(), message: 'NeoNexus connection test', level: 'info' }]),
      signal: AbortSignal.timeout(10_000),
    });
    return response.ok || response.status === 202;
  }

  private ingestingUrl(): string {
    return this.config.ingestingUrl?.trim() || DEFAULT_BETTER_STACK_LOGS_URL;
  }
}
