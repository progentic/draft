import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listenMock = vi.hoisted(() => vi.fn());
const setStateMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/nativeMenu", () => ({
  listenToNativeMenuActions: listenMock,
  setNativeMenuState: setStateMock,
}));

import type { DocumentSession } from "../document-session/useDocumentSession";
import { useWorkspaceActions } from "./useWorkspaceActions";

describe("workspace action dispatcher", () => {
  beforeEach(() => {
    listenMock.mockReset();
    listenMock.mockResolvedValue(vi.fn());
    setStateMock.mockReset();
    setStateMock.mockResolvedValue({ status: "applied" });
  });

  it("routes toolbar and native Save As through one format workflow", async () => {
    const session = documentSession();
    let nativeAction: ((action: string) => void) | undefined;
    listenMock.mockImplementation(async (onAction) => {
      nativeAction = onAction;
      return vi.fn();
    });
    const { result } = renderHook(() => useWorkspaceActions(session, vi.fn()));
    await waitFor(() => expect(nativeAction).toBeTypeOf("function"));

    act(() => result.current.dispatch("save_document_as"));
    act(() => nativeAction?.("save_document_as"));
    act(() => result.current.dispatch("save_document"));

    expect(session.requestSaveAs).toHaveBeenCalledTimes(2);
    expect(session.save).toHaveBeenCalledOnce();
  });

  it("synchronizes busy state and rejects stale native actions", async () => {
    const session = documentSession();
    let nativeAction: ((action: string) => void) | undefined;
    listenMock.mockImplementation(async (onAction) => {
      nativeAction = onAction;
      return vi.fn();
    });
    const { result, rerender } = renderHook(
      ({ current }) => useWorkspaceActions(current, vi.fn()),
      { initialProps: { current: session } },
    );
    await waitFor(() => expect(setStateMock).toHaveBeenCalledWith(enabledNativeState()));

    rerender({ current: { ...session, operation: "choosing_save_format" } });
    await waitFor(() => expect(setStateMock).toHaveBeenLastCalledWith(disabledNativeState()));
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
    const { result } = renderHook(() => useWorkspaceActions(session, vi.fn()));
    await waitFor(() => expect(nativeAction).toBeTypeOf("function"));

    act(() => result.current.dispatch("save_back_to_source"));
    act(() => nativeAction?.("save_back_to_source"));

    expect(result.current.feedback).toBe(session.saveBackUnavailableReason);
    expect(session.requestSaveBack).not.toHaveBeenCalled();
  });

  it("exposes bounded recovery for native bridge failures", async () => {
    setStateMock.mockResolvedValue({
      status: "error",
      error: { type: "command", code: "menu_update_failed" },
    });
    const { result } = renderHook(() => useWorkspaceActions(documentSession(), vi.fn()));

    await waitFor(() => expect(result.current.feedback).toBe(
      "DRAFT could not update the native menu. Use the toolbar.",
    ));
  });

  it("handles native listener setup failure without an unhandled rejection", async () => {
    listenMock.mockRejectedValue(new Error("private event transport detail"));
    const { result } = renderHook(() => useWorkspaceActions(documentSession(), vi.fn()));

    await waitFor(() => expect(result.current.feedback).toBe(
      "DRAFT could not read native menu actions. Use the toolbar.",
    ));
  });

  it("disables document-specific actions when no document is open", async () => {
    renderHook(() => useWorkspaceActions({
      ...documentSession(),
      canClose: false,
      canSave: false,
      canSaveAs: false,
      canSaveBack: false,
    }, vi.fn()));

    await waitFor(() => expect(setStateMock).toHaveBeenCalledWith({
      ...enabledNativeState(),
      canClose: false,
      canSave: false,
      canSaveAs: false,
      canSaveBack: false,
    }));
  });
});

function documentSession(): DocumentSession {
  return {
    canClose: true,
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
    requestSaveAs: vi.fn(),
    requestSaveBack: vi.fn(),
    resolvePendingAction: vi.fn(),
    resolveSaveAs: vi.fn(),
    resolveSaveBack: vi.fn(),
    save: vi.fn().mockResolvedValue(true),
    saveAsOpen: false,
    saveBackConfirmation: null,
    saveBackUnavailableReason: "",
    saveBackVisible: true,
    snapshot: vi.fn(),
    statusLabel: "Saved",
    title: "Research.draft",
    unsaved: false,
  };
}

function enabledNativeState() {
  return {
    canNew: true,
    canOpen: true,
    canClose: true,
    canSave: true,
    canSaveAs: true,
    canSaveBack: true,
  };
}

function disabledNativeState() {
  return {
    canNew: false,
    canOpen: false,
    canClose: false,
    canSave: false,
    canSaveAs: false,
    canSaveBack: false,
  };
}
