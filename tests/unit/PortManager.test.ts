import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../src/utils/ports", () => ({
  isPortAvailable: vi.fn(),
}));

import { PortManager } from "../../src/core/PortManager";
import { isPortAvailable } from "../../src/utils/ports";

describe("PortManager", () => {
  beforeEach(() => {
    vi.mocked(isPortAvailable).mockReset();
  });

  it("skips a managed port index when any port in that index is unavailable", async () => {
    vi.mocked(isPortAvailable).mockImplementation(async (port) => port !== 10333);

    const manager = new PortManager();

    await expect(manager.findNextIndex()).resolves.toBe(1);
    expect(isPortAvailable).toHaveBeenCalledWith(10332);
    expect(isPortAvailable).toHaveBeenCalledWith(10333);
    expect(isPortAvailable).toHaveBeenCalledWith(10342);
    expect(isPortAvailable).toHaveBeenCalledWith(10343);
  });
});
