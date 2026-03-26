import { randomUUID } from "node:crypto";
import type Database from "better-sqlite3";
import type {
  CreateRemoteServerRequest,
  RemoteServerNodeSummary,
  RemoteServerProfile,
  RemoteServerStatusSummary,
  RemoteServerSummary,
  RemoteServerSystemMetrics,
  UpdateRemoteServerRequest,
} from "../types";

export class RemoteServerManager {
  constructor(private db: Database.Database) {}

  createServer(request: CreateRemoteServerRequest): RemoteServerProfile {
    const id = randomUUID();
    const now = Date.now();
    const profile = {
      id,
      name: request.name.trim(),
      baseUrl: this.normalizeBaseUrl(request.baseUrl),
      description: request.description?.trim() || undefined,
      enabled: request.enabled ?? true,
      createdAt: now,
      updatedAt: now,
    };

    this.db
      .prepare(
        `INSERT INTO remote_servers (id, name, base_url, description, enabled, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(
        profile.id,
        profile.name,
        profile.baseUrl,
        profile.description ?? null,
        profile.enabled ? 1 : 0,
        profile.createdAt,
        profile.updatedAt,
      );

    return profile;
  }

  listServers(): RemoteServerProfile[] {
    const rows = this.db.prepare("SELECT * FROM remote_servers ORDER BY created_at").all() as Array<{
      id: string;
      name: string;
      base_url: string;
      description: string | null;
      enabled: number;
      created_at: number;
      updated_at: number;
    }>;

    return rows.map((row) => this.mapRow(row));
  }

  getServer(id: string): RemoteServerProfile | null {
    const row = this.db.prepare("SELECT * FROM remote_servers WHERE id = ?").get(id) as
      | {
          id: string;
          name: string;
          base_url: string;
          description: string | null;
          enabled: number;
          created_at: number;
          updated_at: number;
        }
      | undefined;

    return row ? this.mapRow(row) : null;
  }

  updateServer(id: string, request: UpdateRemoteServerRequest): RemoteServerProfile {
    const current = this.getServer(id);
    if (!current) {
      throw new Error(`Remote server ${id} not found`);
    }

    const next = {
      ...current,
      ...(request.name !== undefined ? { name: request.name.trim() } : {}),
      ...(request.baseUrl !== undefined ? { baseUrl: this.normalizeBaseUrl(request.baseUrl) } : {}),
      ...(request.description !== undefined ? { description: request.description.trim() || undefined } : {}),
      ...(request.enabled !== undefined ? { enabled: request.enabled } : {}),
      updatedAt: Date.now(),
    };

    this.db
      .prepare(
        `UPDATE remote_servers
         SET name = ?, base_url = ?, description = ?, enabled = ?, updated_at = ?
         WHERE id = ?`,
      )
      .run(next.name, next.baseUrl, next.description ?? null, next.enabled ? 1 : 0, next.updatedAt, id);

    return next;
  }

  deleteServer(id: string): void {
    this.db.prepare("DELETE FROM remote_servers WHERE id = ?").run(id);
  }

  async listServersWithStatus(): Promise<RemoteServerSummary[]> {
    const profiles = this.listServers();
    return Promise.all(profiles.map((profile) => this.buildSummary(profile)));
  }

  async getServerSummary(id: string): Promise<RemoteServerSummary> {
    const profile = this.getServer(id);
    if (!profile) {
      throw new Error(`Remote server ${id} not found`);
    }

    return this.buildSummary(profile);
  }

  private async buildSummary(profile: RemoteServerProfile): Promise<RemoteServerSummary> {
    if (!profile.enabled) {
      return {
        profile,
        reachable: false,
        error: "Server profile is disabled",
      };
    }

    try {
      const [status, nodes, systemMetrics] = await Promise.all([
        this.fetchJson<{ status: RemoteServerStatusSummary }>(profile.baseUrl, "/api/public/status"),
        this.fetchJson<{ nodes: RemoteServerNodeSummary[] }>(profile.baseUrl, "/api/public/nodes"),
        this.fetchJson<{ metrics: RemoteServerSystemMetrics }>(profile.baseUrl, "/api/public/metrics/system"),
      ]);

      return {
        profile,
        reachable: true,
        status: status.status,
        nodes: nodes.nodes,
        systemMetrics: systemMetrics.metrics,
      };
    } catch (error) {
      return {
        profile,
        reachable: false,
        error: error instanceof Error ? error.message : "Failed to reach remote server",
      };
    }
  }

  private async fetchJson<T>(baseUrl: string, path: string): Promise<T> {
    const response = await fetch(`${baseUrl}${path}`, {
      signal: AbortSignal.timeout(5000),
    });
    if (!response.ok) {
      throw new Error(`Remote request failed with status ${response.status}`);
    }
    return (await response.json()) as T;
  }

  private normalizeBaseUrl(rawUrl: string): string {
    const url = new URL(rawUrl.trim());
    const path = url.pathname === "/" ? "" : url.pathname.replace(/\/$/, "");
    return `${url.origin}${path}`;
  }

  private mapRow(row: {
    id: string;
    name: string;
    base_url: string;
    description: string | null;
    enabled: number;
    created_at: number;
    updated_at: number;
  }): RemoteServerProfile {
    return {
      id: row.id,
      name: row.name,
      baseUrl: row.base_url,
      description: row.description ?? undefined,
      enabled: row.enabled === 1,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }
}
