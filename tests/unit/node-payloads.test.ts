import { describe, expect, it } from "vitest";
import { normalizeNodeUpsertPayload, toNodeFormValues } from "../../web/src/utils/nodePayloads";

describe("normalizeNodeUpsertPayload", () => {
  it("builds a create payload with trimmed name and parsed numeric settings", () => {
    const payload = normalizeNodeUpsertPayload({
      name: "  Mainnet Node  ",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      maxConnections: "80",
      minPeers: "8",
      maxPeers: "120",
      relay: true,
      debugMode: false,
      customConfig: '{ "Trace": true }',
    });

    expect(payload).toEqual({
      name: "Mainnet Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      settings: {
        maxConnections: 80,
        minPeers: 8,
        maxPeers: 120,
        relay: true,
        debugMode: false,
        customConfig: { Trace: true },
      },
    });
  });

  it("serializes secure signer references without any raw secret material", () => {
    const payload = normalizeNodeUpsertPayload({
      name: "Consensus Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      relay: true,
      debugMode: false,
      keyProtectionMode: "secure-signer",
      secureSignerProfileId: "signer-1",
    });

    expect(payload).toEqual({
      name: "Consensus Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      settings: {
        relay: true,
        debugMode: false,
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });
  });

  it("omits blank optional values from an update payload", () => {
    const payload = normalizeNodeUpsertPayload({
      name: "Updated Node",
      maxConnections: "",
      minPeers: "",
      maxPeers: "40",
      relay: false,
      debugMode: true,
      customConfig: "   ",
    });

    expect(payload).toEqual({
      name: "Updated Node",
      settings: {
        maxPeers: 40,
        relay: false,
        debugMode: true,
      },
    });
  });

  it("throws on invalid custom config json", () => {
    expect(() =>
      normalizeNodeUpsertPayload({
        name: "Broken Node",
        customConfig: "{ invalid json }",
      }),
    ).toThrow("Custom config must be valid JSON");
  });
});

describe("toNodeFormValues", () => {
  it("maps an existing node into editable form state", () => {
    expect(
      toNodeFormValues({
        name: "Existing Node",
        type: "neo-cli",
        network: "testnet",
        syncMode: "light",
        settings: {
          maxConnections: 60,
          minPeers: 5,
          maxPeers: 90,
          relay: false,
          debugMode: true,
          customConfig: { Trace: true },
        },
      }),
    ).toEqual({
      name: "Existing Node",
      type: "neo-cli",
      network: "testnet",
      syncMode: "light",
      maxConnections: "60",
      minPeers: "5",
      maxPeers: "90",
      relay: false,
      debugMode: true,
      customConfig: '{\n  "Trace": true\n}',
      keyProtectionMode: "standard",
      secureSignerProfileId: "",
    });
  });

  it("maps secure signer bindings into editable form state", () => {
    expect(
      toNodeFormValues({
        name: "Protected Node",
        type: "neo-cli",
        network: "mainnet",
        syncMode: "full",
        settings: {
          relay: true,
          debugMode: false,
          keyProtection: {
            mode: "secure-signer",
            signerProfileId: "signer-tee",
          },
        },
      }),
    ).toEqual({
      name: "Protected Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      maxConnections: "",
      minPeers: "",
      maxPeers: "",
      relay: true,
      debugMode: false,
      customConfig: "",
      keyProtectionMode: "secure-signer",
      secureSignerProfileId: "signer-tee",
    });
  });
});
