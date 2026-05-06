import { beforeEach, describe, expect, it, vi } from "vitest";

const pushTimeseriesMock = vi.hoisted(() => vi.fn());

vi.mock("prometheus-remote-write", () => ({
  pushTimeseries: pushTimeseriesMock,
}));

import { buildGrafanaTimeseries, GrafanaMetricsProvider } from "../../src/integrations/providers/metrics/GrafanaMetricsProvider";
import type { NodeMetricsWithContext } from "../../src/integrations/types";
import type { SystemMetrics } from "../../src/types/index";

const systemMetrics: SystemMetrics = {
  cpu: { usage: 11, cores: 8 },
  memory: { total: 1000, used: 250, free: 750, percentage: 25 },
  disk: { total: 2000, used: 1000, free: 1000, percentage: 50 },
  network: { rx: 123, tx: 456 },
};

const nodeMetrics: NodeMetricsWithContext[] = [{
  nodeId: "node-1",
  nodeName: "Consensus Node",
  nodeType: "neo-cli",
  network: "testnet",
  metrics: {
    blockHeight: 100,
    headerHeight: 120,
    connectedPeers: 7,
    unconnectedPeers: 0,
    syncProgress: 83.3,
    memoryUsage: 512,
    cpuUsage: 3.5,
    lastUpdate: Date.now(),
  },
}];

describe("GrafanaMetricsProvider", () => {
  beforeEach(() => {
    pushTimeseriesMock.mockReset();
  });

  it("encodes metrics with Prometheus remote_write instead of plain text exposition", async () => {
    pushTimeseriesMock.mockResolvedValue({ status: 204, statusText: "No Content" });
    const provider = new GrafanaMetricsProvider({
      remoteWriteUrl: "https://prometheus-prod.example.grafana.net/api/prom/push",
      username: "instance-1",
      apiKey: "grafana-key",
    });

    await provider.pushMetrics(systemMetrics, nodeMetrics);

    expect(pushTimeseriesMock).toHaveBeenCalledTimes(1);
    const [series, options] = pushTimeseriesMock.mock.calls[0];
    expect(series).toEqual(expect.arrayContaining([
      expect.objectContaining({
        labels: expect.objectContaining({ __name__: "neonexus_cpu_usage", job: "neonexus" }),
      }),
      expect.objectContaining({
        labels: expect.objectContaining({
          __name__: "neonexus_node_block_height",
          node_id: "node-1",
          node_name: "Consensus Node",
          node_type: "neo-cli",
          network: "testnet",
        }),
        samples: [{ value: 100, timestamp: expect.any(Number) }],
      }),
    ]));
    expect(options).toMatchObject({
      url: "https://prometheus-prod.example.grafana.net/api/prom/push",
      auth: { username: "instance-1", password: "grafana-key" },
      headers: {
        "Content-Encoding": "snappy",
        "X-Prometheus-Remote-Write-Version": "0.1.0",
      },
      timeout: 10_000,
    });
    expect(typeof options.fetch).toBe("function");
  });

  it("reports failed remote_write responses", async () => {
    pushTimeseriesMock.mockResolvedValue({ status: 400, statusText: "Bad Request", errorMessage: "bad sample" });
    const provider = new GrafanaMetricsProvider({
      remoteWriteUrl: "https://prometheus-prod.example.grafana.net/api/prom/push",
      username: "instance-1",
      apiKey: "grafana-key",
    });

    await expect(provider.pushMetrics(systemMetrics, [])).rejects.toThrow(/remote_write failed: 400 Bad Request bad sample/);
  });

  it("skips non-finite samples before sending", () => {
    const series = buildGrafanaTimeseries({
      ...systemMetrics,
      network: { rx: Number.NaN, tx: 1 },
    }, [], 123);

    expect(series.map((entry) => entry.labels.__name__)).not.toContain("neonexus_network_rx_bytes");
    expect(series.map((entry) => entry.labels.__name__)).toContain("neonexus_network_tx_bytes");
  });
});
