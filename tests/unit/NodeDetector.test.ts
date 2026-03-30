import { afterEach, describe, expect, it } from "vitest";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import YAML from "js-yaml";
import { NodeDetector } from "../../src/core/NodeDetector";
import { ConfigManager } from "../../src/core/ConfigManager";
import type { NodeConfig } from "../../src/types";

const tempDirs: string[] = [];

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});

function createTempDir(): string {
  const dir = mkdtempSync(join(tmpdir(), "neo-nexus-node-detector-"));
  tempDirs.push(dir);
  return dir;
}

function createNeoGoNodeConfig(basePath: string): NodeConfig {
  return {
    id: "node-1",
    name: "Imported Neo Go",
    type: "neo-go",
    network: "mainnet",
    syncMode: "full",
    version: "0.118.0",
    ports: {
      rpc: 10332,
      p2p: 10333,
      metrics: 2112,
    },
    paths: {
      base: basePath,
      data: join(basePath, "data"),
      logs: join(basePath, "logs"),
      config: join(basePath, "config"),
    },
    settings: {
      minPeers: 8,
      maxPeers: 50,
      relay: false,
    },
    createdAt: 1,
    updatedAt: 1,
  };
}

function createNeoGoInstallation(): string {
  const basePath = createTempDir();
  mkdirSync(join(basePath, "data"), { recursive: true });

  // The detector should recognize the same config file NeoNexus writes for neo-go nodes.
  const config = ConfigManager.generateNeoGoConfig(createNeoGoNodeConfig(basePath));

  writeFileSync(join(basePath, "neo-go"), "");
  writeFileSync(join(basePath, "protocol.yml"), YAML.dump(config, { lineWidth: -1 }));

  return basePath;
}

describe("NodeDetector", () => {
  it("throws when the installation path does not exist", () => {
    expect(() => NodeDetector.detect("/nonexistent/path/that/does/not/exist")).toThrow(
      "Path does not exist",
    );
  });

  it("detects neo-go installations from the generated protocol.yml format", () => {
    const basePath = createNeoGoInstallation();

    const detected = NodeDetector.detect(basePath);

    expect(detected).toMatchObject({
      type: "neo-go",
      network: "mainnet",
      version: "0.104.0",
      ports: {
        rpc: 10332,
        p2p: 10333,
      },
      dataPath: join(basePath, "data"),
      configPath: join(basePath, "protocol.yml"),
    });
  });

  it("validates detected neo-go imports when the generated files are present", () => {
    const basePath = createNeoGoInstallation();
    const detected = NodeDetector.detect(basePath);

    expect(detected).not.toBeNull();
    expect(NodeDetector.validateImport(detected!)).toEqual({
      valid: true,
      errors: [],
    });
  });
});
