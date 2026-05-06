import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.unmock("better-sqlite3");

import Database from "better-sqlite3";
import { IntegrationManager } from "../../src/integrations/IntegrationManager";
import { registryMap } from "../../src/integrations/registry";
import type { IntegrationEvent } from "../../src/integrations/types";

const waitForAsyncProviderWork = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("IntegrationManager", () => {
  let db: Database.Database;
  let manager: IntegrationManager;

  beforeEach(() => {
    db = new Database(":memory:");
    db.exec(`
      CREATE TABLE integrations (
        id TEXT PRIMARY KEY,
        category TEXT NOT NULL,
        config TEXT NOT NULL DEFAULT '{}',
        enabled INTEGER NOT NULL DEFAULT 0,
        last_test_at TEXT,
        last_error TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
      )
    `);
    manager = new IntegrationManager(db);
  });

  afterEach(() => {
    db.close();
    delete process.env.PORT;
    delete process.env.HOST;
    delete process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS;
  });

  it("redacts sensitive integration values in list and detail responses", () => {
    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("slack", "alerting", JSON.stringify({ webhookUrl: "https://hooks.slack.com/services/T000/B000/SECRET" }), 0);

    const listed = manager.listAll().find((integration) => integration.id === "slack");
    const detail = manager.getOne("slack");

    expect(listed?.configValues.webhookUrl).toBe("••••••••...CRET");
    expect(detail?.configValues.webhookUrl).toBe("••••••••...CRET");
    expect(JSON.stringify(listed)).not.toContain("SECRET");
    expect(JSON.stringify(detail)).not.toContain("SECRET");
  });

  it("preserves existing secrets when saving a redacted placeholder", () => {
    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("slack", "alerting", JSON.stringify({ webhookUrl: "https://hooks.slack.com/services/T000/B000/SECRET" }), 0);

    manager.saveConfig("slack", { webhookUrl: "••••••••...CRET" }, true);

    const row = db.prepare("SELECT config, enabled FROM integrations WHERE id = ?").get("slack") as { config: string; enabled: number };
    expect(JSON.parse(row.config)).toEqual({ webhookUrl: "https://hooks.slack.com/services/T000/B000/SECRET" });
    expect(row.enabled).toBe(1);
  });

  it("rejects redacted placeholders that do not correspond to a saved value", () => {
    expect(() => manager.saveConfig("slack", { webhookUrl: "••••••••...CRET" }, true))
      .toThrow(/before a value has been saved/i);

    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("slack", "alerting", JSON.stringify({ webhookUrl: "https://hooks.slack.com/services/T000/B000/SECRET" }), 0);

    expect(() => manager.saveConfig("slack", { webhookUrl: "••••••••...WRONG" }, true))
      .toThrow(/does not match the stored value/i);
  });

  it("rejects invalid URL fields before persistence", () => {
    expect(() => manager.saveConfig("slack", { webhookUrl: "not-a-url" }, true))
      .toThrow(/invalid url/i);

    expect(db.prepare("SELECT COUNT(*) as count FROM integrations").get()).toEqual({ count: 0 });
  });

  it("records missing required fields when an enabled provider cannot load", () => {
    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("uptimerobot", "uptime", JSON.stringify({ apiKey: "uptime-key" }), 1);

    manager = new IntegrationManager(db);

    const row = db.prepare("SELECT last_error FROM integrations WHERE id = ?").get("uptimerobot") as { last_error: string | null };
    expect(row.last_error).toMatch(/healthUrl/);
  });

  it("allows private integration URL fields only when explicitly enabled", () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS = "true";

    manager.saveConfig("webhook", { url: "http://127.0.0.1:8080/hook" }, true);

    const row = db.prepare("SELECT config, enabled FROM integrations WHERE id = ?").get("webhook") as { config: string; enabled: number };
    expect(JSON.parse(row.config)).toEqual({ url: "http://127.0.0.1:8080/hook" });
    expect(row.enabled).toBe(1);
  });

  it("rejects private or local integration URL fields before persistence", () => {
    expect(() => manager.saveConfig("webhook", { url: "http://127.0.0.1:8080/hook" }, true))
      .toThrow(/private or local/i);

    expect(() => manager.saveConfig("grafana_loki", {
      pushUrl: "http://169.254.169.254/latest",
      username: "123",
      apiKey: "key",
    }, true)).toThrow(/private or local/i);

    expect(() => manager.saveConfig("webhook", { url: "http://[::ffff:127.0.0.1]:8080/hook" }, true))
      .toThrow(/private or local/i);

    expect(() => manager.saveConfig("sentry", { dsn: "https://public@127.0.0.1:9000/1" }, true))
      .toThrow(/private or local/i);

    expect(db.prepare("SELECT COUNT(*) as count FROM integrations").get()).toEqual({ count: 0 });
  });

  it("requires Sentry DSNs to be complete HTTPS DSNs", () => {
    expect(() => manager.saveConfig("sentry", { dsn: "http://public@example.com/1" }, true))
      .toThrow(/https dsn/i);
    expect(() => manager.saveConfig("sentry", { dsn: "https://example.com/1" }, true))
      .toThrow(/public key/i);
    expect(() => manager.saveConfig("sentry", { dsn: "https://public@example.com" }, true))
      .toThrow(/project id/i);
  });

  it("suppresses duplicate notification events inside the cooldown window", async () => {
    const notify = vi.fn().mockResolvedValue(undefined);
    (manager as unknown as {
      notificationProviders: Map<string, { notify: typeof notify }>;
    }).notificationProviders.set("test", { notify });
    const baseEvent: IntegrationEvent = {
      type: "node.error",
      severity: "critical",
      title: "Node error",
      message: "Node entered error state",
      nodeId: "node-1",
      nodeName: "Node 1",
      timestamp: 1_000,
    };

    await manager.broadcastNotification(baseEvent);
    await manager.broadcastNotification({ ...baseEvent, timestamp: 2_000 });
    await manager.broadcastNotification({ ...baseEvent, nodeId: "node-2", timestamp: 3_000 });
    await manager.broadcastNotification({ ...baseEvent, timestamp: 1_000 + 5 * 60 * 1000 });

    expect(notify).toHaveBeenCalledTimes(3);
    expect(notify).toHaveBeenNthCalledWith(1, baseEvent);
    expect(notify).toHaveBeenNthCalledWith(2, expect.objectContaining({ nodeId: "node-2" }));
    expect(notify).toHaveBeenNthCalledWith(3, expect.objectContaining({ timestamp: 301_000 }));
  });

  it("registers and removes uptime monitors using the NeoNexus health endpoint", async () => {
    process.env.PORT = "8080";
    process.env.HOST = "0.0.0.0";
    const uptimeDefinition = registryMap.get("uptimerobot");
    if (!uptimeDefinition) throw new Error("uptimerobot definition missing");
    const originalCreateProvider = uptimeDefinition.createProvider;
    const registerMonitor = vi.fn().mockResolvedValue("monitor-1");
    const removeMonitor = vi.fn().mockResolvedValue(undefined);
    uptimeDefinition.createProvider = vi.fn(() => ({
      name: "Fake Uptime",
      registerMonitor,
      removeMonitor,
      testConnection: vi.fn().mockResolvedValue(true),
    }));

    try {
      db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
        .run("uptimerobot", "uptime", JSON.stringify({
          apiKey: "uptime-key",
          healthUrl: "https://nexus.example.com/api/health",
        }), 1);

      manager = new IntegrationManager(db);
      await waitForAsyncProviderWork();

      expect(registerMonitor).toHaveBeenCalledWith("https://nexus.example.com/api/health", "NeoNexus (uptimerobot)");
      const registered = db.prepare("SELECT config FROM integrations WHERE id = ?").get("uptimerobot") as { config: string };
      expect(JSON.parse(registered.config)._monitorId).toBe("monitor-1");

      manager.deleteConfig("uptimerobot");
      await waitForAsyncProviderWork();

      expect(removeMonitor).toHaveBeenCalledWith("monitor-1");
      expect(db.prepare("SELECT COUNT(*) as count FROM integrations WHERE id = ?").get("uptimerobot")).toEqual({ count: 0 });
    } finally {
      uptimeDefinition.createProvider = originalCreateProvider;
    }
  });

  it("records provider errors while continuing metrics broadcasts", async () => {
    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("grafana_metrics", "metrics", JSON.stringify({ remoteWriteUrl: "https://grafana.example.com", username: "123", apiKey: "key" }), 0);
    db.prepare("INSERT INTO integrations (id, category, config, enabled) VALUES (?, ?, ?, ?)")
      .run("datadog", "metrics", JSON.stringify({ apiKey: "key", site: "datadoghq.com" }), 0);
    const failingProvider = {
      name: "Failing metrics",
      pushMetrics: vi.fn().mockRejectedValue(new Error("metrics down")),
      testConnection: vi.fn().mockResolvedValue(true),
    };
    const healthyProvider = {
      name: "Healthy metrics",
      pushMetrics: vi.fn().mockResolvedValue(undefined),
      testConnection: vi.fn().mockResolvedValue(true),
    };
    (manager as unknown as {
      metricsProviders: Map<string, typeof failingProvider>;
    }).metricsProviders.set("grafana_metrics", failingProvider);
    (manager as unknown as {
      metricsProviders: Map<string, typeof healthyProvider>;
    }).metricsProviders.set("datadog", healthyProvider);

    await manager.broadcastMetrics(
      {
        cpu: { usage: 1, cores: 2 },
        memory: { total: 2, used: 1, free: 1, percentage: 50 },
        disk: { total: 2, used: 1, free: 1, percentage: 50 },
        network: { rx: 0, tx: 0 },
      },
      [],
    );

    expect(failingProvider.pushMetrics).toHaveBeenCalled();
    expect(healthyProvider.pushMetrics).toHaveBeenCalled();
    const failedRow = db.prepare("SELECT last_error FROM integrations WHERE id = ?").get("grafana_metrics") as { last_error: string | null };
    const healthyRow = db.prepare("SELECT last_error FROM integrations WHERE id = ?").get("datadog") as { last_error: string | null };
    expect(failedRow.last_error).toBe("metrics down");
    expect(healthyRow.last_error).toBeNull();
  });
});
