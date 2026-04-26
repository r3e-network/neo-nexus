import { describe, expect, it } from "vitest";
import { normalizeNodeUpsertPayload, toNodeFormValues } from "../../web/src/utils/nodePayloads";

describe("web node payload normalization", () => {
  it("sends secure signer profile bindings when selected in the node configuration editor", () => {
    const payload = normalizeNodeUpsertPayload({
      name: "Protected Validator",
      keyProtectionMode: "secure-signer",
      secureSignerProfileId: " signer-prod ",
    });

    expect(payload).toEqual({
      name: "Protected Validator",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-prod",
        },
      },
    });
  });

  it("sends standard key protection when switching away from secure signer", () => {
    const payload = normalizeNodeUpsertPayload({
      keyProtectionMode: "standard",
      secureSignerProfileId: "previous-signer",
    });

    expect(payload).toEqual({
      settings: {
        keyProtection: {
          mode: "standard",
        },
      },
    });
  });

  it("preserves existing standard/local key protection policy fields when saving unrelated settings", () => {
    const payload = normalizeNodeUpsertPayload(
      {
        keyProtectionMode: "standard",
        maxConnections: "32",
      },
      {
        existingKeyProtection: {
          mode: "standard",
          accountAddress: "Nabcd",
          walletPath: "/secure/wallet.json",
          unlockMode: "tee",
          policy: { allowedOperations: ["vote", "claimGas"] },
        },
      },
    );

    expect(payload).toEqual({
      settings: {
        maxConnections: 32,
        keyProtection: {
          mode: "standard",
          accountAddress: "Nabcd",
          walletPath: "/secure/wallet.json",
          unlockMode: "tee",
          policy: { allowedOperations: ["vote", "claimGas"] },
        },
      },
    });
  });

  it("requires a secure signer profile before enabling secure signer mode", () => {
    expect(() => normalizeNodeUpsertPayload({ keyProtectionMode: "secure-signer" })).toThrow(
      /secure signer profile/i,
    );
  });

  it("preserves existing secure signer policy fields when only the selected profile is round-tripped", () => {
    const payload = normalizeNodeUpsertPayload(
      {
        keyProtectionMode: "secure-signer",
        secureSignerProfileId: "signer-prod",
      },
      {
        existingKeyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-prod",
          policy: {
            allowedContracts: ["0xabc"],
            maxGasPerTransaction: "1.5",
            requireHardwareProtection: true,
          },
          signClient: { account: "validator" },
        },
      },
    );

    expect(payload).toEqual({
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-prod",
          policy: {
            allowedContracts: ["0xabc"],
            maxGasPerTransaction: "1.5",
            requireHardwareProtection: true,
          },
          signClient: { account: "validator" },
        },
      },
    });
  });

  it("hydrates form key protection values from an existing node", () => {
    const values = toNodeFormValues({
      name: "Protected Validator",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-prod",
        },
      },
    });

    expect(values.keyProtectionMode).toBe("secure-signer");
    expect(values.secureSignerProfileId).toBe("signer-prod");
  });
});
