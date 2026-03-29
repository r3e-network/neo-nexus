// src/integrations/providers/alerting/SlackProvider.ts
import type { NotificationProvider, IntegrationEvent, ConfigField } from '../../types';

export const slackSchema: ConfigField[] = [
  { key: 'webhookUrl', label: 'Webhook URL', type: 'url', placeholder: 'https://hooks.slack.com/services/T.../B.../...', required: true, sensitive: true },
];

const SEVERITY_COLORS: Record<string, string> = {
  info: '#2563eb',
  warning: '#f59e0b',
  critical: '#ef4444',
};

export class SlackProvider implements NotificationProvider {
  readonly name = 'Slack';

  constructor(private config: { webhookUrl: string }) {}

  async notify(event: IntegrationEvent): Promise<void> {
    const color = SEVERITY_COLORS[event.severity] || '#64748b';

    const payload = {
      attachments: [{
        color,
        blocks: [
          {
            type: 'section',
            text: { type: 'mrkdwn', text: `*${event.title}*\n${event.message}` },
          },
          {
            type: 'context',
            elements: [
              { type: 'mrkdwn', text: `*Severity:* ${event.severity}` },
              ...(event.nodeName ? [{ type: 'mrkdwn', text: `*Node:* ${event.nodeName}` }] : []),
              { type: 'mrkdwn', text: `*Time:* ${new Date(event.timestamp).toISOString()}` },
            ],
          },
        ],
      }],
    };

    const response = await fetch(this.config.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(5_000),
    });

    if (!response.ok) {
      throw new Error(`Slack webhook failed: ${response.status}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const payload = {
      text: ':white_check_mark: NeoNexus connection test successful.',
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
