// src/integrations/providers/uptime/UptimeRobotProvider.ts
import type { UptimeProvider, ConfigField } from '../../types';

export const uptimeRobotSchema: ConfigField[] = [
  { key: 'apiKey', label: 'API Key', type: 'password', placeholder: 'ur-api-key-...', required: true },
];

export class UptimeRobotProvider implements UptimeProvider {
  readonly name = 'UptimeRobot';

  constructor(private config: { apiKey: string }) {}

  async registerMonitor(url: string, name: string): Promise<string> {
    const params = new URLSearchParams({
      api_key: this.config.apiKey,
      friendly_name: name,
      url,
      type: '1', // HTTP(s)
      interval: '60',
    });

    const response = await fetch('https://api.uptimerobot.com/v2/newMonitor', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params.toString(),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`UptimeRobot create monitor failed: ${response.status}`);
    }

    const body = await response.json() as { stat: string; monitor: { id: number } };
    if (body.stat !== 'ok') {
      throw new Error('UptimeRobot create monitor failed');
    }
    return String(body.monitor.id);
  }

  async removeMonitor(monitorId: string): Promise<void> {
    const params = new URLSearchParams({
      api_key: this.config.apiKey,
      id: monitorId,
    });

    const response = await fetch('https://api.uptimerobot.com/v2/deleteMonitor', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params.toString(),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`UptimeRobot delete monitor failed: ${response.status}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const params = new URLSearchParams({
      api_key: this.config.apiKey,
      limit: '1',
    });

    const response = await fetch('https://api.uptimerobot.com/v2/getMonitors', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params.toString(),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) return false;
    const body = await response.json() as { stat: string };
    return body.stat === 'ok';
  }
}
