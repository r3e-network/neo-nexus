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
