import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../../web/src/utils/api";

describe("web api helper", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.stubGlobal(
      "localStorage",
      {
        getItem: vi.fn((key: string) => (key === "token" ? "token-123" : null)),
      } as Storage,
    );
  });

  it("includes the bearer token on authenticated post requests", async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({ success: true }),
    }));
    vi.stubGlobal("fetch", fetchMock);

    await api.post("/api/test", { hello: "world" });

    expect(fetchMock).toHaveBeenCalledWith("/api/test", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer token-123",
      },
      body: JSON.stringify({ hello: "world" }),
    });
  });

  it("supports authenticated put requests for settings and plugin updates", async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({ success: true }),
    }));
    vi.stubGlobal("fetch", fetchMock);

    await api.put("/api/test", { enabled: true });

    expect(fetchMock).toHaveBeenCalledWith("/api/test", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer token-123",
      },
      body: JSON.stringify({ enabled: true }),
    });
  });
});
