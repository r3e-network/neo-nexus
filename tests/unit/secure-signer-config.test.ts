import { describe, expect, it } from "vitest";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ConfigManager } from "../../src/core/ConfigManager";
import type { NodeConfig } from "../../src/types";

function createNeoCliNode(baseDir: string): NodeConfig {
  return {
    id: "node-secure-signer",
    name: "Secure Signer Node",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.7.5",
    ports: {
      rpc: 10332,
      p2p: 10333,
    },
    paths: {
      base: baseDir,
      data: path.join(baseDir, "Data"),
      logs: path.join(baseDir, "Logs"),
      config: path.join(baseDir, "config"),
      wallet: path.join(baseDir, "wallets"),
    },
    settings: {},
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
}

describe("secure signer config generation", () => {
  it("generates SignClient plugin config from secure signer settings", () => {
    const node = createNeoCliNode(path.join(os.tmpdir(), "neonexus-signclient-generate"));

    const config = (ConfigManager as any).generatePluginConfig("SignClient", node, {
      Name: "Nitro Council Signer",
      Endpoint: "vsock://2345:9991",
    });

    expect(config).toEqual({
      PluginConfiguration: {
        Name: "Nitro Council Signer",
        Endpoint: "vsock://2345:9991",
      },
    });
  });

  it("writes SignClient.json with the stored secure signer endpoint", () => {
    const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-signclient-write-"));
    const node = createNeoCliNode(baseDir);

    (ConfigManager as any).writePluginConfig("SignClient", node, {
      Name: "SGX Signer",
      Endpoint: "https://signer.example.com:9443",
    });

    const written = JSON.parse(fs.readFileSync(path.join(baseDir, "Plugins", "SignClient", "SignClient.json"), "utf8"));
    expect(written).toEqual({
      PluginConfiguration: {
        Name: "SGX Signer",
        Endpoint: "https://signer.example.com:9443",
      },
    });
  });
});
