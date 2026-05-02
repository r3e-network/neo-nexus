import { describe, expect, it } from "vitest";
import { getWebSocketAuthToken } from "../../src/server";

describe("WebSocket authentication token extraction", () => {
  it("accepts bearer tokens from the Authorization header", () => {
    expect(getWebSocketAuthToken({
      headers: { authorization: "Bearer header-token" },
      url: "/ws",
    })).toBe("header-token");
  });

  it("accepts browser-safe bearer tokens from Sec-WebSocket-Protocol", () => {
    expect(getWebSocketAuthToken({
      headers: { "sec-websocket-protocol": "neonexus.auth, protocol-token" },
      url: "/ws",
    })).toBe("protocol-token");
  });

  it("does not accept bearer tokens from the WebSocket URL query string by default", () => {
    expect(getWebSocketAuthToken({
      headers: {},
      url: "/ws?token=query-token",
    })).toBeNull();
  });
});
