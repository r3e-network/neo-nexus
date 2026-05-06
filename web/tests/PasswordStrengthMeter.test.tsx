import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PasswordStrengthMeter } from "../src/components/PasswordStrengthMeter";

describe("PasswordStrengthMeter", () => {
  it("renders strength, progress, and missing requirements without exposing the password", () => {
    const html = renderToStaticMarkup(<PasswordStrengthMeter password="password" />);

    expect(html).toContain("Password strength");
    expect(html).toContain("Weak");
    expect(html).toContain("Add a number or symbol.");
    expect(html).not.toContain("password");
  });

  it("renders the success state when all credential guidance is satisfied", () => {
    const html = renderToStaticMarkup(<PasswordStrengthMeter password="NeoNexus-Validator-2026!" />);

    expect(html).toContain("Excellent");
    expect(html).toContain("Ready for admin access.");
  });
});
