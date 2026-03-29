// src/integrations/providers/alerting/DiscordProvider.ts
import type { NotificationProvider, IntegrationEvent, ConfigField } from '../../types';

export const discordSchema: ConfigField[] = [
  { key: 'webhookUrl', label: 'Webhook URL', type: 'url', placeholder: 'https://discord.com/api/webhooks/...', required: true },
];

const SEVERITY_COLORS: Record<string, number> = {
  info: 0x2563eb,
  warning: 0xf59e0b,
  critical: 0xef4444,
};

export class DiscordProvider implements NotificationProvider {
  readonly name = 'Discord';

  constructor(private config: { webhookUrl: string }) {}

  async notify(event: IntegrationEvent): Promise<void> {
    const color = SEVERITY_COLORS[event.severity] || 0x64748b;

    const fields: { name: string; value: string; inline: boolean }[] = [
      { name: 'Severity', value: event.severity, inline: true },
      { name: 'Type', value: event.type, inline: true },
    ];
    if (event.nodeName) {
      fields.push({ name: 'Node', value: event.nodeName, inline: true });
    }

    const payload = {
      embeds: [{
        title: event.title,
        description: event.message,
        color,
        fields,
        timestamp: new Date(event.timestamp).toISOString(),
        footer: { text: 'NeoNexus Node Manager' },
      }],
    };

    const response = await fetch(this.config.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(5_000),
    });

    if (!response.ok) {
      throw new Error(`Discord webhook failed: ${response.status}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const payload = {
      embeds: [{
        title: 'NeoNexus Connection Test',
        description: 'Connection test successful.',
        color: 0x22c55e,
        timestamp: new Date().toISOString(),
      }],
    };
    const response = await fetch(this.config.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(5_000),
    });
    return response.ok;
  }
}
