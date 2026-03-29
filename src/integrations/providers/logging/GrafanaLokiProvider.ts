// src/integrations/providers/logging/GrafanaLokiProvider.ts
import type { LogProvider, LogEntryWithContext, ConfigField } from '../../types';

export const grafanaLokiSchema: ConfigField[] = [
  { key: 'pushUrl', label: 'Loki Push URL', type: 'url', placeholder: 'https://logs-prod-006.grafana.net', required: true },
  { key: 'username', label: 'Username / Instance ID', type: 'text', placeholder: '123456', required: true },
  { key: 'apiKey', label: 'API Key', type: 'password', placeholder: 'glc_...', required: true },
];

export class GrafanaLokiProvider implements LogProvider {
  readonly name = 'Grafana Loki';

  constructor(private config: { pushUrl: string; username: string; apiKey: string }) {}

  async pushLogs(entries: LogEntryWithContext[]): Promise<void> {
    if (entries.length === 0) return;

    // Group entries by node + level to create Loki streams
    const streams = new Map<string, { stream: Record<string, string>; values: [string, string][] }>();

    for (const entry of entries) {
      const key = `${entry.nodeId}:${entry.level}`;
      if (!streams.has(key)) {
        streams.set(key, {
          stream: {
            job: 'neonexus',
            node_id: entry.nodeId,
            node_name: entry.nodeName,
            level: entry.level,
            source: entry.source || 'node',
          },
          values: [],
        });
      }
      // Loki expects nanosecond timestamps as strings
      const nsTimestamp = String(entry.timestamp * 1_000_000);
      streams.get(key)!.values.push([nsTimestamp, entry.message]);
    }

    const auth = Buffer.from(`${this.config.username}:${this.config.apiKey}`).toString('base64');
    const url = `${this.config.pushUrl.replace(/\/$/, '')}/loki/api/v1/push`;

    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Basic ${auth}`,
      },
      body: JSON.stringify({ streams: [...streams.values()] }),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`Loki push failed: ${response.status} ${response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const auth = Buffer.from(`${this.config.username}:${this.config.apiKey}`).toString('base64');
    const url = `${this.config.pushUrl.replace(/\/$/, '')}/loki/api/v1/push`;

    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Basic ${auth}`,
      },
      body: JSON.stringify({
        streams: [{
          stream: { job: 'neonexus', source: 'test' },
          values: [[String(Date.now() * 1_000_000), 'NeoNexus connection test']],
        }],
      }),
      signal: AbortSignal.timeout(10_000),
    });
    return response.ok || response.status === 204;
  }
}
