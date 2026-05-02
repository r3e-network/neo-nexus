import { afterEach, describe, expect, it, vi } from "vitest";
import { join } from "node:path";

describe("paths", () => {
  const originalDataDir = process.env.DATA_DIR;
  const originalNeonexusDataDir = process.env.NEONEXUS_DATA_DIR;

  afterEach(() => {
    if (originalDataDir === undefined) {
      delete process.env.DATA_DIR;
    } else {
      process.env.DATA_DIR = originalDataDir;
    }
    if (originalNeonexusDataDir === undefined) {
      delete process.env.NEONEXUS_DATA_DIR;
    } else {
      process.env.NEONEXUS_DATA_DIR = originalNeonexusDataDir;
    }
    vi.resetModules();
  });

  it("uses DATA_DIR as the storage root when configured", async () => {
    process.env.DATA_DIR = "/tmp/neonexus-custom-data";
    vi.resetModules();

    const { paths } = await import("../../src/utils/paths");

    expect(paths.base).toBe("/tmp/neonexus-custom-data");
    expect(paths.nodes).toBe(join("/tmp/neonexus-custom-data", "nodes"));
    expect(paths.database).toBe(join("/tmp/neonexus-custom-data", "neonexus.db"));
  });

  it("prefers NEONEXUS_DATA_DIR over DATA_DIR", async () => {
    process.env.DATA_DIR = "/tmp/neonexus-data-dir";
    process.env.NEONEXUS_DATA_DIR = "/tmp/neonexus-explicit-data-dir";
    vi.resetModules();

    const { paths } = await import("../../src/utils/paths");

    expect(paths.base).toBe("/tmp/neonexus-explicit-data-dir");
  });
});
