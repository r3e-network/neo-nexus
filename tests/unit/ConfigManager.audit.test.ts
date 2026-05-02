import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";
import type { NodeConfig } from "../../src/types";

vi.mock("../../src/core/DownloadManager", () => ({
  DownloadManager: {
    getNodeBinaryPath: vi.fn(() => "/tmp/neo-cli"),
  },
}));

describe("ConfigManager audit", () => {
  const tempDirs: string[] = [];

  afterEach(() => {
    for (const dir of tempDirs.splice(0)) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("audits an imported neo-cli config file path instead of assuming base/config.json", async () => {
    const base = mkdtempSync(join(tmpdir(), "neonexus-audit-"));
    tempDirs.push(base);
    const importedConfigPath = join(base, "config.testnet.json");
    writeFileSync(importedConfigPath, "{ invalid json", "utf-8");

    const node: NodeConfig = {
      id: "node-1",
      name: "Imported neo-cli",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 20332, p2p: 20333 },
      paths: {
        base,
        config: importedConfigPath,
        data: join(base, "Chain"),
        logs: join(base, "Logs"),
      },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    };

    const result = await ConfigManager.auditNodeConfig(node);

    expect(result.issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          path: "config.testnet.json",
          severity: "error",
          message: expect.stringContaining("invalid JSON"),
        }),
      ]),
    );
    expect(result.issues).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          message: "Node config.json does not exist on disk",
        }),
      ]),
    );
  });

  it("writes only the detected imported neo-cli config file and does not create base config.json", async () => {
    const base = mkdtempSync(join(tmpdir(), "neonexus-write-imported-"));
    tempDirs.push(base);
    const importedConfigPath = join(base, "config.testnet.json");
    const baseConfigPath = join(base, "config.json");

    const node: NodeConfig = {
      id: "node-imported",
      name: "Imported neo-cli",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 20332, p2p: 20333 },
      paths: {
        base,
        config: importedConfigPath,
        data: join(base, "Chain"),
        logs: join(base, "Logs"),
      },
      settings: {
        import: {
          imported: true,
          existingPath: base,
          importType: "path",
          ownershipMode: "managed-config",
        },
      },
      createdAt: 1,
      updatedAt: 1,
    };

    await ConfigManager.writeNodeConfig(node);

    expect(existsSync(importedConfigPath)).toBe(true);
    expect(readFileSync(importedConfigPath, "utf-8")).toContain("ApplicationConfiguration");
    expect(existsSync(baseConfigPath)).toBe(false);
  });

  it("also treats legacy external neo-cli records without import metadata as non-managed writes", async () => {
    const base = mkdtempSync(join(tmpdir(), "neonexus-write-legacy-imported-"));
    tempDirs.push(base);
    const importedConfigPath = join(base, "config.mainnet.json");
    const baseConfigPath = join(base, "config.json");

    const node: NodeConfig = {
      id: "node-legacy",
      name: "Legacy imported neo-cli",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 10332, p2p: 10333 },
      paths: {
        base,
        config: importedConfigPath,
        data: join(base, "Chain"),
        logs: join(base, "Logs"),
      },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    };

    await ConfigManager.writeNodeConfig(node);

    expect(existsSync(importedConfigPath)).toBe(true);
    expect(existsSync(baseConfigPath)).toBe(false);
  });

  it("does not echo raw on-disk config values in audit messages", async () => {
    const base = mkdtempSync(join(tmpdir(), "neonexus-audit-redaction-"));
    tempDirs.push(base);
    const configPath = join(base, "config.json");
    const node: NodeConfig = {
      id: "node-redact",
      name: "Sensitive config",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 20332, p2p: 20333 },
      paths: {
        base,
        config: configPath,
        data: join(base, "Chain"),
        logs: join(base, "Logs"),
      },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    };
    const expectedConfig = await (ConfigManager as unknown as {
      generateNeoCliConfig: (node: NodeConfig, plugins: string[]) => Promise<Record<string, unknown>>;
    }).generateNeoCliConfig(node, []);
    const applicationConfig = expectedConfig.ApplicationConfiguration as Record<string, unknown>;
    const p2pConfig = applicationConfig.P2P as Record<string, unknown>;
    writeFileSync(configPath, JSON.stringify({
      ...expectedConfig,
      ApplicationConfiguration: {
        ...applicationConfig,
        P2P: {
          ...p2pConfig,
          Port: 65535,
          SecretPassword: "raw-rpc-secret",
        },
      },
    }), "utf-8");

    const result = await ConfigManager.auditNodeConfig(node);
    const messages = result.issues.map((issue) => issue.message).join("\n");

    expect(messages).not.toContain("raw-rpc-secret");
    expect(messages).not.toContain("65535");
    expect(messages).not.toContain("20332");
    expect(result.issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          path: "ApplicationConfiguration.P2P.Port",
          message: expect.stringContaining("Value differs from generated default"),
        }),
        expect.objectContaining({
          path: "ApplicationConfiguration.P2P.SecretPassword",
          message: "Extra key \"SecretPassword\" found in on-disk config",
        }),
      ]),
    );
  });
});
