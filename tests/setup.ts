/**
 * Test Setup for NeoNexus
 * 
 * Global test configuration, mocks, and utilities
 */

import { vi } from "vitest";
import path from "path";
import os from "os";
import fs from "fs";

// ============================================
// Test Environment Configuration
// ============================================

process.env.NODE_ENV = "test";
process.env.JWT_SECRET = "test-jwt-secret-key-for-testing-only";
process.env.DATA_DIR = path.join(os.tmpdir(), "neonexus-test-" + Date.now());

// Create test data directory
if (!fs.existsSync(process.env.DATA_DIR)) {
  fs.mkdirSync(process.env.DATA_DIR, { recursive: true });
}

// ============================================
// Global Mocks
// ============================================

// Mock better-sqlite3
vi.mock("better-sqlite3", () => {
  return {
    default: vi.fn(() => ({
      pragma: vi.fn(),
      exec: vi.fn(),
      prepare: vi.fn(() => ({
        run: vi.fn(),
        get: vi.fn(),
        all: vi.fn(() => []),
      })),
      close: vi.fn(),
    })),
  };
});

// Mock bcrypt
vi.mock("bcrypt", () => ({
  default: {
    hash: vi.fn((pwd: string) => Promise.resolve(`hashed_${pwd}`)),
    compare: vi.fn((pwd: string, hash: string) => Promise.resolve(hash === `hashed_${pwd}`)),
    genSalt: vi.fn(() => Promise.resolve("somesalt")),
  },
  hash: vi.fn((pwd: string) => Promise.resolve(`hashed_${pwd}`)),
  compare: vi.fn((pwd: string, hash: string) => Promise.resolve(hash === `hashed_${pwd}`)),
  genSalt: vi.fn(() => Promise.resolve("somesalt")),
}));

// Mock jsonwebtoken
vi.mock("jsonwebtoken", () => ({
  sign: vi.fn(() => "mock-jwt-token"),
  verify: vi.fn((token: string) => {
    if (token === "valid-token" || token === "mock-jwt-token") {
      return { userId: "test-user-id", username: "admin", role: "admin" };
    }
    throw new Error("Invalid token");
  }),
  decode: vi.fn(() => ({ userId: "test-user-id", username: "admin" })),
}));

// Mock systeminformation
vi.mock("systeminformation", () => ({
  currentLoad: vi.fn(() => Promise.resolve({ currentLoad: 45.5 })),
  mem: vi.fn(() => Promise.resolve({
    total: 16000000000,
    active: 8000000000,
    available: 8000000000,
  })),
  fsSize: vi.fn(() => Promise.resolve([{
    size: 1000000000000,
    used: 500000000000,
    available: 500000000000,
  }])),
  networkStats: vi.fn(() => Promise.resolve([{
    rx_sec: 1024,
    tx_sec: 2048,
  }])),
}));

// Mock ws (WebSocket)
vi.mock("ws", () => ({
  WebSocket: class MockWebSocket {
    static OPEN = 1;
    static CLOSED = 3;
    readyState = 1;
    on = vi.fn();
    send = vi.fn();
    close = vi.fn();
    ping = vi.fn();
  },
  WebSocketServer: class MockWebSocketServer {
    on = vi.fn();
    clients = new Set();
  },
}));

// Mock child_process
vi.mock("child_process", () => ({
  spawn: vi.fn(() => ({
    on: vi.fn(),
    stdout: { on: vi.fn() },
    stderr: { on: vi.fn() },
    kill: vi.fn(),
    pid: 12345,
  })),
  exec: vi.fn((cmd: string, cb: Function) => cb(null, "output", "")),
  execSync: vi.fn(() => Buffer.from("output")),
}));

// Mock fs/promises
vi.mock("fs/promises", () => ({
  access: vi.fn(() => Promise.resolve()),
  mkdir: vi.fn(() => Promise.resolve()),
  readdir: vi.fn(() => Promise.resolve([])),
  readFile: vi.fn(() => Promise.resolve(Buffer.from("{}"))),
  writeFile: vi.fn(() => Promise.resolve()),
  copyFile: vi.fn(() => Promise.resolve()),
  stat: vi.fn(() => Promise.resolve({ isDirectory: () => true } as any)),
  unlink: vi.fn(() => Promise.resolve()),
  rm: vi.fn(() => Promise.resolve()),
  chmod: vi.fn(() => Promise.resolve()),
}));

// Mock node-cron
vi.mock("node-cron", () => ({
  schedule: vi.fn(() => ({
    start: vi.fn(),
    stop: vi.fn(),
    destroy: vi.fn(),
  })),
  validate: vi.fn(() => true),
}));

// ============================================
// Test Utilities
// ============================================

export function createMockRequest(overrides: any = {}) {
  return {
    params: {},
    query: {},
    body: {},
    headers: {},
    user: { userId: "test-user", username: "admin", role: "admin" },
    ...overrides,
  };
}

export function createMockResponse() {
  const res: any = {
    statusCode: 200,
    jsonData: null,
    status: vi.fn(function(code: number) {
      res.statusCode = code;
      return res;
    }),
    json: vi.fn(function(data: any) {
      res.jsonData = data;
      return res;
    }),
    send: vi.fn(function(data: any) {
      res.jsonData = data;
      return res;
    }),
    setHeader: vi.fn(),
    getHeader: vi.fn(),
  };
  return res;
}

export function createMockNode(overrides: any = {}) {
  return {
    id: "test-node-1",
    name: "Test Node",
    type: "neo-cli",
    network: "mainnet",
    status: "stopped",
    version: "3.6.0",
    metrics: {
      blockHeight: 1000,
      headerHeight: 1000,
      connectedPeers: 10,
      unconnectedPeers: 5,
      syncProgress: 100,
      cpuUsage: 25,
      memoryUsage: 512,
      lastUpdate: Date.now(),
    },
    ...overrides,
  };
}

// ============================================
// Cleanup
// ============================================

export function cleanupTestData() {
  if (process.env.DATA_DIR && fs.existsSync(process.env.DATA_DIR)) {
    fs.rmSync(process.env.DATA_DIR, { recursive: true, force: true });
  }
}

// Cleanup on test completion
process.on("exit", cleanupTestData);
