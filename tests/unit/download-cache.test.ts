import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { hasUsableDownloadFile } from "../../src/core/DownloadManager";

describe("hasUsableDownloadFile", () => {
  it("returns false for missing files", () => {
    expect(hasUsableDownloadFile("/tmp/does-not-exist")).toBe(false);
  });

  it("returns false for zero-byte files", () => {
    const dir = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const file = join(dir, "empty.zip");
    writeFileSync(file, "");

    expect(hasUsableDownloadFile(file)).toBe(false);
  });

  it("returns true for non-empty files", () => {
    const dir = mkdtempSync(join(tmpdir(), "neonexus-download-"));
    const file = join(dir, "asset.zip");
    writeFileSync(file, "data");

    expect(hasUsableDownloadFile(file)).toBe(true);
  });
});
