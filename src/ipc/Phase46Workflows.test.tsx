import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
const useRuntimeStatusMock = vi.hoisted(() => vi.fn());
const useConnectivityModeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../features/runtime-status/useRuntimeStatus", () => ({
  useRuntimeStatus: useRuntimeStatusMock,
}));
vi.mock("../features/connectivity/useConnectivityMode", () => ({
  useConnectivityMode: useConnectivityModeMock,
}));

import { App } from "../App";
import { FINDING_POLICIES } from "./textAnalysis";

const OPENED_ID = "00000000-0000-4000-8000-000000000002";
const CREATED_ID = "00000000-0000-4000-8000-000000000001";

describe("Phase 46 visible workflows", () => {
  beforeEach(() => {
    installLayoutStubs();
    invokeMock.mockReset();
    useRuntimeStatusMock.mockReturnValue({ phase: "ready", version: "0.1.0" });
    useConnectivityModeMock.mockReturnValue({
      state: { phase: "ready", mode: "online" },
      refresh: vi.fn(),
      setMode: vi.fn(),
    });
    installDefaultCommands();
  });

  it("saves, closes, and reopens through Rust-owned lifecycle commands", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Document saved.")).toBeTruthy();
    expect(screen.getByText("Saved")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Close" }));
    expect(await screen.findByText("Document closed.")).toBeTruthy();
    expect(screen.getAllByText("No document open")).toHaveLength(2);

    await user.click(screen.getByRole("button", { name: "Open" }));
    expect(await screen.findAllByText("Reopened research notes")).toHaveLength(3);
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toContain(
      "Persisted evidence",
    );

    expect(commandNames()).toEqual(["create_document", "save_document", "close_document", "open_document"]);
  });

  it("fails closed when Rust cannot create the initial document", async () => {
    installDefaultCommands({
      createDocument: () => Promise.reject({ code: "template_invalid" }),
    });
    render(<App />);

    expect(await screen.findByText("DRAFT could not create a document. Choose New to try again.")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Save" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getAllByText("No document open")).toHaveLength(2);
    expect(commandNames()).toEqual(["create_document"]);
  });

  it("protects unsaved work with keyboard-contained recovery choices", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Heading 2" }));
    const openButton = screen.getByRole("button", { name: "Open" });
    await user.click(openButton);

    const dialog = screen.getByRole("alertdialog", { name: "Save your changes?" });
    const save = within(dialog).getByRole("button", { name: "Save and continue" });
    const cancel = within(dialog).getByRole("button", { name: "Keep editing" });
    expect(document.activeElement).toBe(save);

    cancel.focus();
    await user.tab();
    expect(document.activeElement).toBe(save);
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(document.activeElement).toBe(openButton);
    expect(commandNames()).toEqual(["create_document"]);
  });

  it("releases a first-save handle before continuing to another document", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Heading 2" }));
    await user.click(screen.getByRole("button", { name: "Open" }));
    await user.click(screen.getByRole("button", { name: "Save and continue" }));

    expect(await screen.findAllByText("Reopened research notes")).toHaveLength(3);
    expect(commandNames()).toEqual(["create_document", "save_document", "open_document", "close_document"]);
  });

  it("prevents overlapping actions while Save is pending", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ saveDocument: () => pending.promise });
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(screen.getByText("Saving")).toBeTruthy();
    await assertDocumentActionsDisabled(user, "Export DOCX");
    expect(commandNames()).toEqual(["create_document", "save_document"]);

    await act(async () => pending.resolve({ status: "cancelled" }));
    expect(await screen.findByText("Save cancelled. Your document remains unsaved.")).toBeTruthy();
  });

  it("prevents overlapping actions while Open is pending", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ openDocument: () => pending.promise });
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Open" }));
    expect(screen.getByText("Opening")).toBeTruthy();
    await assertDocumentActionsDisabled(user, "Export DOCX");
    expect(commandNames()).toEqual(["create_document", "open_document"]);

    await act(async () => pending.resolve({ status: "cancelled" }));
    expect(await screen.findByText("Open cancelled.")).toBeTruthy();
  });

  it("persists and restores bounded font formatting through lifecycle commands", async () => {
    const user = userEvent.setup();
    let savedEnvelope: ReturnType<typeof createdEnvelope> | undefined;
    installDefaultCommands({
      saveDocument: async (args) => {
        savedEnvelope = (args.request as { snapshot: ReturnType<typeof createdEnvelope> }).snapshot;
        return { status: "saved", documentId: savedEnvelope.document_id };
      },
      openDocument: async () => ({ status: "opened", envelope: savedEnvelope }),
    });
    render(<App />);

    await user.selectOptions(screen.getByRole("combobox", { name: "Font family" }), "georgia");
    await user.selectOptions(screen.getByRole("combobox", { name: "Font size in points" }), "19");
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Document editor" }));
    });
    await user.keyboard("Styled ");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(JSON.stringify(savedEnvelope)).toContain('"family":"georgia"');
    expect(JSON.stringify(savedEnvelope)).toContain('"points":19');
    await user.click(screen.getByRole("button", { name: "Close" }));
    await user.click(screen.getByRole("button", { name: "Open" }));

    const editor = screen.getByRole("textbox", { name: "Document editor" });
    expect(editor.querySelector('[data-draft-font-family="georgia"]')).toBeTruthy();
    expect(editor.querySelector('[data-draft-font-size="19"]')).toBeTruthy();
  });

  it("adds a reference and inserts a resolvable citation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "References" }));
    expect(await screen.findAllByText("No references saved yet.")).toHaveLength(2);

    await user.type(screen.getByRole("textbox", { name: "Citekey" }), "ada2026");
    await user.type(screen.getByRole("textbox", { name: "Title" }), "Research Notes");
    await user.type(screen.getByRole("textbox", { name: "Author" }), "Ada Lovelace");
    await user.type(screen.getByRole("spinbutton", { name: "Year" }), "2026");
    await user.click(screen.getByRole("button", { name: "Add reference" }));
    expect(await screen.findByText("Reference ada2026 added.")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Insert citation for Research Notes" }));
    expect(await screen.findByText("[@ada2026]")).toBeTruthy();
    expect(commandNames()).toEqual(["create_document", "list_references", "add_reference", "resolve_citation"]);
  });

  it("shows pending and successful advisory text-check states", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ analysis: () => pending.promise });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Text checks" }));
    await user.click(screen.getByRole("button", { name: "Check document" }));
    expect(screen.getByText("Checking the current document.")).toBeTruthy();

    await act(async () => pending.resolve({ result: { findings: [finding()] } }));
    expect(await screen.findByText("Repeated word")).toBeTruthy();
    expect(screen.getByText("Findings are suggestions, not conclusions.", { exact: false })).toBeTruthy();
    expect(screen.getByRole("status", { name: "1 text check findings" })).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Show in document" }));
    expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Document editor" }));
  });

  it("explains empty text and preserves the available recovery action", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Close" }));
    await user.click(screen.getByRole("button", { name: "Text checks" }));
    await user.click(screen.getByRole("button", { name: "Check document" }));

    expect(
      screen
        .getByText("Write some text before running text checks.")
        .closest('[role="status"]'),
    ).toBeTruthy();
    expect(commandNames()).toEqual(["create_document"]);
    expect(screen.getByRole("button", { name: "Check document" })).toBeTruthy();
  });

  it("announces a typed text-check failure without inventing a workflow", async () => {
    const user = userEvent.setup();
    installDefaultCommands({
      analysis: () => Promise.reject({ code: "runtime_unavailable" }),
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Text checks" }));
    await user.click(screen.getByRole("button", { name: "Check document" }));

    expect(
      (await screen.findByText("Text checks are unavailable in this installation."))
        .closest('[role="status"]'),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Check document" })).toBeTruthy();
    expect(screen.queryByText(/provider|account|sign in/i)).toBeNull();
  });

  it("discards stale text-check results after the document changes", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ analysis: () => pending.promise });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Text checks" }));
    await user.click(screen.getByRole("button", { name: "Check document" }));
    await user.click(screen.getByRole("button", { name: "Heading 2" }));
    await act(async () => pending.resolve({ result: { findings: [finding()] } }));

    expect(screen.getByText("The document changed. Run text checks again.")).toBeTruthy();
    expect(screen.queryByText("Repeated word")).toBeNull();
  });

  it("exports DOCX with visible completion and source-safety feedback", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Export DOCX" }));

    expect(await screen.findByText("DOCX export complete. Your DRAFT document was not changed.")).toBeTruthy();
    expect(commandNames()).toEqual(["create_document", "export_document"]);
  });

  it("prevents overlapping document actions while DOCX export is pending", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ exportDocument: () => pending.promise });
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Export DOCX" }));
    expect(screen.getByText("Preparing DOCX export.")).toBeTruthy();
    for (const label of ["New", "Open", "Save", "References", "Text checks", "Exporting DOCX", "Close"]) {
      const button = screen.getByRole("button", { name: label }) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      await user.click(button);
    }
    expect(commandNames()).toEqual(["create_document", "export_document"]);

    await act(async () => pending.resolve({ status: "cancelled" }));
    expect(await screen.findByText("DOCX export cancelled.")).toBeTruthy();
  });
});

function installDefaultCommands(overrides?: {
  analysis?: () => Promise<unknown>;
  createDocument?: () => Promise<unknown>;
  exportDocument?: () => Promise<unknown>;
  openDocument?: () => Promise<unknown>;
  saveDocument?: (args: Record<string, unknown>) => Promise<unknown>;
}) {
  invokeMock.mockImplementation(async (command: string, args: Record<string, unknown>) => {
    if (command === "create_document") {
      return overrides?.createDocument
        ? overrides.createDocument()
        : { status: "created", envelope: createdEnvelope() };
    }
    if (command === "save_document") {
      if (overrides?.saveDocument) {
        return overrides.saveDocument(args);
      }
      const request = args.request as { snapshot: { document_id: string } };
      return { status: "saved", documentId: request.snapshot.document_id };
    }
    if (command === "close_document") {
      const request = args.request as { documentId: string };
      return { status: "closed", documentId: request.documentId };
    }
    if (command === "open_document") {
      return overrides?.openDocument
        ? overrides.openDocument()
        : { status: "opened", envelope: openedEnvelope() };
    }
    if (command === "list_references") {
      return [];
    }
    if (command === "add_reference") {
      return { citekey: "ada2026", title: "Research Notes" };
    }
    if (command === "resolve_citation") {
      return { schemaVersion: 1, citekey: "ada2026", renderStyle: "apa7", displayMarker: "[@ada2026]" };
    }
    if (command === "run_text_analysis") {
      return overrides?.analysis ? overrides.analysis() : { result: { findings: [] } };
    }
    if (command === "export_document") {
      return overrides?.exportDocument
        ? overrides.exportDocument()
        : { status: "exported", bytesWritten: 2048 };
    }
    throw new Error(`Unexpected command: ${command}`);
  });
}

function createdEnvelope() {
  return {
    schema_version: 1,
    document_id: CREATED_ID,
    title: "Untitled document",
    document: {
      type: "doc",
      content: [
        { type: "heading", attrs: { level: 1 }, content: [{ type: "text", text: "Untitled document" }] },
        { type: "paragraph", content: [{ type: "text", text: "Begin writing here." }] },
      ],
    },
  };
}

function openedEnvelope() {
  return {
    schema_version: 1,
    document_id: OPENED_ID,
    title: "Reopened research notes",
    document: {
      type: "doc",
      content: [
        { type: "heading", attrs: { level: 1 }, content: [{ type: "text", text: "Reopened research notes" }] },
        { type: "paragraph", content: [{ type: "text", text: "Persisted evidence" }] },
      ],
    },
  };
}

function finding() {
  return {
    code: "repeated_word",
    ...FINDING_POLICIES.repeated_word,
    startByte: 0,
    endByte: 8,
  };
}

function commandNames() {
  return invokeMock.mock.calls.map(([command]) => command);
}

async function assertDocumentActionsDisabled(
  user: ReturnType<typeof userEvent.setup>,
  exportLabel: string,
) {
  for (const label of ["New", "Open", "Save", "References", "Text checks", exportLabel, "Close"]) {
    const button = screen.getByRole("button", { name: label }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    await user.click(button);
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolver) => { resolve = resolver; });
  return { promise, resolve };
}

function installLayoutStubs() {
  const rect = {
    bottom: 0,
    height: 0,
    left: 0,
    right: 0,
    top: 0,
    width: 0,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  } as DOMRect;
  Range.prototype.getClientRects = vi.fn(() => [rect] as unknown as DOMRectList);
  Range.prototype.getBoundingClientRect = vi.fn(() => rect);
  document.elementFromPoint = vi.fn(() => document.querySelector(".draft-editor"));
}
