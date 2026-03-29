# SaaS Integrations Layer — Design Spec

**Date:** 2026-03-30
**Status:** Approved

## Overview

Add an optional SaaS integration layer to NeoNexus. The existing internal monitoring, logging, and alerting remain the baseline. Each SaaS provider is purely additive — only activated when the user fills in credentials on a dedicated Integrations page. If no tokens are configured, the system behaves exactly as it does today.

## Providers

### Metrics (push every 5s on existing interval)

| Provider | ID | Fields | Transport |
|----------|----|--------|-----------|
| Grafana Cloud | `grafana_metrics` | Remote Write URL, Username, API Key | Grafana Cloud HTTP Metrics API (POST JSON to `/api/v1/push` with Basic auth) |
| Datadog | `datadog` | API Key, Site (e.g. `datadoghq.com`) | POST JSON to `https://api.${site}/api/v2/series` |

**Metrics emitted:** `neonexus_cpu_usage`, `neonexus_memory_usage`, `neonexus_disk_usage`, `neonexus_node_block_height`, `neonexus_node_peers`, `neonexus_node_sync_progress`, `neonexus_node_memory`, `neonexus_node_cpu`. Tagged with `node_id`, `node_name`, `network`.

### Logging (push every 5s, batched)

| Provider | ID | Fields | Transport |
|----------|----|--------|-----------|
| Better Stack (Logtail) | `betterstack_logging` | Source Token | POST JSON to `https://in.logs.betterstack.com` with Bearer auth |
| Grafana Loki | `grafana_loki` | Push URL, Username, API Key | POST to `/loki/api/v1/push` with Basic auth, streams labeled by `node_id`, `level`, `source` |

### Uptime (monitor lifecycle on enable/disable)

| Provider | ID | Fields | Transport |
|----------|----|--------|-----------|
| Better Stack Uptime | `betterstack_uptime` | API Token | REST API to create/delete HTTP monitor for `/api/health` |
| UptimeRobot | `uptimerobot` | API Key | REST API to create/delete HTTP monitor for `/api/health` |

### Alerting / Notifications (on events)

| Provider | ID | Fields | Transport |
|----------|----|--------|-----------|
| Webhook | `webhook` | URL | POST `IntegrationEvent` as JSON, 5s timeout |
| Slack | `slack` | Webhook URL | POST block kit message with severity color |
| Discord | `discord` | Webhook URL | POST embed with color-coded severity |
| Telegram | `telegram` | Bot Token, Chat ID | POST to `https://api.telegram.org/bot${token}/sendMessage` with HTML |

### Error Tracking

| Provider | ID | Fields | Transport |
|----------|----|--------|-----------|
| Sentry | `sentry` | DSN | `@sentry/node` SDK, lazy-init on first error |

## Event Types

All enabled notification providers receive all events. No per-event filtering in v1.

```typescript
interface IntegrationEvent {
  type: 'node.started' | 'node.stopped' | 'node.crashed' | 'node.error'
      | 'node.synced' | 'node.behind'
      | 'watchdog.restart' | 'watchdog.exhausted'
      | 'disk.warning' | 'disk.critical'
      | 'security.login_failed' | 'security.default_password';
  severity: 'info' | 'warning' | 'critical';
  title: string;
  message: string;
  nodeId?: string;
  nodeName?: string;
  timestamp: number;
  metadata?: Record<string, unknown>;
}
```

## Provider Interfaces

```typescript
interface MetricsProvider {
  name: string;
  pushMetrics(system: SystemMetrics, nodes: NodeMetrics[]): Promise<void>;
  testConnection(): Promise<boolean>;
}

interface LogProvider {
  name: string;
  pushLogs(entries: LogEntry[]): Promise<void>;
  testConnection(): Promise<boolean>;
}

interface UptimeProvider {
  name: string;
  registerMonitor(url: string, name: string): Promise<void>;
  removeMonitor(id: string): Promise<void>;
  testConnection(): Promise<boolean>;
}

interface NotificationProvider {
  name: string;
  notify(event: IntegrationEvent): Promise<void>;
  testConnection(): Promise<boolean>;
}

interface ErrorProvider {
  name: string;
  captureError(error: Error, context?: Record<string, unknown>): Promise<void>;
  testConnection(): Promise<boolean>;
}
```

Each provider class also exports a static `configSchema` array describing its fields:

```typescript
interface ConfigField {
  key: string;        // e.g. 'apiKey'
  label: string;      // e.g. 'API Key'
  type: 'text' | 'password' | 'url';
  placeholder: string;
  required: boolean;
}
```

## Integration Manager

Singleton `IntegrationManager`:

- Loads all enabled integrations from the database on startup
- Instantiates provider classes with their saved configs
- Exposes: `broadcastMetrics()`, `broadcastLogs()`, `broadcastNotification()`, `captureError()`
- Each call fans out to all enabled providers of that category
- Every provider call is wrapped in try/catch — failures are logged to `last_error` on the integration row, never crash the server
- Hot-reload: when config is saved via API, the affected provider is re-instantiated without restarting

### Event Flow

```
MetricsCollector (5s) ──→ broadcastMetrics()  ──→ Grafana Cloud, Datadog
NodeManager (logs)    ──→ broadcastLogs()     ──→ Better Stack, Loki
NodeManager (status)  ──→ broadcastNotification() ──→ Slack, Discord, Telegram, Webhook
WatchdogManager       ──→ broadcastNotification() ──→ Slack, Discord, Telegram, Webhook
DiskMonitor           ──→ broadcastNotification() ──→ Slack, Discord, Telegram, Webhook
Uncaught errors       ──→ captureError()      ──→ Sentry
```

## Database

New `integrations` table:

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

## API Endpoints

All require admin role.

| Method | Route | Purpose |
|--------|-------|---------|
| GET | `/api/integrations` | List all providers with status (credentials redacted) |
| GET | `/api/integrations/:id` | Get single provider config (credentials redacted) |
| PUT | `/api/integrations/:id` | Save config & set enabled flag |
| POST | `/api/integrations/:id/test` | Run connection test, return success/error |
| DELETE | `/api/integrations/:id` | Clear config & disable |

Credential redaction: sensitive fields (API keys, tokens, DSNs) are masked as `"sk-...xxxx"` (last 4 chars) in GET responses. Full values are never returned after being saved.

## Frontend

### New Sidebar Item

"Integrations" with plug icon, placed between "Servers" and "Settings".

### Integrations Page

- Category tabs: All, Metrics, Logging, Uptime, Alerting, Errors
- Each provider rendered as a card from its `configSchema`
- Card shows: icon, name, one-line description, status indicator
- Status: green "Connected" (last test passed), yellow "Error" (with message), gray "Not configured"
- **Save & Test** button: validates fields, saves, runs test, shows result inline
- **Disable** button: keeps saved config, sets `enabled = 0`
- Auto-rendered form inputs from `configSchema`

### New Files

- `web/src/pages/Integrations.tsx`
- `web/src/components/IntegrationCard.tsx`
- `web/src/hooks/useIntegrations.ts`

### Modified Files

- `web/src/App.tsx` — add route and sidebar item

## Backend File Structure

```
src/integrations/
  IntegrationManager.ts
  types.ts
  registry.ts
  providers/
    metrics/
      GrafanaMetricsProvider.ts
      DatadogProvider.ts
    logging/
      BetterStackLoggingProvider.ts
      GrafanaLokiProvider.ts
    uptime/
      BetterStackUptimeProvider.ts
      UptimeRobotProvider.ts
    alerting/
      WebhookProvider.ts
      SlackProvider.ts
      DiscordProvider.ts
      TelegramProvider.ts
    errors/
      SentryProvider.ts
```

## Modified Existing Files

| File | Change |
|------|--------|
| `src/database/schema.ts` | Add `integrations` table |
| `src/server.ts` | Add `/api/integrations` routes, initialize IntegrationManager |
| `src/monitoring/MetricsCollector.ts` | Call `IntegrationManager.broadcastMetrics()` after collection |
| `src/core/NodeManager.ts` | Emit events to IntegrationManager on status changes and log events |
| `src/core/WatchdogManager.ts` | Emit events to IntegrationManager on restart/exhaustion |
| `src/utils/diskMonitor.ts` | Emit events to IntegrationManager on disk alerts |

## New Dependencies

| Package | Purpose |
|---------|---------|
| `@sentry/node` | Sentry error tracking SDK |
*(No additional deps for Grafana — uses OTLP JSON via plain `fetch()`)*

## Design Principles

- **Zero-impact when unconfigured** — No SaaS code runs unless tokens are set and enabled
- **Fire-and-forget** — Provider failures are silently logged, never block core functionality
- **Hot-reload** — Saving config re-instantiates just that provider, no server restart
- **Auto-render forms** — Frontend builds config forms from `configSchema`, no custom UI per provider
- **Credential safety** — Tokens never returned in full via API after being saved
