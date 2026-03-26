import { describe, expect, it } from "vitest";
import { countProtectedNodes, getNodeProtectionLabel } from "../../web/src/utils/signerVisibility";

describe("signer visibility helpers", () => {
  it("counts only nodes using secure signer protection", () => {
    expect(
      countProtectedNodes([
        { settings: {} },
        { settings: { keyProtection: { mode: "secure-signer" } } },
        { settings: { keyProtection: { mode: "standard" } } },
        { settings: { keyProtection: { mode: "secure-signer" } } },
      ] as any),
    ).toBe(2);
  });

  it("formats node protection labels for operator-facing views", () => {
    expect(getNodeProtectionLabel({ settings: {} } as any)).toEqual({
      label: "Standard",
      tone: "muted",
    });

    expect(
      getNodeProtectionLabel({
        settings: {
          keyProtection: {
            mode: "secure-signer",
            signerName: "Nitro Council",
          },
        },
      } as any),
    ).toEqual({
      label: "Secure Signer",
      detail: "Nitro Council",
      tone: "secure",
    });
  });
});
