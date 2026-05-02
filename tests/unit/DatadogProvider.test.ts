import { describe, expect, it } from "vitest";
import { DatadogProvider } from "../../src/integrations/providers/metrics/DatadogProvider";

describe("DatadogProvider site validation", () => {
  it("accepts standard Datadog site values", () => {
    for (const site of ["datadoghq.com", "us3.datadoghq.com", "us5.datadoghq.com", "datadoghq.eu", "ap1.datadoghq.com"]) {
      expect(() => new DatadogProvider({ apiKey: "key", site })).not.toThrow();
    }
  });

  it("rejects site values that could break out of the URL hostname", () => {
    for (const site of [
      "@evil.com",
      "evil.com#",
      "evil.com/path",
      "evil.com?x=1",
      "evil.com:1234",
      "evil .com",
      "evil_com",
      "127.0.0.1@evil.com",
      "datadoghq.com\r\nHost: evil.com",
    ]) {
      expect(() => new DatadogProvider({ apiKey: "key", site }), `should reject ${JSON.stringify(site)}`).toThrow(/invalid datadog site/i);
    }
  });
});
