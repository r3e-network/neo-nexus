// src/integrations/providers/metrics/GrafanaMetricsProvider.ts
import type { MetricsProvider, NodeMetricsWithContext, ConfigField } from '../../types';
import type { SystemMetrics } from '../../../types/index';
import { safeIntegrationFetch } from '../../safeFetch';
import { pushTimeseries } from 'prometheus-remote-write';

export const grafanaMetricsSchema: ConfigField[] = [
  { key: 'remoteWriteUrl', label: 'Remote Write URL', type: 'url', placeholder: 'https://prometheus-prod-01-prod-us-east-0.grafana.net/api/prom/push', required: true },
  { key: 'username', label: 'Username / Instance ID', type: 'text', placeholder: '123456', required: true },
  { key: 'apiKey', label: 'API Key', type: 'password', placeholder: 'glc_...', required: true },
];

interface PrometheusTimeseries {
  labels: { __name__: string } & Record<string, string>;
  samples: Array<{ value: number; timestamp: number }>;
}

type RemoteWriteFetch = NonNullable<NonNullable<Parameters<typeof pushTimeseries>[1]>['fetch']>;

export class GrafanaMetricsProvider implements MetricsProvider {
  readonly name = 'Grafana Cloud';

  constructor(private config: { remoteWriteUrl: string; username: string; apiKey: string }) {}

  async pushMetrics(system: SystemMetrics, nodes: NodeMetricsWithContext[]): Promise<void> {
    await this.pushRemoteWrite(buildGrafanaTimeseries(system, nodes, Date.now()));
  }

  async testConnection(): Promise<boolean> {
    try {
      await this.pushRemoteWrite([{
        labels: { __name__: 'neonexus_connection_test', job: 'neonexus' },
        samples: [{ value: 1, timestamp: Date.now() }],
      }]);
      return true;
    } catch {
      return false;
    }
  }

  private async pushRemoteWrite(timeseries: PrometheusTimeseries[]): Promise<void> {
    const result = await pushTimeseries(timeseries, {
      url: this.config.remoteWriteUrl,
      auth: {
        username: this.config.username,
        password: this.config.apiKey,
      },
      headers: {
        'Content-Encoding': 'snappy',
        'X-Prometheus-Remote-Write-Version': '0.1.0',
      },
      timeout: 10_000,
      fetch: remoteWriteFetch,
    });

    if (result.status !== 200 && result.status !== 204) {
      throw new Error(`Grafana remote_write failed: ${result.status} ${result.statusText}${result.errorMessage ? ` ${result.errorMessage}` : ''}`);
    }
  }
}

export function buildGrafanaTimeseries(
  system: SystemMetrics,
  nodes: NodeMetricsWithContext[],
  timestamp: number,
): PrometheusTimeseries[] {
  const series: PrometheusTimeseries[] = [];

  const addSeries = (name: string, value: number, labels: Record<string, string> = {}) => {
    if (!Number.isFinite(value)) return;
    series.push({
      labels: { __name__: name, job: 'neonexus', ...labels },
      samples: [{ value, timestamp }],
    });
  };

  addSeries('neonexus_cpu_usage', system.cpu.usage);
  addSeries('neonexus_memory_usage_percent', system.memory.percentage);
  addSeries('neonexus_memory_used_bytes', system.memory.used);
  addSeries('neonexus_disk_usage_percent', system.disk.percentage);
  addSeries('neonexus_disk_used_bytes', system.disk.used);
  addSeries('neonexus_network_rx_bytes', system.network.rx);
  addSeries('neonexus_network_tx_bytes', system.network.tx);

  for (const node of nodes) {
    const labels = {
      node_id: node.nodeId,
      node_name: node.nodeName,
      node_type: node.nodeType,
      network: node.network,
    };
    addSeries('neonexus_node_block_height', node.metrics.blockHeight, labels);
    addSeries('neonexus_node_peers', node.metrics.connectedPeers, labels);
    addSeries('neonexus_node_sync_progress', node.metrics.syncProgress, labels);
    addSeries('neonexus_node_memory_bytes', node.metrics.memoryUsage, labels);
    addSeries('neonexus_node_cpu', node.metrics.cpuUsage, labels);
  }

  return series;
}

const remoteWriteFetch: RemoteWriteFetch = async (url, init) => {
  const response = await safeIntegrationFetch(url, {
    method: init?.method,
    headers: init?.headers,
    body: init?.body ? Buffer.from(init.body as ArrayBuffer) : undefined,
    signal: AbortSignal.timeout(init?.timeout ?? 10_000),
  });

  return {
    status: response.status,
    statusText: response.statusText,
    text: () => response.text(),
  };
};
