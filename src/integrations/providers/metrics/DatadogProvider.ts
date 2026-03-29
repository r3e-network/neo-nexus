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
