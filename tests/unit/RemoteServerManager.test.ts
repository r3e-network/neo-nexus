import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { beforeEach, describe, expect, it, vi } from "vitest";

function createMockDb() {
  const servers: any[] = [];

  return {
    prepare: vi.fn((sql: string) => {
      if (sql.includes("SELECT * FROM remote_servers WHERE id = ?")) {
        return {
          get: vi.fn((id: string) => servers.find((server) => server.id === id)),
        };
      }

      if (sql.includes("SELECT * FROM remote_servers ORDER BY created_at")) {
        return {
          all: vi.fn(() => [...servers]),
        };
      }

      if (sql.includes("INSERT INTO remote_servers")) {
        return {
          run: vi.fn((...args: any[]) => {
            servers.push({
              id: args[0],
              name: args[1],
              base_url: args[2],
              description: args[3],
              enabled: args[4],
              created_at: args[5],
              updated_at: args[6],
            });
          }),
        };
      }

      if (sql.includes("UPDATE remote_servers")) {
        return {
          run: vi.fn((...args: any[]) => {
            const id = args[args.length - 1];
            const server = servers.find((entry) => entry.id === id);
            if (server) {
              if (sql.includes("name = ?")) server.name = args[0];
              if (sql.includes("base_url = ?")) server.base_url = sql.includes("name = ?") ? args[1] : args[0];
              if (sql.includes("description = ?")) server.description = args[sql.includes("enabled = ?") ? 2 : 1];
              if (sql.includes("enabled = ?")) server.enabled = args[sql.includes("description = ?") ? 3 : 2];
              server.updated_at = args[args.length - 2];
            }
          }),
        };
      }

      if (sql.includes("DELETE FROM remote_servers WHERE id = ?")) {
        return {
          run: vi.fn((id: string) => {
            const index = servers.findIndex((server) => server.id === id);
            if (index > -1) {
              servers.splice(index, 1);
            }
          }),
        };
      }

      return {
        get: vi.fn(),
        all: vi.fn(() => []),
        run: vi.fn(),
      };
    }),
  };
}

async function startRemoteSummaryServer() {
  const seenHosts: string[] = [];
  const server = createServer((req, res) => {
    seenHosts.push(req.headers.host ?? "");
    res.setHeader("Content-Type", "application/json");
    if (req.url === "/api/public/status") {
      res.end(JSON.stringify({
        status: {
          totalNodes: 2,
          runningNodes: 1,
          syncingNodes: 0,
          errorNodes: 0,
          totalBlocks: 100,
          totalPeers: 8,
          timestamp: 123,
        },
      }));
      return;
    }
    if (req.url === "/api/public/metrics/system") {
      res.end(JSON.stringify({
        metrics: {
          cpu: { usage: 40, cores: 8 },
          memory: { percentage: 50, used: 1, total: 2 },
          disk: { percentage: 60, used: 3, total: 4 },
        },
      }));
      return;
    }
    if (req.url === "/api/public/nodes") {
      res.end(JSON.stringify({
        nodes: [{ id: "node-1", name: "Remote Node", status: "running", metrics: { blockHeight: 100 } }],
      }));
      return;
    }
    res.statusCode = 404;
    res.end(JSON.stringify({ error: "not found" }));
  });
  const port = await listen(server);
  return {
    port,
    seenHosts,
    close: () => closeServer(server),
  };
}

async function getClosedPort(): Promise<number> {
  const server = createServer();
  const port = await listen(server);
  await closeServer(server);
  return port;
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

describe("RemoteServerManager", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    delete process.env.NEONEXUS_ALLOW_PRIVATE_REMOTE_SERVERS;
  });

  it("creates and lists normalized server profiles", async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);

    const created = manager.createServer({
      name: "Tokyo Node Manager",
      baseUrl: "https://tokyo.example.com/",
      description: "primary remote host",
    });

    expect(created.baseUrl).toBe("https://tokyo.example.com");
    expect(manager.listServers()).toEqual([created]);
  });

  it("rejects unsupported remote server URL protocols", async () => {
    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);

    expect(() => manager.createServer({
      name: "File Target",
      baseUrl: "file:///etc/passwd",
    })).toThrow(/http or https/i);
  });

  it("blocks private and local remote server targets by default", async () => {
    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);

    expect(() => manager.createServer({
      name: "Loopback",
      baseUrl: "http://127.0.0.1:8080",
    })).toThrow(/private or local/i);

    expect(() => manager.createServer({
      name: "Metadata",
      baseUrl: "http://169.254.169.254",
    })).toThrow(/private or local/i);

    expect(() => manager.createServer({
      name: "IPv4 mapped loopback",
      baseUrl: "http://[::ffff:127.0.0.1]:8080",
    })).toThrow(/private or local/i);
  });

  it("allows private remote server targets only when explicitly enabled", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_REMOTE_SERVERS = "true";
    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);

    const created = manager.createServer({
      name: "LAN Manager",
      baseUrl: "http://192.168.1.25:8080/",
    });

    expect(created.baseUrl).toBe("http://192.168.1.25:8080");
  });

  it("fetches remote status summaries for configured servers", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_REMOTE_SERVERS = "true";
    const server = await startRemoteSummaryServer();
    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never, {
      resolveHostname: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    });
    const profile = manager.createServer({
      name: "Tokyo Node Manager",
      baseUrl: `http://remote.example.test:${server.port}`,
    });

    try {
      const summary = await manager.getServerSummary(profile.id);

      expect(summary.reachable).toBe(true);
      expect(summary.profile.name).toBe("Tokyo Node Manager");
      expect(summary.status?.totalNodes).toBe(2);
      expect(summary.nodes).toHaveLength(1);
      expect(server.seenHosts).toContain(`remote.example.test:${server.port}`);
    } finally {
      await server.close();
    }
  });

  it("marks DNS-private remote server targets unreachable before fetching", async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never, {
      resolveHostname: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    });
    const profile = manager.createServer({
      name: "DNS Private",
      baseUrl: "https://dns-private.example.com",
    });

    const summary = await manager.getServerSummary(profile.id);

    expect(summary.reachable).toBe(false);
    expect(summary.error).toMatch(/private or local/i);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("marks servers unreachable when public endpoints fail", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_REMOTE_SERVERS = "true";
    const port = await getClosedPort();
    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);
    const profile = manager.createServer({
      name: "Offline Manager",
      baseUrl: `http://127.0.0.1:${port}`,
    });

    const summary = await manager.getServerSummary(profile.id);

    expect(summary.reachable).toBe(false);
    expect(summary.error).toContain("ECONNREFUSED");
  });
});
