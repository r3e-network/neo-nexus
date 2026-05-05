import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { SettingsPanel } from "../src/pages/Agent";

describe("Agent SettingsPanel", () => {
  it("renders edit mode with the current redacted key and cancel affordance", () => {
    const html = renderToStaticMarkup(
      <SettingsPanel
        token="test-token"
        initialSettings={{
          configured: true,
          provider: "anthropic",
          model: "claude-sonnet-4-6",
          apiKey: "••••••••...cret",
          baseUrl: null,
          enabled: true,
        }}
        onCancel={vi.fn()}
        onSaved={vi.fn()}
      />,
    );

    expect(html).toContain("Edit Hermes Agent");
    expect(html).toContain("value=\"••••••••...cret\"");
    expect(html).toContain("Keep the redacted value to preserve the current key.");
    expect(html).toContain("Cancel");
    expect(html).toContain("Save changes");
  });

  it("requires a base URL when editing an OpenAI-compatible provider", () => {
    const html = renderToStaticMarkup(
      <SettingsPanel
        token="test-token"
        initialSettings={{
          configured: true,
          provider: "openai-compatible",
          model: "mistral-large-latest",
          apiKey: "••••••••...cret",
          baseUrl: "",
          enabled: true,
        }}
        onSaved={vi.fn()}
      />,
    );

    expect(html).toContain("Base URL");
    expect(html).toContain("/v1/chat/completions");
    expect(html).toContain("disabled");
  });
});
