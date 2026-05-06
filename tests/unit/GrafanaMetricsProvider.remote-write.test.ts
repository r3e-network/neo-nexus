import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { afterEach, describe, expect, it } from "vitest";
import { GrafanaMetricsProvider } from "../../src/integrations/providers/metrics/GrafanaMetricsProvider";

function listen(server: Server): Promise<number> {
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => {
      resolve((server.address() as AddressInfo).port);
    });
  });
}

function closeServer(server: Server): Promise<void> {
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });
}

describe("GrafanaMetricsProvider remote_write transport", () => {
  afterEach(() => {
    delete process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS;
  });

  it("sends an actual snappy protobuf remote_write request through the guarded fetcher", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS = "true";
    let bodyLength = 0;
    let headers: Record<string, string | string[] | undefined> = {};
    const server = createServer((req, res) => {
      headers = req.headers;
      req.on("data", (chunk: Buffer) => {
        bodyLength += chunk.length;
      });
      req.on("end", () => {
        res.statusCode = 204;
        res.end();
      });
    });
    const port = await listen(server);
    const provider = new GrafanaMetricsProvider({
      remoteWriteUrl: `http://127.0.0.1:${port}/api/prom/push`,
      username: "instance-1",
      apiKey: "grafana-key",
    });

    try {
      await provider.pushMetrics({
        cpu: { usage: 1, cores: 4 },
        memory: { total: 100, used: 50, free: 50, percentage: 50 },
        disk: { total: 100, used: 20, free: 80, percentage: 20 },
        network: { rx: 3, tx: 4 },
      }, []);

      expect(headers["content-type"]).toBe("application/x-protobuf");
      expect(headers["content-encoding"]).toBe("snappy");
      expect(headers["x-prometheus-remote-write-version"]).toBe("0.1.0");
      expect(headers.authorization).toMatch(/^Basic /);
      expect(bodyLength).toBeGreaterThan(0);
    } finally {
      await closeServer(server);
    }
  });
});
