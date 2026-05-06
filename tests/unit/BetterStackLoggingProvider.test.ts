import { beforeEach, describe, expect, it, vi } from "vitest";

const safeFetchMock = vi.hoisted(() => vi.fn());

vi.mock("../../src/integrations/safeFetch", () => ({
  safeIntegrationFetch: safeFetchMock,
}));

import { BetterStackLoggingProvider } from "../../src/integrations/providers/logging/BetterStackLoggingProvider";

describe("BetterStackLoggingProvider", () => {
  beforeEach(() => {
    safeFetchMock.mockReset();
    safeFetchMock.mockResolvedValue({ ok: true, status: 202, statusText: "Accepted" });
  });

  it("uses the default Better Stack ingest endpoint when none is configured", async () => {
    const provider = new BetterStackLoggingProvider({ sourceToken: "token" });

    await provider.testConnection();

    expect(safeFetchMock).toHaveBeenCalledWith(
      "https://in.logs.betterstack.com",
      expect.objectContaining({
        method: "POST",
        headers: expect.objectContaining({ Authorization: "Bearer token" }),
      }),
    );
  });

  it("allows operators to use the source-specific ingest endpoint from Better Stack", async () => {
    const provider = new BetterStackLoggingProvider({
      sourceToken: "token",
      ingestingUrl: "https://s12345.eu-nbg-2.betterstackdata.com",
    });

    await provider.pushLogs([{
      timestamp: Date.now(),
      level: "info",
      message: "node started",
      source: "node",
      nodeId: "node-1",
      nodeName: "Node 1",
    }]);

    expect(safeFetchMock).toHaveBeenCalledWith(
      "https://s12345.eu-nbg-2.betterstackdata.com",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining("node started"),
      }),
    );
  });
});
