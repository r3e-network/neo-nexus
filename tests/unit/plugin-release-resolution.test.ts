import { describe, expect, it } from "vitest";
import { resolvePluginReleaseVersion } from "../../src/core/PluginManager";

describe("resolvePluginReleaseVersion", () => {
  it("uses the explicit mapped version when available", () => {
    expect(resolvePluginReleaseVersion("3.7.5", "v3.7.5")).toBe("v3.7.5");
  });

  it("falls back to the latest known neo-modules release when the node version has no matching plugin release", () => {
    expect(resolvePluginReleaseVersion("99.0.0", "v3.7.5")).toBe("v3.7.5");
  });
});
