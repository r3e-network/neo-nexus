import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { NeoCliNode } from "../../src/nodes/NeoCliNode";

describe("NeoCliNode getStartArgs", () => {
  it("shell-quotes imported neo-cli paths before passing them to script -c", () => {
    const root = mkdtempSync(join(tmpdir(), "neonexus-cli-"));
    const base = join(root, "node with ' quote; touch injected");
    mkdirSync(base, { recursive: true });
    writeFileSync(join(base, "neo-cli.dll"), "");

    try {
      const node = new NeoCliNode({
        id: "node-imported",
        name: "Imported",
        type: "neo-cli",
        network: "testnet",
        syncMode: "full",
        version: "v3.9.2",
        ports: { rpc: 20332, p2p: 20333 },
        paths: {
          base,
          data: join(base, "Chain"),
          logs: join(base, "Logs"),
          config: join(base, "config.json"),
        },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
      });

      const args = node.getStartArgs();
      const command = args[1];

      expect(args[0]).toBe("-qfc");
      expect(command).toContain("'dotnet'");
      expect(command).toContain("'--log-level' 'Info'");
      expect(command).toContain("'\"'\"'");
      expect(command).not.toContain(`${base}/neo-cli.dll --rpc`);
      expect(args[2]).toBe("/dev/null");
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
