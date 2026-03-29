// src/integrations/providers/uptime/BetterStackUptimeProvider.ts
import type { UptimeProvider, ConfigField } from '../../types';

export const betterStackUptimeSchema: ConfigField[] = [
  { key: 'apiToken', label: 'API Token', type: 'password', placeholder: 'your-betterstack-api-token', required: true },
];

export class BetterStackUptimeProvider implements UptimeProvider {
  readonly name = 'Better Stack Uptime';

  constructor(private config: { apiToken: string }) {}

  async registerMonitor(url: string, name: string): Promise<string> {
    const response = await fetch('https://uptime.betterstack.com/api/v2/monitors', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.config.apiToken}`,
      },
      body: JSON.stringify({
        monitor_type: 'status',
        url,
        pronounceable_name: name,
        check_frequency: 60,
      }),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`Better Stack create monitor failed: ${response.status}`);
    }

    const body = await response.json() as { data: { id: string } };
    return body.data.id;
  }

  async removeMonitor(monitorId: string): Promise<void> {
    const response = await fetch(`https://uptime.betterstack.com/api/v2/monitors/${monitorId}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${this.config.apiToken}` },
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok && response.status !== 404) {
      throw new Error(`Better Stack delete monitor failed: ${response.status}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const response = await fetch('https://uptime.betterstack.com/api/v2/monitors?per_page=1', {
      headers: { 'Authorization': `Bearer ${this.config.apiToken}` },
      signal: AbortSignal.timeout(10_000),
    });
    return response.ok;
  }
}
