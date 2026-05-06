import { createServer, type Server } from "node:http";
import http from "node:http";
import https from "node:https";
import type { AddressInfo } from "node:net";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WebhookProvider } from "../../src/integrations/providers/alerting/WebhookProvider";

async function startWebhookServer() {
  const seenHosts: string[] = [];
  const seenPaths: string[] = [];
  const server = createServer((req, res) => {
    seenHosts.push(req.headers.host ?? "");
    seenPaths.push(req.url ?? "");
    res.statusCode = 204;
    res.end();
  });
  const port = await listen(server);
  return {
    port,
    seenHosts,
    seenPaths,
    close: () => closeServer(server),
  };
}

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

describe("integration outbound target protection", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.doUnmock("node:dns/promises");
    vi.resetModules();
    delete process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS;
  });

  it("blocks notification providers from sending to private or local targets", async () => {
    const httpRequestSpy = vi.spyOn(http, "request");
    const httpsRequestSpy = vi.spyOn(https, "request");
    for (const url of [
      "http://127.0.0.1:8080/hook",
      "http://[::ffff:127.0.0.1]:8080/hook",
      "http://[::ffff:169.254.169.254]:8080/hook",
      "http://[fe93::1]:8080/hook",
      "http://[feb0::1]:8080/hook",
    ]) {
      const provider = new WebhookProvider({ url });

      await expect(provider.notify({
        type: "node.error",
        severity: "critical",
        title: "Node error",
        message: "Node entered error state",
        timestamp: Date.now(),
      })).rejects.toThrow(/private or local/i);
    }

    expect(httpRequestSpy).not.toHaveBeenCalled();
    expect(httpsRequestSpy).not.toHaveBeenCalled();
  });

  it("blocks notification providers when DNS resolves a public hostname to a private target", async () => {
    vi.doMock("node:dns/promises", () => ({
      lookup: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    }));
    const httpsRequestSpy = vi.spyOn(https, "request");
    const { WebhookProvider: GuardedWebhookProvider } = await import(
      "../../src/integrations/providers/alerting/WebhookProvider"
    );
    const provider = new GuardedWebhookProvider({ url: "https://hooks.example.com/hook" });

    await expect(provider.notify({
      type: "node.error",
      severity: "critical",
      title: "Node error",
      message: "Node entered error state",
      timestamp: Date.now(),
    })).rejects.toThrow(/private or local/i);

    expect(httpsRequestSpy).not.toHaveBeenCalled();
  });

  it("allows private notification targets only when explicitly enabled", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS = "true";
    const server = await startWebhookServer();
    const provider = new WebhookProvider({ url: `http://127.0.0.1:${server.port}/hook` });

    try {
      await provider.notify({
        type: "node.error",
        severity: "critical",
        title: "Node error",
        message: "Node entered error state",
        timestamp: Date.now(),
      });

      expect(server.seenPaths).toEqual(["/hook"]);
    } finally {
      await server.close();
    }
  });

  it("connects to the already-resolved address for allowed private integration targets", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS = "true";
    vi.resetModules();
    vi.doMock("node:dns/promises", () => ({
      lookup: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    }));
    const server = await startWebhookServer();
    const { WebhookProvider: GuardedWebhookProvider } = await import(
      "../../src/integrations/providers/alerting/WebhookProvider"
    );
    const provider = new GuardedWebhookProvider({ url: `http://hooks.example.test:${server.port}/hook` });

    try {
      await provider.notify({
        type: "node.error",
        severity: "critical",
        title: "Node error",
        message: "Node entered error state",
        timestamp: Date.now(),
      });

      expect(server.seenHosts).toEqual([`hooks.example.test:${server.port}`]);
    } finally {
      await server.close();
    }
  });

  it("overrides caller-supplied host headers with the pinned outbound target", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS = "true";
    vi.resetModules();
    vi.doMock("node:dns/promises", () => ({
      lookup: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    }));
    const server = await startWebhookServer();
    const { safeIntegrationFetch } = await import("../../src/integrations/safeFetch");

    try {
      await safeIntegrationFetch(`http://hooks.example.test:${server.port}/hook`, {
        headers: { host: "evil.example.test" },
      });

      expect(server.seenHosts).toEqual([`hooks.example.test:${server.port}`]);
    } finally {
      await server.close();
    }
  });
});
