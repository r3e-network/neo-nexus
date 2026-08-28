import { describe, expect, it } from "vitest";
import { isZipSymlink, validateZipEntryPath } from "../../src/utils/safeZip";

describe("safe ZIP extraction boundaries", () => {
  it.each([
    "../outside.txt",
    "nested/../../outside.txt",
    "/absolute.txt",
    "C:/windows.txt",
    "nested\\windows.txt",
    "bad\0name.txt",
  ])("rejects unsafe entry path %s", (entryName) => {
    expect(() => validateZipEntryPath("/tmp/extract", entryName)).toThrow();
  });

  it("keeps normal entries inside the destination", () => {
    expect(validateZipEntryPath("/tmp/extract", "plugins/RpcServer.dll")).toBe(
      "/tmp/extract/plugins/RpcServer.dll",
    );
  });

  it("detects Unix symbolic-link entries", () => {
    expect(isZipSymlink({ externalFileAttributes: 0o120777 << 16 })).toBe(true);
    expect(isZipSymlink({ externalFileAttributes: 0o100644 << 16 })).toBe(false);
  });
});
