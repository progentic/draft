import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listenMock = vi.hoisted(() => vi.fn());
const setStateMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/nativeMenu", () => ({
  listenToNativeMenuActions: listenMock,
  setNativeMenuState: setStateMock,
}));

import type { DocumentSession } from "../document-session/useDocumentSession";
import type { DocxExportState } from "../docx-export/useDocxExport";
import { useWorkspaceActions } from "./useWorkspaceActions";

describe("workspace action dispatcher", () => {
  beforeEach(() => {
    listenMock.mockReset();
    listenMock.mockResolvedValue(vi.fn());
    setStateMock.mockReset();
    setStateMock.mockResolvedValue({ status: "applied" });
  });

  it("routes toolbar and native menu actions through the same operations", async () => {
    const session = documentSession();
    const docxExport = exportState();
    const togglePanel = vi.fn();
    let nativeAction: ((action: string) => void) | undefined;
    listenMock.mockImplementation(async (onAction) => {
      nativeAction = onAction;
      return vi.fn();
    });
    const { result } = renderHook(() => useWorkspaceActions(session, docxExport, togglePanel));
    await waitFor(() => expect(nativeAction).toBeTypeOf("function"));

    act(() => result.current.dispatch("save_document"));
    act(() => nativeAction?.("save_document"));
    act(() => result.current.dispatch("save_back_to_source"));
    act(() => nativeAction?.("save_back_to_source"));
    act(() => result.current.dispatch("open_references"));

    expect(session.save).toHaveBeenCalledTimes(2);
    expect(session.requestSaveBack).toHaveBeenCalledTimes(2);
    expect(togglePanel).toHaveBeenCalledWith("references");
  });

  it("synchronizes closed state and rejects stale native actions", async () => {
    const session = documentSession();
    let nativeAction: ((action: string) => void) | undefined;
    listenMock.mockImplementation(async (onAction) => {
      nativeAction = onAction;
      return vi.fn();
    });
    const { result, rerender } = renderHook(
      ({ current }) => useWorkspaceActions(current, exportState(), vi.fn()),
      { initialProps: { current: session } },
    );
    await waitFor(() => expect(setStateMock).toHaveBeenCalledWith({
      canNew: true,
      canOpen: true,
      canClose: true,
      canSave: true,
      canSaveAs: true,
      canSaveBack: true,
      canExport: true,
    }));

    rerender({ current: { ...session, operation: "saving" } });
    await waitFor(() => expect(setStateMock).toHaveBeenLastCalledWith({
      canNew: false,
      canOpen: false,
      canClose: false,
      canSave: false,
      canSaveAs: false,
      canSaveBack: false,
      canExport: false,
    }));
    act(() => nativeAction?.("save_document"));

    expect(session.save).not.toHaveBeenCalled();
    expect(result.current.feedback).toBe("Finish the current document action first.");
  });

  it("gives toolbar and native Save Back the same stale-source disposition", async () => {
    const session = {
      ...documentSession(),
      canSaveBack: false,
      saveBackUnavailableReason:
        "The source changed outside DRAFT. Reopen it before saving back.",
    };
    let nativeAction: ((action: string) => void) | undefined;
    listenMock.mockImplementation(async (onAction) => {
      nativeAction = onAction;
      return vi.fn();
    });
    const { result } = renderHook(() =>
      useWorkspaceActions(session, exportState(), vi.fn()),
    );
    await waitFor(() => expect(nativeAction).toBeTypeOf("function"));

    act(() => result.current.dispatch("save_back_to_source"));
    expect(result.current.feedback).toBe(session.saveBackUnavailableReason);
    act(() => nativeAction?.("save_back_to_source"));

    expect(result.current.feedback).toBe(session.saveBackUnavailableReason);
    expect(session.requestSaveBack).not.toHaveBeenCalled();
    await waitFor(() => expect(setStateMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ canSaveBack: false }),
    ));
  });

  it("exposes bounded recovery when native state synchronization fails", async () => {
    setStateMock.mockResolvedValue({
      status: "error",
      error: { type: "command", code: "menu_update_failed" },
    });
    const { result } = renderHook(() =>
      useWorkspaceActions(documentSession(), exportState(), vi.fn()),
    );

    await waitFor(() => expect(result.current.feedback).toBe(
      "DRAFT could not update the native menu. Use the toolbar.",
    ));
  });

  it("handles native listener setup failure without an unhandled rejection", async () => {
    listenMock.mockRejectedValue(new Error("private event transport detail"));
    const { result } = renderHook(() =>
      useWorkspaceActions(documentSession(), exportState(), vi.fn()),
    );

    await waitFor(() => expect(result.current.feedback).toBe(
      "DRAFT could not read native menu actions. Use the toolbar.",
    ));
  });

  it.each([
    [
      "no document",
      { canClose: false, canExport: false, canSave: false, canSaveAs: false, canSaveBack: false },
      false,
      { canNew: true, canOpen: true, canClose: false, canSave: false, canSaveAs: false, canSaveBack: false, canExport: false },
    ],
    [
      "export pending",
      {},
      true,
      { canNew: false, canOpen: false, canClose: false, canSave: false, canSaveAs: false, canSaveBack: false, canExport: false },
    ],
  ] as const)("applies the %s native state", async (_name, sessionPatch, exporting, expected) => {
    renderHook(() => useWorkspaceActions(
      { ...documentSession(), ...sessionPatch },
      { ...exportState(), disabled: exporting },
      vi.fn(),
    ));

    await waitFor(() => expect(setStateMock).toHaveBeenCalledWith(expected));
  });
});

function documentSession(): DocumentSession {
  return {
    canClose: true,
    canExport: true,
    canSave: true,
    canSaveAs: true,
    canSaveBack: true,
    documentId: "00000000-0000-4000-8000-000000000001",
    feedback: "",
    operation: "ready",
    pendingAction: null,
    requestClose: vi.fn(),
    requestNew: vi.fn(),
    requestOpen: vi.fn(),
    requestSaveBack: vi.fn(),
    resolvePendingAction: vi.fn(),
    resolveSaveBack: vi.fn(),
    save: vi.fn().mockResolvedValue(true),
    saveAs: vi.fn().mockResolvedValue(true),
    saveBackConfirmation: null,
    saveBackUnavailableReason: "",
    saveBackVisible: true,
    snapshot: vi.fn(),
    statusLabel: "Saved",
    title: "Research.draft",
  };
}

function exportState(): DocxExportState {
  return {
    disabled: false,
    feedback: "",
    label: "Export DOCX…",
    run: vi.fn(),
  };
}
