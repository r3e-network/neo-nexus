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

describe("RemoteServerManager", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
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

  it("fetches remote status summaries for configured servers", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.endsWith("/api/public/status")) {
          return {
            ok: true,
            json: async () => ({
              status: {
                totalNodes: 2,
                runningNodes: 1,
                syncingNodes: 0,
                errorNodes: 0,
                totalBlocks: 100,
                totalPeers: 8,
                timestamp: 123,
              },
            }),
          } as Response;
        }
        if (url.endsWith("/api/public/metrics/system")) {
          return {
            ok: true,
            json: async () => ({
              metrics: {
                cpu: { usage: 40, cores: 8 },
                memory: { percentage: 50, used: 1, total: 2 },
                disk: { percentage: 60, used: 3, total: 4 },
              },
            }),
          } as Response;
        }
        if (url.endsWith("/api/public/nodes")) {
          return {
            ok: true,
            json: async () => ({
              nodes: [{ id: "node-1", name: "Remote Node", status: "running", metrics: { blockHeight: 100 } }],
            }),
          } as Response;
        }
        throw new Error(`Unexpected URL ${url}`);
      }),
    );

    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);
    const profile = manager.createServer({
      name: "Tokyo Node Manager",
      baseUrl: "https://tokyo.example.com",
    });

    const summary = await manager.getServerSummary(profile.id);

    expect(summary.reachable).toBe(true);
    expect(summary.profile.name).toBe("Tokyo Node Manager");
    expect(summary.status?.totalNodes).toBe(2);
    expect(summary.nodes).toHaveLength(1);
  });

  it("marks servers unreachable when public endpoints fail", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => {
      throw new Error("connect ECONNREFUSED");
    }));

    const { RemoteServerManager } = await import("../../src/core/RemoteServerManager");
    const manager = new RemoteServerManager(createMockDb() as never);
    const profile = manager.createServer({
      name: "Offline Manager",
      baseUrl: "https://offline.example.com",
    });

    const summary = await manager.getServerSummary(profile.id);

    expect(summary.reachable).toBe(false);
    expect(summary.error).toContain("ECONNREFUSED");
  });
});
