import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";

async function importDownloadManagerWithTempPaths(root: string) {
  vi.resetModules();
  const extractZip = vi.fn();
  vi.doMock("extract-zip", () => ({ default: extractZip }));
  vi.doMock("../../src/utils/paths", () => ({
    paths: {
      base: root,
      nodes: join(root, "nodes"),
      plugins: join(root, "plugins"),
      downloads: join(root, "downloads"),
      database: join(root, "neonexus.db"),
      logs: join(root, "logs"),
      config: join(root, "config"),
    },
  }));

  const module = await import("../../src/core/DownloadManager");
  return { ...module, extractZip };
}

describe("hasUsableDownloadFile", () => {
  afterEach(() => {
    delete process.env.NEO_PLUGIN_BUILD_DIR;
    vi.doUnmock("extract-zip");
    vi.doUnmock("../../src/utils/paths");
    vi.resetModules();
  });

  it("returns false for missing files", async () => {
    const { hasUsableDownloadFile } = await import("../../src/core/DownloadManager");
    expect(hasUsableDownloadFile("/tmp/does-not-exist")).toBe(false);
  });

  it("returns false for zero-byte files", async () => {
    const { hasUsableDownloadFile } = await import("../../src/core/DownloadManager");
    const dir = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const file = join(dir, "empty.zip");
    writeFileSync(file, "");

    expect(hasUsableDownloadFile(file)).toBe(false);
  });

  it("returns true for non-empty files", async () => {
    const { hasUsableDownloadFile } = await import("../../src/core/DownloadManager");
    const dir = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const file = join(dir, "asset.zip");
    writeFileSync(file, "data");

    expect(hasUsableDownloadFile(file)).toBe(true);
  });

  it("returns an extracted plugin cache without downloading or extracting again", async () => {
    const root = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const { DownloadManager, extractZip } = await importDownloadManagerWithTempPaths(root);
    const pluginDir = join(root, "plugins", "RpcServer", "v3.9.2");
    mkdirSync(pluginDir, { recursive: true });
    writeFileSync(join(pluginDir, "RpcServer.dll"), "binary");

    const result = await DownloadManager.downloadPlugin("RpcServer", "v3.9.2");

    expect(result).toBe(pluginDir);
    expect(extractZip).not.toHaveBeenCalled();
  });

  it("prefers a local plugin build output over GitHub downloads", async () => {
    const root = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const localBuildRoot = mkdtempSync(join(tmpdir(), "neonexus-plugin-build-"));
    const localOutput = join(localBuildRoot, "RpcServer", "bin", "Release", "net10.0");
    mkdirSync(localOutput, { recursive: true });
    process.env.NEO_PLUGIN_BUILD_DIR = localBuildRoot;
    const { DownloadManager, extractZip } = await importDownloadManagerWithTempPaths(root);

    const result = await DownloadManager.downloadPlugin("RpcServer", "v3.9.2");

    expect(result).toBe(localOutput);
    expect(extractZip).not.toHaveBeenCalled();
  });
});
