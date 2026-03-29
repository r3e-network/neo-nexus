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
