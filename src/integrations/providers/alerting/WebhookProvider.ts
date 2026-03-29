// src/integrations/providers/alerting/WebhookProvider.ts
import type { NotificationProvider, IntegrationEvent, ConfigField } from '../../types';

export const webhookSchema: ConfigField[] = [
  { key: 'url', label: 'Webhook URL', type: 'url', placeholder: 'https://example.com/webhook', required: true },
];

export class WebhookProvider implements NotificationProvider {
  readonly name = 'Webhook';

  constructor(private config: { url: string }) {}

  async notify(event: IntegrationEvent): Promise<void> {
    const response = await fetch(this.config.url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(event),
      signal: AbortSignal.timeout(5_000),
    });

    if (!response.ok) {
      throw new Error(`Webhook failed: ${response.status} ${response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const response = await fetch(this.config.url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'test',
        severity: 'info',
        title: 'NeoNexus Connection Test',
        message: 'This is a test notification from NeoNexus.',
        timestamp: Date.now(),
      }),
      signal: AbortSignal.timeout(5_000),
    });
    return response.ok;
  }
}
