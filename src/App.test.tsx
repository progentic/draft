import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const useRuntimeStatusMock = vi.hoisted(() => vi.fn());

vi.mock("./features/runtime-status/useRuntimeStatus", () => ({
  useRuntimeStatus: useRuntimeStatusMock,
}));

import { App } from "./App";

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

  it.each([
    ["transport", "Core unavailable"],
    ["command", "Core event failed"],
    ["invalid-payload", "Core status invalid"],
  ] as const)("shows the bounded %s failure state", (reason, expectedLabel) => {
    useRuntimeStatusMock.mockReturnValueOnce({ phase: "unavailable", reason });

    render(<App />);

    expect(screen.getByText(expectedLabel)).toBeTruthy();
  });
});
