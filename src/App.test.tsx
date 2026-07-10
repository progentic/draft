import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const useRuntimeStatusMock = vi.hoisted(() => vi.fn());

vi.mock("./features/runtime-status/useRuntimeStatus", () => ({
  useRuntimeStatus: useRuntimeStatusMock,
}));

import { App } from "./App";
import type { RuntimeConnectionState } from "./features/runtime-status/runtimeStatusSession";

type RuntimeCommandFailureCode = Extract<
  Extract<RuntimeConnectionState, { phase: "unavailable" }>["reason"],
  { type: "command" }
>["code"];

const RUNTIME_COMMAND_FAILURE_LABELS = {
  invalid_application_version: "DRAFT received an unsupported application version.",
  event_delivery_failed: "DRAFT could not deliver the core status event.",
} satisfies Record<RuntimeCommandFailureCode, string>;

describe("DRAFT workspace shell", () => {
  beforeEach(() => {
    useRuntimeStatusMock.mockReset();
    useRuntimeStatusMock.mockReturnValue({ phase: "ready", version: "0.1.0" });
  });

  it("renders the editor, navigation, and session state", async () => {
    render(<App />);

    expect(screen.getByRole("main", { name: "DRAFT workspace" })).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Document editor" })).toBeTruthy();
    expect(screen.getByRole("toolbar", { name: "Text formatting" })).toBeTruthy();
    expect(screen.getByRole("complementary", { name: "Document outline" })).toBeTruthy();
    expect(screen.getByRole("complementary", { name: "Document details" })).toBeTruthy();
    expect(screen.getAllByText("Untitled document").length).toBeGreaterThan(1);
    expect(screen.getByText("Not saved")).toBeTruthy();
    expect(await screen.findByText("Core v0.1.0")).toBeTruthy();
  });

  it("places the workspace title before panel headings", () => {
    render(<App />);

    const workspaceTitles = screen.getAllByRole("heading", { level: 1, name: "DRAFT" });
    const outlineTitle = screen.getByRole("heading", { level: 2, name: "Outline" });
    const editor = screen.getByRole("textbox", { name: "Document editor" });

    expect(workspaceTitles).toHaveLength(1);
    expect(workspaceTitles[0]?.compareDocumentPosition(outlineTitle)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(
      within(editor).getByRole("heading", { level: 1, name: "Untitled document" }),
    ).toBeTruthy();
  });

  it("toggles the document outline without changing document state", async () => {
    const user = userEvent.setup();
    render(<App />);

    const toggle = screen.getByRole("button", { name: "Close outline" });
    const outline = screen.getByRole("complementary", { name: "Document outline" });

    await user.click(toggle);

    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    expect(outline.getAttribute("aria-hidden")).toBe("true");
    expect(outline.hasAttribute("inert")).toBe(true);
    expect(screen.getByTestId("workspace-body").className).toContain(
      "workspace-body--outline-closed",
    );
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toContain(
      "Begin writing here.",
    );
  });

  it("exposes working Tiptap formatting controls", async () => {
    const user = userEvent.setup();
    render(<App />);

    const boldButton = screen.getByRole("button", { name: "Bold" });
    expect(boldButton.getAttribute("aria-pressed")).toBe("false");

    await user.click(boldButton);

    expect(boldButton.getAttribute("aria-pressed")).toBe("true");
  });

  it("exposes a horizontal formatting toolbar with one Tab entry", async () => {
    const user = userEvent.setup();
    render(<App />);

    const toolbar = screen.getByRole("toolbar", { name: "Text formatting" });
    const outlineEntry = screen.getByRole("button", { name: "H1Untitled document" });
    const undoButton = screen.getByRole("button", { name: "Undo" });
    const boldButton = screen.getByRole("button", { name: "Bold" });
    const enabledButtons = Array.from(
      toolbar.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"),
    );

    expect(toolbar.getAttribute("aria-orientation")).toBe("horizontal");
    expect(enabledButtons.filter((button) => button.tabIndex === 0)).toEqual([boldButton]);
    expect(undoButton.hasAttribute("aria-pressed")).toBe(false);

    outlineEntry.focus();
    await user.tab();
    expect(document.activeElement).toBe(boldButton);
  });

  it("moves toolbar focus with horizontal navigation keys", async () => {
    const user = userEvent.setup();
    render(<App />);

    const boldButton = screen.getByRole("button", { name: "Bold" });
    const italicButton = screen.getByRole("button", { name: "Italic" });
    const formattingReviewButton = screen.getByRole("button", { name: "Formatting review" });

    boldButton.focus();
    await user.keyboard("{ArrowRight}");
    expect(document.activeElement).toBe(italicButton);

    await user.keyboard("{End}");
    expect(document.activeElement).toBe(formattingReviewButton);

    await user.keyboard("{ArrowRight}");
    expect(document.activeElement).toBe(boldButton);

    await user.keyboard("{ArrowLeft}");
    expect(document.activeElement).toBe(formattingReviewButton);

    await user.keyboard("{Home}");
    expect(document.activeElement).toBe(boldButton);
  });

  it("opens and closes the formatting review from its toolbar control", async () => {
    const user = userEvent.setup();
    render(<App />);

    const toggle = screen.getByRole("button", { name: "Formatting review" });
    const panel = document.getElementById("formatting-review-panel")!;
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(panel.hidden).toBe(true);

    await user.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    expect(panel.hidden).toBe(false);

    await user.click(screen.getByRole("button", { name: "Close formatting review" }));
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(panel.hidden).toBe(true);
  });

  it.each([
    [{ type: "transport" }, "Core unavailable"],
    [{ type: "invalid-payload" }, "Core status invalid"],
    [{ type: "invalid-response" }, "Core status invalid"],
    ...Object.entries(RUNTIME_COMMAND_FAILURE_LABELS).map(([code, label]) => [
      { type: "command", code },
      label,
    ] as const),
    [
      { type: "command", code: "unknown_command_failure" },
      "DRAFT could not read the core status.",
    ],
  ] as const)("shows the bounded $reason.type failure state", (reason, expectedLabel) => {
    useRuntimeStatusMock.mockReturnValueOnce({ phase: "unavailable", reason });

    render(<App />);

    expect(screen.getByText(expectedLabel)).toBeTruthy();
  });
});
