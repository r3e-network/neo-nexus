import { describe, expect, it, vi } from "vitest";
import { assertProductionAuthenticationSafe, isLoopbackAddress } from "../../src/utils/authSecurity";

describe("authentication security", () => {
  it.each(["127.0.0.1", "127.25.4.9", "::1", "::ffff:127.0.0.1"])(
    "recognizes loopback address %s",
    (address) => {
      expect(isLoopbackAddress(address)).toBe(true);
    },
  );

  it.each(["10.0.0.1", "172.67.135.179", "::ffff:10.0.0.1", "2001:db8::1", undefined])(
    "rejects non-loopback address %s",
    (address) => {
      expect(isLoopbackAddress(address)).toBe(false);
    },
  );

  it("refuses production startup when an admin still has the legacy default password", async () => {
    const userManager = {
      getAllUsers: vi.fn(() => [{ id: "admin-1", role: "admin" as const }]),
      isUsingDefaultPassword: vi.fn(async () => true),
    };

    await expect(assertProductionAuthenticationSafe(userManager, "production")).rejects.toThrow(
      "Refusing to start in production",
    );
  });

  it("allows first-run production startup and non-production development", async () => {
    const firstRunManager = {
      getAllUsers: vi.fn(() => []),
      isUsingDefaultPassword: vi.fn(async () => true),
    };
    const developmentManager = {
      getAllUsers: vi.fn(() => [{ id: "admin-1", role: "admin" as const }]),
      isUsingDefaultPassword: vi.fn(async () => true),
    };

    await expect(assertProductionAuthenticationSafe(firstRunManager, "production")).resolves.toBeUndefined();
    await expect(assertProductionAuthenticationSafe(developmentManager, "development")).resolves.toBeUndefined();
    expect(developmentManager.getAllUsers).not.toHaveBeenCalled();
  });
});
