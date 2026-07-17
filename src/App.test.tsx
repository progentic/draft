import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const useRuntimeStatusMock = vi.hoisted(() => vi.fn());
const useConnectivityModeMock = vi.hoisted(() => vi.fn());
const setConnectivityModeMock = vi.hoisted(() => vi.fn());
const refreshConnectivityMock = vi.hoisted(() => vi.fn());
const createUnsavedDocumentMock = vi.hoisted(() => vi.fn());
const listenToNativeMenuActionsMock = vi.hoisted(() => vi.fn());
const setNativeMenuStateMock = vi.hoisted(() => vi.fn());
const listenToApplicationOpenRequestsMock = vi.hoisted(() => vi.fn());
const takeApplicationOpenRequestMock = vi.hoisted(() => vi.fn());
const setWindowTitleMock = vi.hoisted(() => vi.fn());

vi.mock("./ipc/documentCreate", () => ({
  createUnsavedDocument: createUnsavedDocumentMock,
}));

vi.mock("./features/runtime-status/useRuntimeStatus", () => ({
  useRuntimeStatus: useRuntimeStatusMock,
}));

vi.mock("./features/connectivity/useConnectivityMode", () => ({
  useConnectivityMode: useConnectivityModeMock,
}));

vi.mock("./ipc/nativeMenu", () => ({
  listenToNativeMenuActions: listenToNativeMenuActionsMock,
  setNativeMenuState: setNativeMenuStateMock,
}));

vi.mock("./ipc/applicationOpen", () => ({
  dismissApplicationOpenRequest: vi.fn(async () => true),
  listenToApplicationOpenRequests: listenToApplicationOpenRequestsMock,
  takeApplicationOpenRequest: takeApplicationOpenRequestMock,
}));

vi.mock("./ipc/windowTitle", () => ({
  setWindowTitle: setWindowTitleMock,
}));

import { App } from "./App";
import type { RuntimeConnectionState } from "./features/runtime-status/runtimeStatusSession";

type RuntimeCommandFailureCode = Extract<
  Extract<RuntimeConnectionState, { phase: "unavailable" }>["reason"],
  { type: "command" }
>["code"];

const RUNTIME_COMMAND_FAILURE_LABELS = {
  invalid_application_version: "DRAFT received an unsupported application version.",
  invalid_build_metadata: "DRAFT could not verify this application build.",
  event_delivery_failed: "DRAFT could not deliver the core status event.",
} satisfies Record<RuntimeCommandFailureCode, string>;

describe("DRAFT workspace shell", () => {
  beforeEach(() => {
    createUnsavedDocumentMock.mockReset();
    createUnsavedDocumentMock.mockResolvedValue({
      status: "created",
      envelope: initialEnvelope(),
    });
    useRuntimeStatusMock.mockReset();
    useRuntimeStatusMock.mockReturnValue({
      buildCommit: "0123456789abcdef0123456789abcdef01234567",
      buildProfile: "release",
      phase: "ready",
      version: "0.1.0",
    });
    setConnectivityModeMock.mockReset();
    refreshConnectivityMock.mockReset();
    useConnectivityModeMock.mockReset();
    useConnectivityModeMock.mockReturnValue({
      state: { phase: "ready", mode: "online" },
      refresh: refreshConnectivityMock,
      setMode: setConnectivityModeMock,
    });
    listenToNativeMenuActionsMock.mockReset();
    listenToNativeMenuActionsMock.mockResolvedValue(vi.fn());
    setNativeMenuStateMock.mockReset();
    setNativeMenuStateMock.mockResolvedValue({ status: "applied" });
    listenToApplicationOpenRequestsMock.mockReset();
    listenToApplicationOpenRequestsMock.mockResolvedValue(vi.fn());
    takeApplicationOpenRequestMock.mockReset();
    takeApplicationOpenRequestMock.mockResolvedValue({ status: "none" });
    setWindowTitleMock.mockReset();
    setWindowTitleMock.mockResolvedValue({ status: "applied" });
  });

  it("renders the editor, navigation, and session state", async () => {
    render(<App />);
    await screen.findByText("Not saved");

    expect(screen.getByRole("main", { name: "DRAFT workspace" })).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Document editor" })).toBeTruthy();
    expect(screen.getByRole("toolbar", { name: "Text formatting" })).toBeTruthy();
    expect(screen.getByRole("complementary", { name: "Document outline" })).toBeTruthy();
    expect(screen.getByRole("complementary", { name: "Document details" })).toBeTruthy();
    const header = document.querySelector<HTMLElement>(".workspace-header")!;
    expect(within(header).getByText("Untitled document")).toBeTruthy();
    expect(within(header).getByText("Unsaved")).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toBe("");
    expect(screen.getByText("Not saved")).toBeTruthy();
    expect(await screen.findByText("v0.1.0 · 01234567")).toBeTruthy();
    await waitFor(() => {
      expect(setWindowTitleMock).toHaveBeenLastCalledWith({
        displayName: "Untitled document",
        unsaved: true,
      });
    });
  });

  it("keeps document and connectivity state in the bottom status bar", async () => {
    render(<App />);
    await screen.findByText("Not saved");

    const header = document.querySelector<HTMLElement>(".workspace-header")!;
    const status = screen.getByLabelText("Workspace status");

    expect(within(header).queryByText("Not saved")).toBeNull();
    expect(within(header).queryByText("Online")).toBeNull();
    expect(within(status).getByText("Not saved")).toBeTruthy();
    expect(within(status).getByText("Online")).toBeTruthy();
    expect(within(status).getByText("Ready")).toBeTruthy();
  });

  it("places the workspace title before panel headings", async () => {
    render(<App />);
    await screen.findByText("Not saved");

    const workspaceTitles = screen.getAllByRole("heading", { level: 1, name: "DRAFT" });
    const outlineTitle = screen.getByRole("heading", { level: 2, name: "Outline" });
    const editor = screen.getByRole("textbox", { name: "Document editor" });

    expect(workspaceTitles).toHaveLength(1);
    const productMark = workspaceTitles[0]?.querySelector<HTMLImageElement>("img.wordmark__mark");
    expect(productMark?.getAttribute("src")).toContain("32x32.png");
    expect(productMark?.getAttribute("alt")).toBe("");
    expect(workspaceTitles[0]?.compareDocumentPosition(outlineTitle)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(within(editor).queryByRole("heading", { level: 1 })).toBeNull();
  });

  it("toggles the document outline without changing document state", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Not saved");

    const toggle = screen.getByRole("button", { name: "Close outline" });
    const outline = screen.getByRole("complementary", { name: "Document outline" });

    await user.click(toggle);

    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    expect(outline.getAttribute("aria-hidden")).toBe("true");
    expect(outline.hasAttribute("inert")).toBe(true);
    expect(screen.getByTestId("workspace-body").className).toContain(
      "workspace-body--outline-closed",
    );
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toBe("");
  });

  it("exposes working Tiptap formatting controls", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Not saved");

    const boldButton = screen.getByRole("button", { name: "Bold" });
    expect(boldButton.getAttribute("aria-pressed")).toBe("false");

    await user.click(boldButton);

    expect(boldButton.getAttribute("aria-pressed")).toBe("true");
  });

  it("applies bounded font controls and preserves control focus", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Not saved");

    const family = screen.getByRole("combobox", { name: "Font family" });
    const size = screen.getByRole("combobox", { name: "Font size in points" });
    const editor = screen.getByRole("textbox", { name: "Document editor" });
    expect((family as HTMLSelectElement).value).toBe("georgia");
    expect((size as HTMLSelectElement).value).toBe("13");
    await user.selectOptions(family, "arial");
    await waitFor(() => expect(document.activeElement).toBe(editor));
    expect((family as HTMLSelectElement).value).toBe("arial");
    await user.selectOptions(size, "18");
    await waitFor(() => expect(document.activeElement).toBe(editor));
    expect((size as HTMLSelectElement).value).toBe("18");
    await user.selectOptions(family, "__document_default__");
    await waitFor(() => expect((family as HTMLSelectElement).value).toBe("georgia"));
    await user.selectOptions(size, "__document_default__");
    await waitFor(() => expect((size as HTMLSelectElement).value).toBe("13"));
  });

  it("exposes a horizontal formatting toolbar with one Tab entry", async () => {
    render(<App />);
    await screen.findByText("Not saved");

    const toolbar = screen.getByRole("toolbar", { name: "Text formatting" });
    const undoButton = screen.getByRole("button", { name: "Undo" });
    const boldButton = screen.getByRole("button", { name: "Bold" });
    const enabledButtons = Array.from(
      toolbar.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"),
    );

    expect(toolbar.getAttribute("aria-orientation")).toBe("horizontal");
    expect(enabledButtons.filter((button) => button.tabIndex === 0)).toEqual([boldButton]);
    expect(undoButton.hasAttribute("aria-pressed")).toBe(false);
  });

  it("moves toolbar focus with horizontal navigation keys", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Not saved");

    const boldButton = screen.getByRole("button", { name: "Bold" });
    const italicButton = screen.getByRole("button", { name: "Italic" });
    const formattingReviewButton = screen.getByRole("button", { name: "Formatting review" });
    const editor = screen.getByRole("textbox", { name: "Document editor" });

    await waitFor(() => expect(document.activeElement).toBe(editor));
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
    await screen.findByText("Not saved");

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

  it("exposes the Rust-owned session connectivity toggle", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Not saved");

    const toggle = screen.getByRole("button", { name: "Work offline" });
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    expect(toggle.textContent).toContain("Online");

    await user.click(toggle);
    expect(setConnectivityModeMock).toHaveBeenCalledWith("offline");
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
  ] as const)("shows the bounded $reason.type failure state", async (reason, expectedLabel) => {
    useRuntimeStatusMock.mockReturnValue({ phase: "unavailable", reason });

    render(<App />);
    await screen.findByText("Not saved");

    const message = screen.getByText(expectedLabel);
    expect(message.closest('[role="status"]')?.getAttribute("aria-atomic")).toBe("true");
  });
});

function initialEnvelope() {
  return {
    schema_version: 2,
    document_id: "00000000-0000-4000-8000-000000000001",
    title: "Untitled document",
    document: {
      type: "doc",
      content: [{ type: "paragraph" }],
    },
  };
}
