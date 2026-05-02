import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { ConfirmDialog } from "../src/components/ConfirmDialog";
import { NodeLogsToolbar } from "../src/pages/node-detail/NodeLogsView";

describe("ConfirmDialog", () => {
  const handlers = {
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
  };

  it("renders nothing while closed", () => {
    const html = renderToStaticMarkup(
      <ConfirmDialog
        open={false}
        title="Delete node"
        description="This action cannot be undone."
        onConfirm={handlers.onConfirm}
        onCancel={handlers.onCancel}
      />,
    );

    expect(html).toBe("");
  });

  it("renders an accessible modal with safe defaults", () => {
    const html = renderToStaticMarkup(
      <ConfirmDialog
        open
        title="Delete node"
        description="This action cannot be undone."
        confirmLabel="Delete"
        onConfirm={handlers.onConfirm}
        onCancel={handlers.onCancel}
      />,
    );

    expect(html).toContain('role="dialog"');
    expect(html).toContain('aria-modal="true"');
    expect(html).toContain("Delete node");
    expect(html).toContain("This action cannot be undone.");
    expect(html).toContain("Close confirmation dialog");
    expect(html).toContain("Cancel");
    expect(html).toContain("Delete");
    expect(html).toContain("btn-error");
  });

  it("shows progress text and disables actions while confirming", () => {
    const html = renderToStaticMarkup(
      <ConfirmDialog
        open
        title="Restore configuration"
        description="Current data will be replaced."
        confirmVariant="primary"
        isConfirming
        onConfirm={handlers.onConfirm}
        onCancel={handlers.onCancel}
      />,
    );

    expect(html).toContain("Working...");
    expect(html).toContain("disabled");
    expect(html).toContain("btn-primary");
  });
});

describe("NodeLogsToolbar", () => {
  it("renders controls for pausing live follow, copying, and clearing the local view", () => {
    const html = renderToStaticMarkup(
      <NodeLogsToolbar
        connected
        following
        onToggleFollow={vi.fn()}
        onCopy={vi.fn()}
        onClear={vi.fn()}
      />,
    );

    expect(html).toContain("Pause follow");
    expect(html).toContain("Copy logs");
    expect(html).toContain("Clear view");
  });
});
