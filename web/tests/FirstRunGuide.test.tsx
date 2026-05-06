import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it } from "vitest";
import { FirstRunGuide } from "../src/components/FirstRunGuide";

describe("FirstRunGuide", () => {
  it("turns an empty fleet into a guided first-node workflow", () => {
    const html = renderToStaticMarkup(
      <MemoryRouter>
        <FirstRunGuide />
      </MemoryRouter>,
    );

    expect(html).toContain("Start with a safe node");
    expect(html).toContain("Create native node");
    expect(html).toContain("Plan private network");
    expect(html).toContain("Import observe-only");
    expect(html).toContain("Choose a role preset");
    expect(html).toContain("Pick storage and sync");
    expect(html).toContain("/nodes/create");
    expect(html).toContain("/private-networks");
    expect(html).toContain("/nodes/import");
  });
});
