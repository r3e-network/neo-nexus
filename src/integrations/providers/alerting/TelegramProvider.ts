// src/integrations/providers/alerting/TelegramProvider.ts
import type { NotificationProvider, IntegrationEvent, ConfigField } from '../../types';

export const telegramSchema: ConfigField[] = [
  { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: '123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11', required: true },
  { key: 'chatId', label: 'Chat ID', type: 'text', placeholder: '-1001234567890', required: true },
];

const SEVERITY_EMOJI: Record<string, string> = {
  info: '\u2139\ufe0f',
  warning: '\u26a0\ufe0f',
  critical: '\ud83d\udea8',
};

export class TelegramProvider implements NotificationProvider {
  readonly name = 'Telegram';

  constructor(private config: { botToken: string; chatId: string }) {}

  async notify(event: IntegrationEvent): Promise<void> {
    const emoji = SEVERITY_EMOJI[event.severity] || '\u2139\ufe0f';
    const nodeInfo = event.nodeName ? `\n<b>Node:</b> ${this.escapeHtml(event.nodeName)}` : '';

    const text = [
      `${emoji} <b>${this.escapeHtml(event.title)}</b>`,
      ``,
      this.escapeHtml(event.message),
      nodeInfo,
      `<i>${event.severity} \u2022 ${new Date(event.timestamp).toISOString()}</i>`,
    ].join('\n');

    const response = await fetch(`https://api.telegram.org/bot${this.config.botToken}/sendMessage`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        chat_id: this.config.chatId,
        text,
        parse_mode: 'HTML',
        disable_web_page_preview: true,
      }),
      signal: AbortSignal.timeout(5_000),
    });

    if (!response.ok) {
      const body = await response.json().catch(() => ({})) as Record<string, unknown>;
      throw new Error(`Telegram failed: ${(body.description as string) || response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const response = await fetch(`https://api.telegram.org/bot${this.config.botToken}/getMe`, {
      signal: AbortSignal.timeout(5_000),
    });
    if (!response.ok) return false;

    const body = await response.json() as { ok: boolean };
    return body.ok === true;
  }

  private escapeHtml(text: string): string {
    return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
}
