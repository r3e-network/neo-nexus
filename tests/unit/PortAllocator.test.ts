/**
 * Unit Tests: PortAllocator
 * 
 * Tests port allocation logic with mocked database
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { PortAllocator } from "../../src/core/PortAllocator";

// Create a mock database
function createMockDb(usedPorts: Array<{ rpc_port: number; p2p_port: number }> = []) {
  return {
    prepare: vi.fn((sql: string) => {
      if (sql.includes("SELECT rpc_port, p2p_port FROM nodes")) {
        return {
          all: vi.fn(() => usedPorts),
        };
      }
      return { all: vi.fn(() => []), get: vi.fn(() => null), run: vi.fn() };
    }),
  };
}

vi.mock("../../src/utils/ports", () => ({
  isPortAvailable: vi.fn((port: number) => Promise.resolve(port < 65000)),
  findAvailablePort: vi.fn(async (startPort: number) => {
    // Simulate finding next available port
    let port = startPort;
    while (port < 65000) {
      if (await vi.mocked(isPortAvailable)(port)) {
        return port;
      }
      port++;
    }
    throw new Error("No available ports");
  }),
}));

import { isPortAvailable } from "../../src/utils/ports";

describe("PortAllocator", () => {
  let mockDb: any;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("allocatePorts", () => {
    it("should allocate default ports when no nodes exist", async () => {
      mockDb = createMockDb([]);
      const allocator = new PortAllocator(mockDb, 10333, 10334);

      const ports = await allocator.allocatePorts();

      expect(ports.rpcPort).toBeGreaterThanOrEqual(10333);
      expect(ports.p2pPort).toBeGreaterThanOrEqual(10334);
      expect(ports.rpcPort).not.toBe(ports.p2pPort);
    });

    it("should skip used ports", async () => {
      mockDb = createMockDb([
        { rpc_port: 10333, p2p_port: 10334 },
      ]);
      
      // Re-import to get fresh mock
      const { PortAllocator: FreshPortAllocator } = await import("../../src/core/PortAllocator");
      const allocator = new FreshPortAllocator(mockDb, 10333, 10334);

      // Mock the port check to return unavailable for used ports
      vi.mocked(isPortAvailable).mockImplementation((port) => {
        return Promise.resolve(port > 10334); // Only ports > 10334 available
      });

      const ports = await allocator.allocatePorts();

      expect(ports.rpcPort).toBeGreaterThan(10333);
      expect(ports.p2pPort).toBeGreaterThan(10334);
    });

    it("should allocate different ports for multiple calls", async () => {
      mockDb = createMockDb([]);
      
      // Track allocated ports
      let lastRpcPort = 10332;
      vi.mocked(isPortAvailable).mockImplementation((port) => {
        return Promise.resolve(port > lastRpcPort);
      });

      const { PortAllocator: FreshPortAllocator } = await import("../../src/core/PortAllocator");
      const allocator = new FreshPortAllocator(mockDb, 10333, 10334);

      const ports1 = await allocator.allocatePorts();
      lastRpcPort = ports1.rpcPort;
      
      // Update mock db with first allocation
      mockDb = createMockDb([{ rpc_port: ports1.rpcPort, p2p_port: ports1.p2pPort }]);
      const allocator2 = new FreshPortAllocator(mockDb, 10333, 10334);
      
      const ports2 = await allocator2.allocatePorts();

      expect(ports1.rpcPort).not.toBe(ports2.rpcPort);
      expect(ports1.p2pPort).not.toBe(ports2.p2pPort);
    });
  });

  describe("port range validation", () => {
    it("should use custom base ports", async () => {
      mockDb = createMockDb([]);
      const allocator = new PortAllocator(mockDb, 20000, 20001);

      const ports = await allocator.allocatePorts();

      expect(ports.rpcPort).toBeGreaterThanOrEqual(20000);
      expect(ports.p2pPort).toBeGreaterThanOrEqual(20001);
    });

    it("should handle system-reserved ports gracefully", async () => {
      mockDb = createMockDb([]);
      
      // Try to use low port numbers
      const allocator = new PortAllocator(mockDb, 80, 81);
      
      // Mock that low ports are not available
      vi.mocked(isPortAvailable).mockResolvedValue(false);
      
      await expect(allocator.allocatePorts()).rejects.toThrow();
    });
  });
});
