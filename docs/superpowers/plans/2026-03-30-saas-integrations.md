# SaaS Integrations Layer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional, token-gated SaaS integration layer (11 providers across 5 categories) with a dedicated Integrations page.

**Architecture:** Plugin-based — each provider implements a category interface. An `IntegrationManager` singleton loads enabled providers from SQLite, hooks into existing event sources (metrics collector, node manager, watchdog, disk monitor), and fans out to providers. Frontend auto-renders config forms from provider schemas.

**Tech Stack:** Express API routes, SQLite (better-sqlite3), React + TailwindCSS + React Query, `@sentry/node` for Sentry provider, plain `fetch()` for all others.

---

### Task 1: Integration Types and Config Schemas

**Files:**
- Create: `src/integrations/types.ts`

- [ ] **Step 1: Create the integration types file**

```typescript
// src/integrations/types.ts
import type { SystemMetrics, NodeMetrics, LogEntry } from '../types/index';

// --- Provider category interfaces ---

export interface MetricsProvider {
  readonly name: string;
  pushMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface LogProvider {
  readonly name: string;
  pushLogs(entries: LogEntryWithContext[]): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface UptimeProvider {
  readonly name: string;
  registerMonitor(url: string, name: string): Promise<string>;
  removeMonitor(monitorId: string): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface NotificationProvider {
  readonly name: string;
  notify(event: IntegrationEvent): Promise<void>;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

export interface ErrorProvider {
  readonly name: string;
  captureError(error: Error, context?: Record<string, unknown>): void;
  testConnection(): Promise<boolean>;
  shutdown?(): void;
}

// --- Shared data types ---

export interface NodeMetricsWithContext {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  network: string;
  metrics: NodeMetrics;
}

export interface LogEntryWithContext extends LogEntry {
  nodeId: string;
  nodeName: string;
}

export type IntegrationEventType =
  | 'node.started' | 'node.stopped' | 'node.crashed' | 'node.error'
  | 'node.synced' | 'node.behind'
  | 'watchdog.restart' | 'watchdog.exhausted'
  | 'disk.warning' | 'disk.critical'
  | 'security.login_failed' | 'security.default_password';

export type IntegrationEventSeverity = 'info' | 'warning' | 'critical';

export interface IntegrationEvent {
  type: IntegrationEventType;
  severity: IntegrationEventSeverity;
  title: string;
  message: string;
  nodeId?: string;
  nodeName?: string;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

// --- Config schema (drives frontend form rendering) ---

export type ConfigFieldType = 'text' | 'password' | 'url';

export interface ConfigField {
  key: string;
  label: string;
  type: ConfigFieldType;
  placeholder: string;
  required: boolean;
}

export type IntegrationCategory = 'metrics' | 'logging' | 'uptime' | 'alerting' | 'errors';

export type IntegrationId =
  | 'grafana_metrics' | 'datadog'
  | 'betterstack_logging' | 'grafana_loki'
  | 'betterstack_uptime' | 'uptimerobot'
  | 'webhook' | 'slack' | 'discord' | 'telegram'
  | 'sentry';

export interface IntegrationDefinition {
  id: IntegrationId;
  name: string;
  description: string;
  category: IntegrationCategory;
  configSchema: ConfigField[];
  createProvider(config: Record<string, string>): MetricsProvider | LogProvider | UptimeProvider | NotificationProvider | ErrorProvider;
}

// --- Database row ---

export interface IntegrationRow {
  id: string;
  category: string;
  enabled: number;
  config: string;
  last_test_at: string | null;
  last_error: string | null;
  created_at: string;
  updated_at: string;
}

// --- API response types ---

export interface IntegrationStatus {
  id: IntegrationId;
  name: string;
  description: string;
  category: IntegrationCategory;
  enabled: boolean;
  configured: boolean;
  configSchema: ConfigField[];
  configValues: Record<string, string>;
  lastTestAt: string | null;
  lastError: string | null;
}
```

- [ ] **Step 2: Verify the file compiles**

Run: `cd /home/neo/git/neo-nexus && npx tsc --noEmit src/integrations/types.ts 2>&1 | head -20`
Expected: No errors (or only unrelated project-wide errors)

- [ ] **Step 3: Commit**

```bash
git add src/integrations/types.ts
git commit -m "feat(integrations): add types, interfaces, and config schemas"
```

---

### Task 2: Database Schema — Integrations Table

**Files:**
- Modify: `src/database/schema.ts:14-180` (add table inside the `db.exec` block)

- [ ] **Step 1: Add integrations table to schema**

In `src/database/schema.ts`, inside the `db.exec(...)` template literal, after the `audit_log` index (line 179), add:

```sql
    CREATE TABLE IF NOT EXISTS integrations (
      id TEXT PRIMARY KEY,
      category TEXT NOT NULL,
      enabled INTEGER DEFAULT 0,
      config TEXT NOT NULL DEFAULT '{}',
      last_test_at TEXT,
      last_error TEXT,
      created_at TEXT DEFAULT (datetime('now')),
      updated_at TEXT DEFAULT (datetime('now'))
    );
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /home/neo/git/neo-nexus && npx tsc --noEmit`
Expected: Clean compilation

- [ ] **Step 3: Commit**

```bash
git add src/database/schema.ts
git commit -m "feat(db): add integrations table to schema"
```

---

### Task 3: Metrics Providers — Grafana Cloud and Datadog

**Files:**
- Create: `src/integrations/providers/metrics/GrafanaMetricsProvider.ts`
- Create: `src/integrations/providers/metrics/DatadogProvider.ts`

- [ ] **Step 1: Create GrafanaMetricsProvider**

```typescript
// src/integrations/providers/metrics/GrafanaMetricsProvider.ts
import type { MetricsProvider, NodeMetricsWithContext, ConfigField } from '../../types';
import type { SystemMetrics } from '../../../types/index';

export const grafanaMetricsSchema: ConfigField[] = [
  { key: 'remoteWriteUrl', label: 'Remote Write URL', type: 'url', placeholder: 'https://prometheus-prod-01-prod-us-east-0.grafana.net/api/prom/push', required: true },
  { key: 'username', label: 'Username / Instance ID', type: 'text', placeholder: '123456', required: true },
  { key: 'apiKey', label: 'API Key', type: 'password', placeholder: 'glc_...', required: true },
];

export class GrafanaMetricsProvider implements MetricsProvider {
  readonly name = 'Grafana Cloud';

  constructor(private config: { remoteWriteUrl: string; username: string; apiKey: string }) {}

  async pushMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void> {
    const now = Date.now();
    const lines: string[] = [];

    lines.push(`neonexus_cpu_usage ${system.cpu.usage} ${now}`);
    lines.push(`neonexus_memory_usage_percent ${system.memory.percentage} ${now}`);
    lines.push(`neonexus_memory_used_bytes ${system.memory.used} ${now}`);
    lines.push(`neonexus_disk_usage_percent ${system.disk.percentage} ${now}`);
    lines.push(`neonexus_disk_used_bytes ${system.disk.used} ${now}`);
    lines.push(`neonexus_network_rx_bytes ${system.network.rx} ${now}`);
    lines.push(`neonexus_network_tx_bytes ${system.network.tx} ${now}`);

    for (const node of nodes) {
      const labels = `node_id="${node.nodeId}",node_name="${node.nodeName}",network="${node.network}"`;
      lines.push(`neonexus_node_block_height{${labels}} ${node.metrics.blockHeight} ${now}`);
      lines.push(`neonexus_node_peers{${labels}} ${node.metrics.connectedPeers} ${now}`);
      lines.push(`neonexus_node_sync_progress{${labels}} ${node.metrics.syncProgress} ${now}`);
      lines.push(`neonexus_node_memory_bytes{${labels}} ${node.metrics.memoryUsage} ${now}`);
      lines.push(`neonexus_node_cpu{${labels}} ${node.metrics.cpuUsage} ${now}`);
    }

    const body = lines.join('\n') + '\n';
    const auth = Buffer.from(`${this.config.username}:${this.config.apiKey}`).toString('base64');

    const response = await fetch(this.config.remoteWriteUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'text/plain',
        'Authorization': `Basic ${auth}`,
      },
      body,
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`Grafana push failed: ${response.status} ${response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const auth = Buffer.from(`${this.config.username}:${this.config.apiKey}`).toString('base64');
    const response = await fetch(this.config.remoteWriteUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'text/plain',
        'Authorization': `Basic ${auth}`,
      },
      body: `neonexus_test_metric 1 ${Date.now()}\n`,
      signal: AbortSignal.timeout(10_000),
    });
    return response.ok || response.status === 204;
  }
}
```

- [ ] **Step 2: Create DatadogProvider**

```typescript
// src/integrations/providers/metrics/DatadogProvider.ts
import type { MetricsProvider, NodeMetricsWithContext, ConfigField } from '../../types';
import type { SystemMetrics } from '../../../types/index';

export const datadogSchema: ConfigField[] = [
  { key: 'apiKey', label: 'API Key', type: 'password', placeholder: 'dd-api-key-...', required: true },
  { key: 'site', label: 'Datadog Site', type: 'text', placeholder: 'datadoghq.com', required: true },
];

export class DatadogProvider implements MetricsProvider {
  readonly name = 'Datadog';

  constructor(private config: { apiKey: string; site: string }) {}

  async pushMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void> {
    const now = Math.floor(Date.now() / 1000);
    const series: Array<{
      metric: string;
      type: number;
      points: Array<{ timestamp: number; value: number }>;
      tags?: string[];
    }> = [];

    const addMetric = (metric: string, value: number, tags?: string[]) => {
      series.push({
        metric,
        type: 3, // gauge
        points: [{ timestamp: now, value }],
        tags,
      });
    };

    addMetric('neonexus.cpu.usage', system.cpu.usage);
    addMetric('neonexus.memory.percentage', system.memory.percentage);
    addMetric('neonexus.memory.used', system.memory.used);
    addMetric('neonexus.disk.percentage', system.disk.percentage);
    addMetric('neonexus.disk.used', system.disk.used);
    addMetric('neonexus.network.rx', system.network.rx);
    addMetric('neonexus.network.tx', system.network.tx);

    for (const node of nodes) {
      const tags = [`node_id:${node.nodeId}`, `node_name:${node.nodeName}`, `network:${node.network}`];
      addMetric('neonexus.node.block_height', node.metrics.blockHeight, tags);
      addMetric('neonexus.node.peers', node.metrics.connectedPeers, tags);
      addMetric('neonexus.node.sync_progress', node.metrics.syncProgress, tags);
      addMetric('neonexus.node.memory', node.metrics.memoryUsage, tags);
      addMetric('neonexus.node.cpu', node.metrics.cpuUsage, tags);
    }

    const response = await fetch(`https://api.${this.config.site}/api/v2/series`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'DD-API-KEY': this.config.apiKey,
      },
      body: JSON.stringify({ series }),
      signal: AbortSignal.timeout(10_000),
    });

    if (!response.ok) {
      throw new Error(`Datadog push failed: ${response.status} ${response.statusText}`);
    }
  }

  async testConnection(): Promise<boolean> {
    const response = await fetch(`https://api.${this.config.site}/api/v1/validate`, {
      headers: { 'DD-API-KEY': this.config.apiKey },
      signal: AbortSignal.timeout(10_000),
    });
    return response.ok;
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/integrations/providers/metrics/
git commit -m "feat(integrations): add Grafana Cloud and Datadog metrics providers"
```

---

### Task 4: Logging Providers — Better Stack and Grafana Loki

**Files:**
- Create: `src/integrations/providers/logging/BetterStackLoggingProvider.ts`
- Create: `src/integrations/providers/logging/GrafanaLokiProvider.ts`

- [ ] **Step 1: Create BetterStackLoggingProvider**

```typescript
// src/integrations/providers/logging/BetterStackLoggingProvider.ts
import type { LogProvider, LogEntryWithContext, ConfigField } from '../../types';

export const betterStackLoggingSchema: ConfigField[] = [
  { key: 'sourceToken', label: 'Source Token', type: 'password', placeholder: 'your-logtail-source-token', required: true },
];

export class BetterStackLoggingProvider implements LogProvider {
  readonly name = 'Better Stack';

  constructor(private config: { sourceToken: string }) {}

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

    const response = await fetch('https://in.logs.betterstack.com', {
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
    const response = await fetch('https://in.logs.betterstack.com', {
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
}
```

- [ ] **Step 2: Create GrafanaLokiProvider**

```typescript
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
```

- [ ] **Step 3: Commit**

```bash
git add src/integrations/providers/logging/
git commit -m "feat(integrations): add Better Stack and Grafana Loki logging providers"
```

---

### Task 5: Alerting Providers — Webhook, Slack, Discord, Telegram

**Files:**
- Create: `src/integrations/providers/alerting/WebhookProvider.ts`
- Create: `src/integrations/providers/alerting/SlackProvider.ts`
- Create: `src/integrations/providers/alerting/DiscordProvider.ts`
- Create: `src/integrations/providers/alerting/TelegramProvider.ts`

- [ ] **Step 1: Create WebhookProvider**

```typescript
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
```

- [ ] **Step 2: Create SlackProvider**

```typescript
// src/integrations/providers/alerting/SlackProvider.ts
import type { NotificationProvider, IntegrationEvent, ConfigField } from '../../types';

export const slackSchema: ConfigField[] = [
  { key: 'webhookUrl', label: 'Webhook URL', type: 'url', placeholder: 'https://hooks.slack.com/services/T.../B.../...', required: true },
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
```

- [ ] **Step 3: Create DiscordProvider**

```typescript
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

    const fields = [
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
```

- [ ] **Step 4: Create TelegramProvider**

```typescript
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
```

- [ ] **Step 5: Commit**

```bash
git add src/integrations/providers/alerting/
git commit -m "feat(integrations): add Webhook, Slack, Discord, and Telegram alerting providers"
```

---

### Task 6: Uptime Providers — Better Stack Uptime and UptimeRobot

**Files:**
- Create: `src/integrations/providers/uptime/BetterStackUptimeProvider.ts`
- Create: `src/integrations/providers/uptime/UptimeRobotProvider.ts`

- [ ] **Step 1: Create BetterStackUptimeProvider**

```typescript
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
```

- [ ] **Step 2: Create UptimeRobotProvider**

```typescript
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
```

- [ ] **Step 3: Commit**

```bash
git add src/integrations/providers/uptime/
git commit -m "feat(integrations): add Better Stack Uptime and UptimeRobot uptime providers"
```

---

### Task 7: Error Provider — Sentry

**Files:**
- Create: `src/integrations/providers/errors/SentryProvider.ts`

- [ ] **Step 1: Install @sentry/node**

Run: `cd /home/neo/git/neo-nexus && npm install @sentry/node`

- [ ] **Step 2: Create SentryProvider**

```typescript
// src/integrations/providers/errors/SentryProvider.ts
import type { ErrorProvider, ConfigField } from '../../types';

export const sentrySchema: ConfigField[] = [
  { key: 'dsn', label: 'DSN', type: 'password', placeholder: 'https://examplePublicKey@o0.ingest.sentry.io/0', required: true },
];

export class SentryProvider implements ErrorProvider {
  readonly name = 'Sentry';
  private initialized = false;

  constructor(private config: { dsn: string }) {}

  private async ensureInit(): Promise<typeof import('@sentry/node')> {
    const Sentry = await import('@sentry/node');
    if (!this.initialized) {
      Sentry.init({
        dsn: this.config.dsn,
        tracesSampleRate: 0,
        defaultIntegrations: false,
      });
      this.initialized = true;
    }
    return Sentry;
  }

  captureError(error: Error, context?: Record<string, unknown>): void {
    this.ensureInit().then(Sentry => {
      if (context) {
        Sentry.withScope(scope => {
          for (const [key, value] of Object.entries(context)) {
            scope.setExtra(key, value);
          }
          Sentry.captureException(error);
        });
      } else {
        Sentry.captureException(error);
      }
    }).catch(() => {
      // Silently fail — Sentry is best-effort
    });
  }

  async testConnection(): Promise<boolean> {
    try {
      // Validate DSN format
      const url = new URL(this.config.dsn);
      return url.protocol === 'https:' && url.pathname.length > 1;
    } catch {
      return false;
    }
  }

  shutdown(): void {
    if (this.initialized) {
      import('@sentry/node').then(Sentry => {
        Sentry.close(2000);
      }).catch(() => {});
      this.initialized = false;
    }
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/integrations/providers/errors/
git commit -m "feat(integrations): add Sentry error tracking provider"
```

---

### Task 8: Provider Registry

**Files:**
- Create: `src/integrations/registry.ts`

- [ ] **Step 1: Create the registry file that maps IDs to definitions**

```typescript
// src/integrations/registry.ts
import type { IntegrationDefinition } from './types';

import { GrafanaMetricsProvider, grafanaMetricsSchema } from './providers/metrics/GrafanaMetricsProvider';
import { DatadogProvider, datadogSchema } from './providers/metrics/DatadogProvider';
import { BetterStackLoggingProvider, betterStackLoggingSchema } from './providers/logging/BetterStackLoggingProvider';
import { GrafanaLokiProvider, grafanaLokiSchema } from './providers/logging/GrafanaLokiProvider';
import { BetterStackUptimeProvider, betterStackUptimeSchema } from './providers/uptime/BetterStackUptimeProvider';
import { UptimeRobotProvider, uptimeRobotSchema } from './providers/uptime/UptimeRobotProvider';
import { WebhookProvider, webhookSchema } from './providers/alerting/WebhookProvider';
import { SlackProvider, slackSchema } from './providers/alerting/SlackProvider';
import { DiscordProvider, discordSchema } from './providers/alerting/DiscordProvider';
import { TelegramProvider, telegramSchema } from './providers/alerting/TelegramProvider';
import { SentryProvider, sentrySchema } from './providers/errors/SentryProvider';

export const integrationRegistry: IntegrationDefinition[] = [
  {
    id: 'grafana_metrics',
    name: 'Grafana Cloud',
    description: 'Push system and node metrics to Grafana Cloud',
    category: 'metrics',
    configSchema: grafanaMetricsSchema,
    createProvider: (config) => new GrafanaMetricsProvider(config as { remoteWriteUrl: string; username: string; apiKey: string }),
  },
  {
    id: 'datadog',
    name: 'Datadog',
    description: 'Send metrics to Datadog for monitoring and dashboards',
    category: 'metrics',
    configSchema: datadogSchema,
    createProvider: (config) => new DatadogProvider(config as { apiKey: string; site: string }),
  },
  {
    id: 'betterstack_logging',
    name: 'Better Stack',
    description: 'Ship structured logs to Better Stack (Logtail)',
    category: 'logging',
    configSchema: betterStackLoggingSchema,
    createProvider: (config) => new BetterStackLoggingProvider(config as { sourceToken: string }),
  },
  {
    id: 'grafana_loki',
    name: 'Grafana Loki',
    description: 'Push logs to Grafana Loki for aggregation and search',
    category: 'logging',
    configSchema: grafanaLokiSchema,
    createProvider: (config) => new GrafanaLokiProvider(config as { pushUrl: string; username: string; apiKey: string }),
  },
  {
    id: 'betterstack_uptime',
    name: 'Better Stack Uptime',
    description: 'Monitor NeoNexus uptime with Better Stack',
    category: 'uptime',
    configSchema: betterStackUptimeSchema,
    createProvider: (config) => new BetterStackUptimeProvider(config as { apiToken: string }),
  },
  {
    id: 'uptimerobot',
    name: 'UptimeRobot',
    description: 'Monitor NeoNexus uptime with UptimeRobot',
    category: 'uptime',
    configSchema: uptimeRobotSchema,
    createProvider: (config) => new UptimeRobotProvider(config as { apiKey: string }),
  },
  {
    id: 'webhook',
    name: 'Webhook',
    description: 'Send event notifications to any URL as JSON',
    category: 'alerting',
    configSchema: webhookSchema,
    createProvider: (config) => new WebhookProvider(config as { url: string }),
  },
  {
    id: 'slack',
    name: 'Slack',
    description: 'Send alerts to a Slack channel via incoming webhook',
    category: 'alerting',
    configSchema: slackSchema,
    createProvider: (config) => new SlackProvider(config as { webhookUrl: string }),
  },
  {
    id: 'discord',
    name: 'Discord',
    description: 'Send alerts to a Discord channel via webhook',
    category: 'alerting',
    configSchema: discordSchema,
    createProvider: (config) => new DiscordProvider(config as { webhookUrl: string }),
  },
  {
    id: 'telegram',
    name: 'Telegram',
    description: 'Send alerts to a Telegram chat via bot',
    category: 'alerting',
    configSchema: telegramSchema,
    createProvider: (config) => new TelegramProvider(config as { botToken: string; chatId: string }),
  },
  {
    id: 'sentry',
    name: 'Sentry',
    description: 'Track uncaught errors and exceptions with Sentry',
    category: 'errors',
    configSchema: sentrySchema,
    createProvider: (config) => new SentryProvider(config as { dsn: string }),
  },
];

export const registryMap = new Map(integrationRegistry.map(d => [d.id, d]));
```

- [ ] **Step 2: Commit**

```bash
git add src/integrations/registry.ts
git commit -m "feat(integrations): add provider registry mapping IDs to definitions"
```

---

### Task 9: IntegrationManager

**Files:**
- Create: `src/integrations/IntegrationManager.ts`

- [ ] **Step 1: Create IntegrationManager**

```typescript
// src/integrations/IntegrationManager.ts
import type Database from 'better-sqlite3';
import type { SystemMetrics, NodeMetrics } from '../types/index';
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
        // uptime providers don't need to be stored — they act on enable/disable
      }
    } catch (error) {
      this.updateLastError(row.id, error instanceof Error ? error.message : 'Failed to initialize');
    }
  }

  reloadProvider(id: string): void {
    // Remove existing instances for this provider
    const definition = registryMap.get(id as IntegrationId);
    if (!definition) return;

    this.removeProviderInstances(id, definition.category);

    // Re-instantiate if enabled
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
        // Only update if not a redacted value placeholder
        if (value && !value.match(/^\*+\.{3}\w{4}$/)) {
          mergedConfig[key] = value;
        }
      }
    }

    if (existing) {
      this.db.prepare(
        'UPDATE integrations SET config = ?, enabled = ?, last_error = NULL, updated_at = datetime(\'now\') WHERE id = ?'
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
          'UPDATE integrations SET last_test_at = datetime(\'now\'), last_error = NULL WHERE id = ?'
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
      'UPDATE integrations SET last_error = ?, updated_at = datetime(\'now\') WHERE id = ?'
    ).run(error, id);
  }

  private handleProviderError(providerName: string, error: unknown): void {
    const msg = error instanceof Error ? error.message : String(error);
    console.error(`[integrations] ${providerName} error: ${msg}`);

    // Find the integration ID by provider name and update last_error
    for (const def of integrationRegistry) {
      if (def.name === providerName) {
        this.updateLastError(def.id, msg);
        break;
      }
    }
  }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /home/neo/git/neo-nexus && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add src/integrations/IntegrationManager.ts
git commit -m "feat(integrations): add IntegrationManager singleton with CRUD and broadcasting"
```

---

### Task 10: API Routes for Integrations

**Files:**
- Create: `src/api/routes/integrations.ts`

- [ ] **Step 1: Create the integrations router**

```typescript
// src/api/routes/integrations.ts
import { Router, type Request, type Response } from 'express';
import type { IntegrationManager } from '../../integrations/IntegrationManager';

export function createIntegrationsRouter(integrationManager: IntegrationManager): Router {
  const router = Router();

  // List all integrations with status
  router.get('/', (_req: Request, res: Response) => {
    try {
      const integrations = integrationManager.listAll();
      res.json({ integrations });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : 'Internal server error' });
    }
  });

  // Get single integration
  router.get('/:id', (req: Request, res: Response) => {
    try {
      const integration = integrationManager.getOne(req.params.id as string);
      if (!integration) {
        return res.status(404).json({ error: 'Integration not found' });
      }
      res.json({ integration });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : 'Internal server error' });
    }
  });

  // Save config and enable/disable
  router.put('/:id', (req: Request, res: Response) => {
    try {
      const { config, enabled } = req.body as { config?: Record<string, string>; enabled?: boolean };
      if (!config || typeof enabled !== 'boolean') {
        return res.status(400).json({ error: 'Missing required fields: config (object), enabled (boolean)' });
      }
      integrationManager.saveConfig(req.params.id as string, config, enabled);
      const integration = integrationManager.getOne(req.params.id as string);
      res.json({ integration });
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : 'Bad request' });
    }
  });

  // Test connection
  router.post('/:id/test', async (req: Request, res: Response) => {
    try {
      const result = await integrationManager.testProvider(req.params.id as string);
      res.json(result);
    } catch (error) {
      res.status(500).json({ success: false, error: error instanceof Error ? error.message : 'Test failed' });
    }
  });

  // Delete / clear config
  router.delete('/:id', (req: Request, res: Response) => {
    try {
      integrationManager.deleteConfig(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : 'Bad request' });
    }
  });

  return router;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/routes/integrations.ts
git commit -m "feat(api): add /api/integrations CRUD and test routes"
```

---

### Task 11: Server Wiring — Routes and Event Hooks

**Files:**
- Modify: `src/server.ts`

This task wires the IntegrationManager into the existing server: adds the API route, hooks into the metrics broadcast loop, node status events, watchdog events, and disk alerts.

- [ ] **Step 1: Add imports to server.ts**

At the top of `src/server.ts`, after the existing imports (after line 31), add:

```typescript
import { IntegrationManager } from "./integrations/IntegrationManager";
import { createIntegrationsRouter } from "./api/routes/integrations";
import type { IntegrationEvent, LogEntryWithContext } from "./integrations/types";
```

- [ ] **Step 2: Initialize IntegrationManager and log buffer**

In `createAppServer()`, after `const auditLogger = new AuditLogger(config.db);` (line 99), add:

```typescript
  const integrationManager = new IntegrationManager(config.db);
  const logBuffer: LogEntryWithContext[] = [];
```

- [ ] **Step 3: Add the API route**

After `app.use("/api/secure-signers", ...)` (line 214), add:

```typescript
  app.use("/api/integrations", requireAuth, requireAdmin, createIntegrationsRouter(integrationManager));
```

- [ ] **Step 4: Hook into nodeStatus events for notifications**

In the existing `nodeManager.on("nodeStatus", ...)` handler (around line 290), after the watchdog calls (after line 302), add:

```typescript
    // Integration notifications
    const node = nodeManager.getNode(nodeId);
    const nodeName = node?.name ?? nodeId;
    let integrationEvent: IntegrationEvent | null = null;

    if (status === 'running' && previousStatus === 'starting') {
      integrationEvent = { type: 'node.started', severity: 'info', title: 'Node Started', message: `Node ${nodeName} has started.`, nodeId, nodeName, timestamp: Date.now() };
    } else if (status === 'stopped' && previousStatus === 'stopping') {
      integrationEvent = { type: 'node.stopped', severity: 'info', title: 'Node Stopped', message: `Node ${nodeName} has stopped.`, nodeId, nodeName, timestamp: Date.now() };
    } else if (status === 'error') {
      integrationEvent = { type: 'node.crashed', severity: 'critical', title: 'Node Crashed', message: `Node ${nodeName} has crashed.`, nodeId, nodeName, timestamp: Date.now() };
    }

    if (integrationEvent) {
      integrationManager.broadcastNotification(integrationEvent).catch(() => {});
    }
```

- [ ] **Step 5: Buffer logs for integration shipping**

In the existing `nodeManager.on("nodeLog", ...)` handler (around line 305), after `broadcast(buildNodeLogMessage(...))`, add:

```typescript
    const logNode = nodeManager.getNode(nodeId);
    logBuffer.push({ ...entry, nodeId, nodeName: logNode?.name ?? nodeId });
```

- [ ] **Step 6: Hook into the metrics interval for metrics + log flushing**

In the existing `metricsInterval` callback (around line 314), after `broadcast(buildSystemMessage(systemMetrics));` and before the node metrics loop, add:

```typescript
      // Flush buffered logs to integration providers
      if (logBuffer.length > 0) {
        const batch = logBuffer.splice(0, logBuffer.length);
        integrationManager.broadcastLogs(batch).catch(() => {});
      }

      // Push system + node metrics to integration providers
      const allNodes = nodeManager.getAllNodes();
      const nodeMetricsForIntegration = allNodes
        .filter(n => n.process.status === 'running' && n.metrics)
        .map(n => ({
          nodeId: n.id,
          nodeName: n.name,
          nodeType: n.type,
          network: n.network,
          metrics: n.metrics!,
        }));
      integrationManager.broadcastMetrics(systemMetrics, nodeMetricsForIntegration).catch(() => {});
```

- [ ] **Step 7: Hook into disk alerts**

In the existing disk alert section (around line 322), after `console.warn(...)`, add inside the `if (diskAlert !== null)` block:

```typescript
        integrationManager.broadcastNotification({
          type: diskAlert === 'critical' ? 'disk.critical' : 'disk.warning',
          severity: diskAlert === 'critical' ? 'critical' : 'warning',
          title: `Disk ${diskAlert === 'critical' ? 'Critical' : 'Warning'}`,
          message: `Disk usage at ${systemMetrics.disk.percentage.toFixed(1)}%`,
          timestamp: Date.now(),
        }).catch(() => {});
```

- [ ] **Step 8: Hook into watchdog events**

In the existing `nodeManager.on("nodeStatus", ...)` handler, update the watchdog section. After `watchdog.onNodeExited(nodeId, wasExpected);` add:

```typescript
      if (!wasExpected) {
        integrationManager.broadcastNotification({
          type: watchdog.isExhausted(nodeId) ? 'watchdog.exhausted' : 'watchdog.restart',
          severity: watchdog.isExhausted(nodeId) ? 'critical' : 'warning',
          title: watchdog.isExhausted(nodeId) ? 'Watchdog Exhausted' : 'Watchdog Restart',
          message: watchdog.isExhausted(nodeId)
            ? `Node ${nodeName} crashed too many times. Watchdog giving up.`
            : `Node ${nodeName} crashed unexpectedly. Watchdog scheduling restart.`,
          nodeId,
          nodeName,
          timestamp: Date.now(),
        }).catch(() => {});
      }
```

- [ ] **Step 9: Hook into uncaught errors for Sentry**

In `src/index.ts`, in the `uncaughtException` handler (line 38), add before `await server.stop()`:

```typescript
      // Forward to Sentry if configured
      if (server.integrationManager) {
        server.integrationManager.captureError(error, { source: 'uncaughtException' });
      }
```

And expose `integrationManager` from the return object of `createAppServer` (around line 454), add it alongside the existing exports:

```typescript
    integrationManager,
```

- [ ] **Step 10: Shutdown IntegrationManager on server stop**

In the `stop()` function of `createAppServer`, after `watchdog.clearAll();` (line 419), add:

```typescript
    integrationManager.shutdown();
```

- [ ] **Step 11: Verify compilation**

Run: `cd /home/neo/git/neo-nexus && npx tsc --noEmit`

- [ ] **Step 12: Commit**

```bash
git add src/server.ts src/index.ts
git commit -m "feat(server): wire IntegrationManager into routes, events, and metrics loop"
```

---

### Task 12: Frontend Hook — useIntegrations

**Files:**
- Create: `web/src/hooks/useIntegrations.ts`

- [ ] **Step 1: Create the hook**

```typescript
// web/src/hooks/useIntegrations.ts
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api } from '../utils/api';

export type IntegrationCategory = 'metrics' | 'logging' | 'uptime' | 'alerting' | 'errors';

export interface ConfigField {
  key: string;
  label: string;
  type: 'text' | 'password' | 'url';
  placeholder: string;
  required: boolean;
}

export interface IntegrationStatus {
  id: string;
  name: string;
  description: string;
  category: IntegrationCategory;
  enabled: boolean;
  configured: boolean;
  configSchema: ConfigField[];
  configValues: Record<string, string>;
  lastTestAt: string | null;
  lastError: string | null;
}

export function useIntegrations() {
  return useQuery({
    queryKey: ['integrations'],
    queryFn: async () => {
      const response = await api.get<{ integrations: IntegrationStatus[] }>('/integrations');
      return response.integrations;
    },
  });
}

export function useIntegration(id: string) {
  return useQuery({
    queryKey: ['integrations', id],
    queryFn: async () => {
      const response = await api.get<{ integration: IntegrationStatus }>(`/integrations/${id}`);
      return response.integration;
    },
    enabled: !!id,
  });
}

export function useSaveIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, config, enabled }: { id: string; config: Record<string, string>; enabled: boolean }) => {
      const response = await api.put<{ integration: IntegrationStatus }>(`/integrations/${id}`, { config, enabled } as unknown as Record<string, unknown>);
      return response.integration;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}

export function useTestIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      const response = await api.post<{ success: boolean; error?: string }>(`/integrations/${id}/test`);
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}

export function useDeleteIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/integrations/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] });
    },
  });
}
```

- [ ] **Step 2: Commit**

```bash
git add web/src/hooks/useIntegrations.ts
git commit -m "feat(web): add useIntegrations hook for integration CRUD"
```

---

### Task 13: IntegrationCard Component

**Files:**
- Create: `web/src/components/IntegrationCard.tsx`

- [ ] **Step 1: Create the component**

```tsx
// web/src/components/IntegrationCard.tsx
import { useState, useEffect } from 'react';
import { CheckCircle2, XCircle, Loader2, Eye, EyeOff } from 'lucide-react';
import type { IntegrationStatus } from '../hooks/useIntegrations';

interface IntegrationCardProps {
  integration: IntegrationStatus;
  onSave: (id: string, config: Record<string, string>, enabled: boolean) => Promise<void>;
  onTest: (id: string) => Promise<{ success: boolean; error?: string }>;
  isSaving: boolean;
  isTesting: boolean;
}

export function IntegrationCard({ integration, onSave, onTest, isSaving, isTesting }: IntegrationCardProps) {
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [enabled, setEnabled] = useState(integration.enabled);
  const [testResult, setTestResult] = useState<{ success: boolean; error?: string } | null>(null);
  const [revealedFields, setRevealedFields] = useState<Set<string>>(new Set());

  useEffect(() => {
    setFormValues(integration.configValues);
    setEnabled(integration.enabled);
  }, [integration]);

  const handleSaveAndTest = async () => {
    setTestResult(null);
    await onSave(integration.id, formValues, true);
    const result = await onTest(integration.id);
    setTestResult(result);
    if (result.success) {
      setEnabled(true);
    }
  };

  const handleDisable = async () => {
    setTestResult(null);
    await onSave(integration.id, formValues, false);
    setEnabled(false);
  };

  const toggleReveal = (key: string) => {
    setRevealedFields(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const statusColor = integration.lastError
    ? 'text-amber-400'
    : integration.enabled && integration.configured
      ? 'text-emerald-400'
      : 'text-slate-500';

  const statusLabel = integration.lastError
    ? 'Error'
    : integration.enabled && integration.configured
      ? 'Connected'
      : 'Not configured';

  return (
    <div className="card space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-white">{integration.name}</h3>
          <p className="text-sm text-slate-400 mt-1">{integration.description}</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`w-2 h-2 rounded-full ${statusColor.replace('text-', 'bg-')}`} />
          <span className={`text-xs font-medium ${statusColor}`}>{statusLabel}</span>
        </div>
      </div>

      {integration.lastError && (
        <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2 text-sm text-amber-300">
          {integration.lastError}
        </div>
      )}

      <div className="space-y-3">
        {integration.configSchema.map(field => (
          <div key={field.key}>
            <label className="block text-sm font-medium text-slate-300 mb-1.5">{field.label}</label>
            <div className="relative">
              <input
                type={field.type === 'password' && !revealedFields.has(field.key) ? 'password' : 'text'}
                className="input w-full pr-10"
                value={formValues[field.key] || ''}
                onChange={e => setFormValues(prev => ({ ...prev, [field.key]: e.target.value }))}
                placeholder={field.placeholder}
              />
              {field.type === 'password' && (
                <button
                  type="button"
                  onClick={() => toggleReveal(field.key)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-slate-300"
                >
                  {revealedFields.has(field.key) ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      {testResult && (
        <div className={`flex items-center gap-2 text-sm ${testResult.success ? 'text-emerald-400' : 'text-red-400'}`}>
          {testResult.success ? <CheckCircle2 className="w-4 h-4" /> : <XCircle className="w-4 h-4" />}
          {testResult.success ? 'Connection successful' : testResult.error || 'Connection failed'}
        </div>
      )}

      <div className="flex gap-3 pt-1">
        <button
          type="button"
          className="btn btn-primary flex-1 justify-center"
          disabled={isSaving || isTesting}
          onClick={handleSaveAndTest}
        >
          {(isSaving || isTesting) && <Loader2 className="w-4 h-4 animate-spin" />}
          Save & Test
        </button>
        {enabled && (
          <button
            type="button"
            className="btn btn-secondary"
            disabled={isSaving}
            onClick={handleDisable}
          >
            Disable
          </button>
        )}
      </div>

      {integration.lastTestAt && (
        <p className="text-xs text-slate-500">
          Last tested: {new Date(integration.lastTestAt).toLocaleString()}
        </p>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add web/src/components/IntegrationCard.tsx
git commit -m "feat(web): add IntegrationCard component with auto-rendered config forms"
```

---

### Task 14: Integrations Page

**Files:**
- Create: `web/src/pages/Integrations.tsx`

- [ ] **Step 1: Create the page**

```tsx
// web/src/pages/Integrations.tsx
import { useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { CardSkeleton } from '../components/LoadingSkeleton';
import { EmptyState } from '../components/EmptyState';
import { IntegrationCard } from '../components/IntegrationCard';
import {
  useIntegrations,
  useSaveIntegration,
  useTestIntegration,
  type IntegrationCategory,
} from '../hooks/useIntegrations';

const CATEGORIES: { key: IntegrationCategory | 'all'; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'metrics', label: 'Metrics' },
  { key: 'logging', label: 'Logging' },
  { key: 'uptime', label: 'Uptime' },
  { key: 'alerting', label: 'Alerting' },
  { key: 'errors', label: 'Errors' },
];

export default function Integrations() {
  const { data: integrations = [], isLoading, refetch, isFetching } = useIntegrations();
  const saveIntegration = useSaveIntegration();
  const testIntegration = useTestIntegration();
  const [activeCategory, setActiveCategory] = useState<IntegrationCategory | 'all'>('all');
  const [activeId, setActiveId] = useState<string | null>(null);

  const filtered = activeCategory === 'all'
    ? integrations
    : integrations.filter(i => i.category === activeCategory);

  // Group by category when showing "all"
  const grouped = activeCategory === 'all'
    ? CATEGORIES.filter(c => c.key !== 'all').map(c => ({
        ...c,
        items: integrations.filter(i => i.category === c.key),
      })).filter(g => g.items.length > 0)
    : [{ key: activeCategory, label: CATEGORIES.find(c => c.key === activeCategory)?.label || '', items: filtered }];

  const handleSave = async (id: string, config: Record<string, string>, enabled: boolean) => {
    setActiveId(id);
    await saveIntegration.mutateAsync({ id, config, enabled });
  };

  const handleTest = async (id: string) => {
    setActiveId(id);
    return testIntegration.mutateAsync(id);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white">Integrations</h1>
          <p className="text-slate-400 mt-1">Connect NeoNexus to external monitoring, logging, and notification services.</p>
        </div>
        <button
          className="btn btn-secondary"
          onClick={() => refetch()}
          type="button"
        >
          <RefreshCw className={`w-4 h-4 ${isFetching ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Category tabs */}
      <div className="flex gap-1 overflow-x-auto pb-1">
        {CATEGORIES.map(cat => (
          <button
            key={cat.key}
            type="button"
            onClick={() => setActiveCategory(cat.key)}
            className={`px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors ${
              activeCategory === cat.key
                ? 'bg-blue-500/10 text-blue-400'
                : 'text-slate-400 hover:text-white hover:bg-slate-800'
            }`}
          >
            {cat.label}
          </button>
        ))}
      </div>

      {/* Content */}
      {isLoading ? (
        <CardSkeleton count={4} />
      ) : integrations.length === 0 ? (
        <EmptyState
          title="No integrations available"
          description="Integration providers are not loaded."
        />
      ) : (
        <div className="space-y-8">
          {grouped.map(group => (
            <div key={group.key}>
              {activeCategory === 'all' && (
                <h2 className="text-lg font-semibold text-white mb-4 capitalize">{group.label}</h2>
              )}
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
                {group.items.map(integration => (
                  <IntegrationCard
                    key={integration.id}
                    integration={integration}
                    onSave={handleSave}
                    onTest={handleTest}
                    isSaving={saveIntegration.isPending && activeId === integration.id}
                    isTesting={testIntegration.isPending && activeId === integration.id}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add web/src/pages/Integrations.tsx
git commit -m "feat(web): add Integrations page with category tabs and provider cards"
```

---

### Task 15: App & Layout Wiring

**Files:**
- Modify: `web/src/App.tsx`
- Modify: `web/src/components/Layout.tsx`

- [ ] **Step 1: Add Integrations route to App.tsx**

In `web/src/App.tsx`, add the import after the Servers import (line 12):

```typescript
import Integrations from './pages/Integrations';
```

Add the route after the `/servers` route (line 149), before the `/plugins` route:

```tsx
      <Route path="/integrations" element={<ProtectedPage><Integrations /></ProtectedPage>} />
```

- [ ] **Step 2: Add Integrations to sidebar navigation in Layout.tsx**

In `web/src/components/Layout.tsx`, add `Plug` to the lucide-react import (line 3):

```typescript
import {
  LayoutDashboard,
  Server,
  Puzzle,
  Settings,
  Network,
  Plug,
  Menu,
  Github,
  Activity,
  User,
  LogOut,
  Shield,
  Bell,
  CheckCircle2,
  AlertTriangle,
  AlertOctagon,
  X
} from 'lucide-react';
```

Add the Integrations nav item to `navItems` array after the Servers entry (after line 31):

```typescript
  { path: '/integrations', icon: Plug, label: 'Integrations' },
```

So the full array becomes:

```typescript
const navItems = [
  { path: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { path: '/nodes', icon: Server, label: 'Nodes' },
  { path: '/servers', icon: Network, label: 'Servers' },
  { path: '/integrations', icon: Plug, label: 'Integrations' },
  { path: '/plugins', icon: Puzzle, label: 'Features' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];
```

- [ ] **Step 3: Verify frontend compiles**

Run: `cd /home/neo/git/neo-nexus/web && npx tsc --noEmit`

- [ ] **Step 4: Commit**

```bash
git add web/src/App.tsx web/src/components/Layout.tsx
git commit -m "feat(web): add Integrations route and sidebar navigation item"
```

---

### Task 16: Full Build and Smoke Test

**Files:** None (verification only)

- [ ] **Step 1: Run full backend typecheck**

Run: `cd /home/neo/git/neo-nexus && npx tsc --noEmit`
Expected: Clean compilation

- [ ] **Step 2: Run full frontend typecheck**

Run: `cd /home/neo/git/neo-nexus/web && npx tsc --noEmit`
Expected: Clean compilation

- [ ] **Step 3: Run existing tests**

Run: `cd /home/neo/git/neo-nexus && npm test`
Expected: All existing tests pass (no regressions)

- [ ] **Step 4: Run frontend build**

Run: `cd /home/neo/git/neo-nexus/web && npm run build`
Expected: Successful build

- [ ] **Step 5: Run full build**

Run: `cd /home/neo/git/neo-nexus && npm run build`
Expected: Successful build

- [ ] **Step 6: Fix any issues found**

If any step above fails, fix the issues and re-run. Commit fixes.

- [ ] **Step 7: Final commit**

```bash
git add -A
git commit -m "chore: fix any build issues from SaaS integrations feature"
```
