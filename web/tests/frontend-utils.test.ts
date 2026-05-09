import { describe, expect, it } from "vitest";
import { formatBytes, formatDuration, formatVersion } from "../src/utils/format";
import { hasPluginConfigChanges } from "../src/pages/plugins/PluginCard";
import { nextSaveAndTestEnabledState } from "../src/components/IntegrationCard";
import { getPublicDashboardFreshness, PUBLIC_DASHBOARD_STALE_AFTER_MS } from "../src/pages/PublicDashboard";
import { mergeNodeLogs, type RealtimeLogEntry } from "../src/utils/realtime";
import { PROJECT_LINKS } from "../src/config/constants";
import { getDefaultCreateNodeFormValues } from "../src/pages/CreateNode";
import { getBlockHeightStatus } from "../src/utils/blockHeightStatus";
import { evaluatePasswordStrength } from "../src/utils/passwordStrength";

describe("frontend formatting utilities", () => {
  it("formats byte counts for dashboard resource cards", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1 KB");
    expect(formatBytes(1.5 * 1024 * 1024)).toBe("1.5 MB");
  });

  it("formats version tags without double-prefixing 'v'", () => {
    expect(formatVersion("3.5.2")).toBe("v3.5.2");
    expect(formatVersion("v3.5.2")).toBe("v3.5.2");
  });

  it("leaves non-semver labels (e.g. 'external' on sidecars) unprefixed", () => {
    // Sidecars persist `version: "external"`; rendering "vexternal" was
    // ugly and operationally wrong since there's no such version.
    expect(formatVersion("external")).toBe("external");
    expect(formatVersion("nightly")).toBe("nightly");
  });

  it("returns empty string for null/undefined/empty input", () => {
    expect(formatVersion(null)).toBe("");
    expect(formatVersion(undefined)).toBe("");
    expect(formatVersion("")).toBe("");
    expect(formatVersion("   ")).toBe("");
  });

  it("formats uptime durations compactly", () => {
    expect(formatDuration(5_000)).toBe("5s");
    expect(formatDuration(90_000)).toBe("1m");
    expect(formatDuration(3 * 60 * 60 * 1000 + 10 * 60 * 1000)).toBe("3h 10m");
    expect(formatDuration(2 * 24 * 60 * 60 * 1000 + 4 * 60 * 60 * 1000)).toBe("2d 4h");
  });
});

describe("realtime log merging", () => {
  it("deduplicates and sorts base plus realtime log entries", () => {
    const baseLogs: RealtimeLogEntry[] = [
      { timestamp: 20, level: "info", message: "started" },
      { timestamp: 10, level: "warn", message: "warming" },
    ];
    const realtimeLogs: RealtimeLogEntry[] = [
      { timestamp: 20, level: "info", message: "started" },
      { timestamp: 30, level: "error", message: "failed" },
    ];

    expect(mergeNodeLogs(baseLogs, realtimeLogs)).toEqual([
      { timestamp: 10, level: "warn", message: "warming" },
      { timestamp: 20, level: "info", message: "started" },
      { timestamp: 30, level: "error", message: "failed" },
    ]);
  });
});

describe("operator surface helpers", () => {
  it("uses the canonical NeoNexus repository link across the frontend", () => {
    expect(PROJECT_LINKS).toMatchObject({
      repositoryUrl: "https://github.com/r3e-network/neo-nexus",
      repositoryLabel: "github.com/r3e-network/neo-nexus",
    });
  });

  it("does not mark installed plugin config dirty when the draft matches saved config", () => {
    expect(hasPluginConfigChanges({ Port: 10332, Enabled: true }, { Port: 10332, Enabled: true })).toBe(false);
  });

  it("marks installed plugin config dirty when nested values change", () => {
    expect(hasPluginConfigChanges(
      { Servers: [{ Port: 10332 }] },
      { Servers: [{ Port: 20332 }] },
    )).toBe(true);
  });

  it("preserves disabled integration state when saving and testing an existing disabled integration", () => {
    expect(nextSaveAndTestEnabledState(false)).toBe(false);
  });

  it("defaults new nodes to Neo CLI so built-in plugin roles are immediately compatible", () => {
    expect(getDefaultCreateNodeFormValues()).toMatchObject({
      type: "neo-cli",
      network: "mainnet",
      storageEngine: "leveldb",
      syncStrategy: "full",
    });
  });

  it("marks public dashboard data stale when a public query is failing", () => {
    expect(getPublicDashboardFreshness({
      lastUpdatedAt: 1_700_000_000_000,
      now: 1_700_000_005_000,
      hasError: true,
      isFetching: false,
    })).toMatchObject({
      stale: true,
      tone: "warning",
      label: "Stale, updated just now",
    });
  });

  it("marks public-network block heights stale while the node is still catching up", () => {
    expect(getBlockHeightStatus(
      { network: "mainnet", metrics: { blockHeight: 95 } },
      { mainnet: 100, testnet: 0, timestamp: 1_700_000_000_000 },
      1_700_000_001_000,
    )).toMatchObject({
      status: "syncing",
      label: "Syncing",
      detail: "5 blocks behind latest",
      localHeight: 95,
      networkHeight: 100,
      remainingBlocks: 5,
      progressPercent: 95,
      stale: true,
      safeToUseAsLatest: false,
    });
  });

  it("marks caught-up public-network block heights safe to use as latest", () => {
    expect(getBlockHeightStatus(
      { network: "testnet", metrics: { blockHeight: 101 } },
      { mainnet: 0, testnet: 100, timestamp: 1_700_000_000_000 },
      1_700_000_001_000,
    )).toMatchObject({
      status: "synced",
      label: "Synced",
      detail: "Caught up to latest network height",
      localHeight: 101,
      networkHeight: 100,
      remainingBlocks: 0,
      progressPercent: 100,
      stale: false,
      safeToUseAsLatest: true,
    });
  });

  it("does not call a node syncing when block height has not been reported", () => {
    expect(getBlockHeightStatus(
      { network: "mainnet" },
      { mainnet: 100, testnet: 0, timestamp: 1_700_000_000_000 },
      1_700_000_001_000,
    )).toMatchObject({
      status: "unknown",
      label: "No height",
      detail: "Block height has not been reported yet",
      localHeight: null,
      networkHeight: 100,
      remainingBlocks: null,
      stale: false,
      safeToUseAsLatest: false,
    });
  });

  it("gives actionable password feedback for weak setup credentials", () => {
    expect(evaluatePasswordStrength("password")).toMatchObject({
      label: "Weak",
      acceptable: true,
    });
    expect(evaluatePasswordStrength("password").feedback).toContain("Add a number or symbol.");
  });

  it("recognizes long mixed passwords as excellent", () => {
    expect(evaluatePasswordStrength("NeoNexus-Validator-2026!")).toMatchObject({
      label: "Excellent",
      acceptable: true,
      feedback: [],
    });
  });

  it("marks public dashboard data stale after several missed refresh windows", () => {
    expect(getPublicDashboardFreshness({
      lastUpdatedAt: 1_700_000_000_000,
      now: 1_700_000_000_000 + PUBLIC_DASHBOARD_STALE_AFTER_MS + 1,
      hasError: false,
      isFetching: false,
    })).toMatchObject({
      stale: true,
      tone: "warning",
    });
  });
});
