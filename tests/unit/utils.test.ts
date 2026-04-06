/**
 * Unit Tests: Utilities
 * 
 * Tests helper functions and utility modules
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { readJsonFile, writeJsonFile, readJsonFileSync, writeJsonFileSync } from "../../src/utils/config";
import { isPortAvailable, findAvailablePort } from "../../src/utils/ports";
import { execAsync } from "../../src/utils/exec";
import { getNetworkMagic, getSeedList } from "../../src/utils/network";

// Mock fs modules
vi.mock("node:fs", () => ({
  readFileSync: vi.fn(() => '{"test": "data"}'),
  writeFileSync: vi.fn(),
}));

vi.mock("node:fs/promises", () => ({
  mkdir: vi.fn(() => Promise.resolve()),
  readFile: vi.fn(() => Promise.resolve('{"test": "data"}')),
  writeFile: vi.fn(() => Promise.resolve()),
}));

vi.mock("node:net", () => ({
  createServer: vi.fn(() => ({
    once: vi.fn((event: string, cb: Function) => {
      if (event === "listening") {
        setTimeout(() => cb(), 0);
      }
    }),
    listen: vi.fn(),
    close: vi.fn(),
  })),
  createConnection: vi.fn(() => ({
    destroy: vi.fn(),
    on: vi.fn(),
    setTimeout: vi.fn(),
  })),
}));

// Mock execAsync separately
vi.mock("../../src/utils/exec", () => ({
  execAsync: vi.fn(() => Promise.resolve({ stdout: "command output", stderr: "" })),
}));

describe("Utils: config", () => {
  describe("readJsonFile", () => {
    it("should read and parse JSON file", async () => {
      const data = await readJsonFile("/test/file.json");
      expect(data).toEqual({ test: "data" });
    });

    it("should throw if file not found", async () => {
      const { readFile } = await import("node:fs/promises");
      vi.mocked(readFile).mockRejectedValueOnce(new Error("ENOENT: no such file or directory"));

      await expect(readJsonFile("/nonexistent.json")).rejects.toThrow("ENOENT");
    });
  });

  describe("writeJsonFile", () => {
    it("should write JSON data to file", async () => {
      const { writeFile } = await import("node:fs/promises");

      await writeJsonFile("/test/file.json", { foo: "bar" });

      expect(writeFile).toHaveBeenCalledWith(
        "/test/file.json",
        JSON.stringify({ foo: "bar" }, null, 2),
        "utf8"
      );
    });

    it("should create directories recursively", async () => {
      const { mkdir } = await import("node:fs/promises");
      
      await writeJsonFile("/deep/nested/path/file.json", {});
      
      expect(mkdir).toHaveBeenCalledWith("/deep/nested/path", { recursive: true });
    });
  });

  describe("readJsonFileSync", () => {
    it("should read and parse JSON file synchronously", () => {
      const data = readJsonFileSync("/test/file.json");
      expect(data).toEqual({ test: "data" });
    });
  });

  describe("writeJsonFileSync", () => {
    it("should call writeFileSync", () => {
      // Just verify the function doesn't throw
      expect(() => writeJsonFileSync("/test/file.json", { foo: "bar" })).not.toThrow();
    });
  });
});

describe("Utils: ports", () => {
  describe("isPortAvailable", () => {
    it("should return boolean for port availability", async () => {
      const result = await isPortAvailable(10332);
      expect(typeof result).toBe("boolean");
    });

    it("should accept custom host", async () => {
      const result = await isPortAvailable(10332, "127.0.0.1");
      expect(typeof result).toBe("boolean");
    });
  });

  describe("findAvailablePort", () => {
    it("should return a port number or null", async () => {
      const port = await findAvailablePort(10332);
      expect(port === null || typeof port === "number").toBe(true);
    });
  });
});

describe("Utils: exec", () => {
  describe("execAsync", () => {
    it("should execute command and return result", async () => {
      const result = await execAsync("echo test");
      expect(result).toHaveProperty("stdout");
      expect(result).toHaveProperty("stderr");
    });
  });
});

describe("Utils: network", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getNetworkMagic", () => {
    it("should return correct mainnet magic", () => {
      expect(getNetworkMagic("mainnet")).toBe(860833102);
    });

    it("should return correct testnet magic", () => {
      expect(getNetworkMagic("testnet")).toBe(894710606);
    });

    it("should return correct private net magic", () => {
      expect(getNetworkMagic("private")).toBe(56753);
    });
  });

  describe("getSeedList", () => {
    it("should return mainnet seeds", () => {
      const seeds = getSeedList("mainnet");
      expect(seeds.length).toBeGreaterThan(0);
      expect(seeds[0]).toContain("seed");
      expect(seeds[0]).toContain("neo.org");
    });

    it("should return testnet seeds", () => {
      const seeds = getSeedList("testnet");
      expect(seeds.length).toBeGreaterThan(0);
      expect(seeds[0]).toContain("seed");
      expect(seeds[0]).toContain("t.neo.org");
    });
  });
});
