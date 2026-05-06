import { mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import https from "node:https";
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
    vi.restoreAllMocks();
    delete process.env.NEO_PLUGIN_BUILD_DIR;
    delete process.env.NEONEXUS_ALLOW_PRIVATE_DOWNLOAD_TARGETS;
    vi.doUnmock("extract-zip");
    vi.doUnmock("../../src/utils/paths");
    vi.doUnmock("node:dns/promises");
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

  it("does not treat symlinks as usable download cache files", async () => {
    const { hasUsableDownloadFile } = await import("../../src/core/DownloadManager");
    const dir = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const target = join(dir, "target.zip");
    const link = join(dir, "asset.zip");
    writeFileSync(target, "data");
    symlinkSync(target, link);

    expect(hasUsableDownloadFile(link)).toBe(false);
  });

  it("refuses download size probes to private literal targets", async () => {
    const { DownloadManager } = await import("../../src/core/DownloadManager");
    await expect(DownloadManager.getDownloadSize("https://127.0.0.1/releases/node.zip")).resolves.toBeNull();
  });

  it("refuses download size probes when DNS resolves to a private target", async () => {
    vi.doMock("node:dns/promises", () => ({
      lookup: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    }));
    const requestSpy = vi.spyOn(https, "request");
    const { DownloadManager } = await import("../../src/core/DownloadManager");

    await expect(DownloadManager.getDownloadSize("https://downloads.example.test/releases/node.zip")).resolves.toBeNull();

    expect(requestSpy).not.toHaveBeenCalled();
  });

  it("refuses binary downloads when DNS resolves GitHub to a private target", async () => {
    vi.doMock("node:dns/promises", () => ({
      lookup: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    }));
    const requestSpy = vi.spyOn(https, "request");
    const root = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const { DownloadManager } = await importDownloadManagerWithTempPaths(root);

    await expect(DownloadManager.downloadNeoGo("v0.104.0")).rejects.toThrow(/private or local address/i);

    expect(requestSpy).not.toHaveBeenCalled();
  });

  it("does not resolve node binary paths through symlinks", async () => {
    const root = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const { DownloadManager } = await importDownloadManagerWithTempPaths(root);
    const binaryDir = join(root, "downloads", "neo-go-v0.104.0");
    mkdirSync(binaryDir, { recursive: true });
    const target = join(binaryDir, "target-neo-go");
    const link = join(binaryDir, process.platform === "win32" ? "neo-go.exe" : "neo-go");
    writeFileSync(target, "binary");
    symlinkSync(target, link);

    expect(DownloadManager.getNodeBinaryPath("neo-go", "v0.104.0")).toBeNull();
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

  it("discovers local plugin build target frameworks instead of requiring net10.0", async () => {
    const root = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const localBuildRoot = mkdtempSync(join(tmpdir(), "neonexus-plugin-build-"));
    const localOutput = join(localBuildRoot, "RpcServer", "bin", "Release", "net9.0");
    mkdirSync(localOutput, { recursive: true });
    mkdirSync(join(root, "downloads"), { recursive: true });
    writeFileSync(join(root, "downloads", "plugin-RpcServer-v3.9.2.zip"), "cached zip");
    process.env.NEO_PLUGIN_BUILD_DIR = localBuildRoot;
    const { DownloadManager, extractZip } = await importDownloadManagerWithTempPaths(root);

    const result = await DownloadManager.downloadPlugin("RpcServer", "v3.9.2");

    expect(result).toBe(localOutput);
    expect(extractZip).not.toHaveBeenCalled();
  });
});
