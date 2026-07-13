import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
const useRuntimeStatusMock = vi.hoisted(() => vi.fn());
const useConnectivityModeMock = vi.hoisted(() => vi.fn());
const listenToNativeMenuActionsMock = vi.hoisted(() => vi.fn());
const setNativeMenuStateMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../features/runtime-status/useRuntimeStatus", () => ({
  useRuntimeStatus: useRuntimeStatusMock,
}));
vi.mock("../features/connectivity/useConnectivityMode", () => ({
  useConnectivityMode: useConnectivityModeMock,
}));
vi.mock("./nativeMenu", () => ({
  listenToNativeMenuActions: listenToNativeMenuActionsMock,
  setNativeMenuState: setNativeMenuStateMock,
}));

import { App } from "../App";
import { FINDING_POLICIES } from "./textAnalysis";

const OPENED_ID = "00000000-0000-4000-8000-000000000002";
const CREATED_ID = "00000000-0000-4000-8000-000000000001";
const IMPORTED_ID = "00000000-0000-4000-8000-000000000003";

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
    listenToNativeMenuActionsMock.mockReset();
    listenToNativeMenuActionsMock.mockResolvedValue(vi.fn());
    setNativeMenuStateMock.mockReset();
    setNativeMenuStateMock.mockResolvedValue({ status: "applied" });
    installDefaultCommands();
  });

  it("saves, closes, and reopens through Rust-owned lifecycle commands", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Saved as Research notes.draft.")).toBeTruthy();
    expect(screen.getByText("Research notes.draft")).toBeTruthy();
    expect(screen.getByText("Saved")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Close" }));
    expect(await screen.findByText("Document closed.")).toBeTruthy();
    expect(screen.getAllByText("No document open")).toHaveLength(2);

    await user.click(screen.getByRole("button", { name: "Open…" }));
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
    const openButton = screen.getByRole("button", { name: "Open…" });
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
    await user.click(screen.getByRole("button", { name: "Open…" }));
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
    await assertDocumentActionsDisabled(user, "Export DOCX…");
    expect(commandNames()).toEqual(["create_document", "save_document"]);

    await act(async () => pending.resolve({ status: "cancelled" }));
    expect(await screen.findByText("Save cancelled. Your document remains unsaved.")).toBeTruthy();
    await waitFor(() => expect(setNativeMenuStateMock).toHaveBeenLastCalledWith({
      canNew: true,
      canOpen: true,
      canClose: true,
      canSave: true,
      canSaveAs: true,
      canExport: true,
    }));
  });

  it("prevents overlapping actions while Open is pending", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ openDocument: () => pending.promise });
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Open…" }));
    expect(screen.getByText("Opening")).toBeTruthy();
    await assertDocumentActionsDisabled(user, "Export DOCX…");
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
        return {
          status: "saved",
          documentId: savedEnvelope.document_id,
          displayName: "Styled notes.draft",
          wasSaveAs: true,
        };
      },
      openDocument: async () => ({ status: "opened_draft", envelope: savedEnvelope }),
    });
    render(<App />);

    await user.selectOptions(screen.getByRole("combobox", { name: "Font family" }), "avenir_next");
    await user.selectOptions(screen.getByRole("combobox", { name: "Font size in points" }), "19");
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Document editor" }));
    });
    await user.keyboard("Styled ");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(JSON.stringify(savedEnvelope)).toContain('"family":"avenir_next"');
    expect(JSON.stringify(savedEnvelope)).toContain('"points":19');
    await user.click(screen.getByRole("button", { name: "Close" }));
    await user.click(screen.getByRole("button", { name: "Open…" }));

    const editor = screen.getByRole("textbox", { name: "Document editor" });
    expect(editor.querySelector('[data-draft-font-family="avenir_next"]')).toBeTruthy();
    expect(editor.querySelector('[data-draft-font-size="19"]')).toBeTruthy();
    expect((screen.getByRole("combobox", { name: "Font family" }) as HTMLSelectElement).value).toBe("avenir_next");
    expect((screen.getByRole("combobox", { name: "Font size in points" }) as HTMLSelectElement).value).toBe("19");
  });

  it("creates a focused blank document with a caret ready for typing", async () => {
    const user = userEvent.setup();
    render(<App />);
    const editor = screen.getByRole("textbox", { name: "Document editor" });
    await waitFor(() => expect(document.activeElement).toBe(editor));
    await user.type(editor, "Temporary text");
    await user.click(screen.getByRole("button", { name: "New Document" }));
    await user.click(screen.getByRole("button", { name: "Discard changes" }));

    await waitFor(() => expect(document.activeElement).toBe(editor));
    expect(editor.textContent).toBe("");
    expect(window.getSelection()?.anchorOffset).toBe(0);
    expect(commandNames()).toEqual(["create_document", "create_document"]);
  });

  it("keeps an imported filename display-only through first and later saves", async () => {
    const user = userEvent.setup();
    const saveRequests: Record<string, unknown>[] = [];
    installDefaultCommands({
      openDocument: async () => ({ status: "imported_text", envelope: importedEnvelope() }),
      saveDocument: async (args) => {
        saveRequests.push(args);
        return {
          status: "saved",
          documentId: IMPORTED_ID,
          displayName: "Imported notes.draft",
          wasSaveAs: saveRequests.length === 1,
        };
      },
    });
    render(<App />);
    const editor = screen.getByRole("textbox", { name: "Document editor" });

    await user.click(screen.getByRole("button", { name: "Open…" }));
    expect(await screen.findByText("Text imported. Save as a DRAFT document to keep your work.")).toBeTruthy();
    expect(screen.getByText("Imported, unsaved")).toBeTruthy();
    expect(screen.getByText("notes.md")).toBeTruthy();
    expect(editor.textContent).toContain("# Literal Markdown");

    await user.click(screen.getByRole("button", { name: "Save" }));
    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(screen.getByText("Saved")).toBeTruthy();
    expect(screen.getByText("Imported notes.draft")).toBeTruthy();
    expect(saveRequests).toHaveLength(2);
    expect(Object.keys(saveRequests[0] ?? {})).toEqual(["request"]);
    expect(JSON.stringify(saveRequests)).not.toContain("sourcePath");
    expect(JSON.stringify(saveRequests)).not.toContain("source_path");
  });

  it("keeps an imported session unsaved when its first Save is cancelled", async () => {
    const user = userEvent.setup();
    installDefaultCommands({
      openDocument: async () => ({ status: "imported_text", envelope: importedEnvelope() }),
      saveDocument: async () => ({ status: "cancelled" }),
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open…" }));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("Save cancelled. Your document remains unsaved.")).toBeTruthy();
    expect(screen.getByText("Imported, unsaved")).toBeTruthy();
    expect(screen.getByText("notes.md")).toBeTruthy();
  });

  it("keeps the saved filename and dirty state when a later Save fails", async () => {
    const user = userEvent.setup();
    let saves = 0;
    installDefaultCommands({
      saveDocument: async () => {
        if (saves++ === 0) {
          return {
            status: "saved",
            documentId: CREATED_ID,
            displayName: "Research notes.draft",
            wasSaveAs: true,
          };
        }
        throw { code: "write_failed", cause: { code: "replace_target" } };
      },
    });
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Saved as Research notes.draft.")).toBeTruthy();
    const editor = screen.getByRole("textbox", { name: "Document editor" });
    await user.type(editor, "Unsaved revision");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("DRAFT could not save the document. Your open document has not been replaced.")).toBeTruthy();
    expect(screen.getByText("Research notes.draft")).toBeTruthy();
    expect(screen.getByText("Unsaved changes")).toBeTruthy();
  });

  it("uses Save As once and reuses the Rust-owned replacement target", async () => {
    const user = userEvent.setup();
    const modes: string[] = [];
    installDefaultCommands({
      saveDocument: async (args) => {
        const request = args.request as {
          mode: string;
          snapshot: { document_id: string };
        };
        modes.push(request.mode);
        return {
          status: "saved",
          documentId: request.snapshot.document_id,
          displayName: request.mode === "save_as" ? "Archive.draft" : "Research notes.draft",
          wasSaveAs: request.mode === "save_as" || modes.length === 1,
        };
      },
    });
    render(<App />);
    await screen.findByText("Not saved");

    await user.click(screen.getByRole("button", { name: "Save" }));
    await screen.findByText("Research notes.draft");
    await clickWorkspaceAction(user, "Save As…");
    await screen.findByText("Archive.draft");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(modes).toEqual(["save", "save_as", "save"]));
  });

  it("rejects a non-DRAFT first-save target without changing import state", async () => {
    const user = userEvent.setup();
    installDefaultCommands({
      openDocument: async () => ({ status: "imported_text", envelope: importedEnvelope() }),
      saveDocument: async () => {
        throw { code: "invalid_target" };
      },
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open…" }));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("Choose a .draft file name. Your document remains unsaved.")).toBeTruthy();
    expect(screen.getByText("Imported, unsaved")).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toContain(
      "# Literal Markdown",
    );
  });

  it("preserves native content, selection, title, and state when Open is cancelled", async () => {
    const user = userEvent.setup();
    let opens = 0;
    installDefaultCommands({
      openDocument: async () =>
        opens++ === 0
          ? { status: "opened_draft", envelope: openedEnvelope() }
          : { status: "cancelled" },
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open…" }));
    const editor = screen.getByRole("textbox", { name: "Document editor" });
    editor.focus();
    selectText(editor.querySelector("p")?.firstChild, 2, 8);
    document.dispatchEvent(new Event("selectionchange"));

    fireEvent.click(screen.getByRole("button", { name: "Open…" }));
    expect(await screen.findByText("Open cancelled.")).toBeTruthy();
    expect(editor.textContent).toContain("Persisted evidence");
    expect(screen.getAllByText("Reopened research notes")).toHaveLength(3);
    expect(screen.getByText("Saved")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Bold" }));
    expect(editor.querySelector("strong")?.textContent).toBe("rsiste");
  });

  it("preserves the open document when a typed Open failure is presented", async () => {
    const user = userEvent.setup();
    let opens = 0;
    installDefaultCommands({
      openDocument: async () => {
        if (opens++ === 0) {
          return { status: "opened_draft", envelope: openedEnvelope() };
        }
        throw { code: "file_not_found" };
      },
    });
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open…" }));
    await user.click(screen.getByRole("button", { name: "Open…" }));

    expect(await screen.findByText("That file is no longer available. Choose another document.")).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "Document editor" }).textContent).toContain(
      "Persisted evidence",
    );
    expect(screen.getByText("Saved")).toBeTruthy();
  });

  it("preserves unsaved content when a later New command fails", async () => {
    const user = userEvent.setup();
    let creates = 0;
    installDefaultCommands({
      createDocument: async () => {
        if (creates++ === 0) {
          return { status: "created", envelope: createdEnvelope() };
        }
        throw { code: "template_invalid" };
      },
    });
    render(<App />);
    const editor = screen.getByRole("textbox", { name: "Document editor" });
    await user.type(editor, "Keep this text");
    await user.click(screen.getByRole("button", { name: "New Document" }));
    await user.click(screen.getByRole("button", { name: "Discard changes" }));

    expect(await screen.findByText("DRAFT could not create a document. Try again.")).toBeTruthy();
    expect(editor.textContent).toContain("Keep this text");
    expect(screen.getByText("Unsaved changes")).toBeTruthy();
  });

  it("adds a reference and inserts a resolvable citation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByRole("textbox", { name: "Document editor" }));
    });
    await clickWorkspaceAction(user, "References");
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
    await user.type(screen.getByRole("textbox", { name: "Document editor" }), "Word word.");
    await clickWorkspaceAction(user, "Text checks");
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
    await clickWorkspaceAction(user, "Text checks");
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
    await user.type(screen.getByRole("textbox", { name: "Document editor" }), "Text to check.");
    await clickWorkspaceAction(user, "Text checks");
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
    await clickWorkspaceAction(user, "Text checks");
    await user.click(screen.getByRole("button", { name: "Check document" }));
    await user.click(screen.getByRole("button", { name: "Heading 2" }));
    await act(async () => pending.resolve({ result: { findings: [finding()] } }));

    expect(screen.getByText("The document changed. Run text checks again.")).toBeTruthy();
    expect(screen.queryByText("Repeated word")).toBeNull();
  });

  it("exports DOCX with visible completion and source-safety feedback", async () => {
    const user = userEvent.setup();
    render(<App />);
    await clickWorkspaceAction(user, "Export DOCX…");

    expect(await screen.findByText("DOCX export complete. Your DRAFT document was not changed.")).toBeTruthy();
    expect(commandNames()).toEqual(["create_document", "export_document"]);
  });

  it("prevents overlapping document actions while DOCX export is pending", async () => {
    const user = userEvent.setup();
    const pending = deferred<unknown>();
    installDefaultCommands({ exportDocument: () => pending.promise });
    render(<App />);

    await clickWorkspaceAction(user, "Export DOCX…");
    expect(screen.getByText("Preparing DOCX export.")).toBeTruthy();
    await assertDocumentActionsDisabled(user, "Exporting DOCX");
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
      return {
        status: "saved",
        documentId: request.snapshot.document_id,
        displayName: "Research notes.draft",
        wasSaveAs: true,
      };
    }
    if (command === "close_document") {
      const request = args.request as { documentId: string };
      return { status: "closed", documentId: request.documentId };
    }
    if (command === "open_document") {
      return overrides?.openDocument
        ? overrides.openDocument()
        : { status: "opened_draft", envelope: openedEnvelope() };
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
    schema_version: 2,
    document_id: CREATED_ID,
    title: "Untitled document",
    document: {
      type: "doc",
      content: [{ type: "paragraph" }],
    },
  };
}

function importedEnvelope() {
  return {
    schema_version: 2,
    document_id: IMPORTED_ID,
    title: "notes.md",
    document: {
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "# Literal Markdown" }] },
        { type: "paragraph", content: [{ type: "text", text: "**not parsed**" }] },
      ],
    },
  };
}

function selectText(node: ChildNode | null | undefined, start: number, end: number) {
  if (!node) {
    throw new Error("Expected selectable editor text");
  }
  const range = document.createRange();
  range.setStart(node, start);
  range.setEnd(node, end);
  const selection = window.getSelection();
  selection?.removeAllRanges();
  selection?.addRange(range);
}

function openedEnvelope() {
  return {
    schema_version: 2,
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
  for (const label of ["New Document", "Open…", "Save", "Close"]) {
    const button = screen.getByRole("button", { name: label }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    await user.click(button);
  }
  const menu = await openWorkspaceOverflow(user);
  for (const label of ["Save As…", "References", "Text checks", exportLabel]) {
    const button = within(menu).getByText(label).closest("button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    await user.click(button);
  }
}

async function clickWorkspaceAction(
  user: ReturnType<typeof userEvent.setup>,
  label: string,
) {
  const menu = await openWorkspaceOverflow(user);
  await user.click(within(menu).getByText(label).closest("button") as HTMLButtonElement);
}

async function openWorkspaceOverflow(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "More document actions" }));
  return screen.getByRole("menu", { name: "More document actions" });
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
