import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

import {
  listenToNativeMenuActions,
  setNativeMenuState,
  type NativeMenuAction,
  type NativeMenuState,
} from "./nativeMenu";

const MENU_STATE: NativeMenuState = {
  canNew: true,
  canOpen: true,
  canClose: false,
  canSave: false,
  canSaveAs: false,
  canExport: false,
};

describe("native menu client", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("sends only the bounded action availability state", async () => {
    invokeMock.mockResolvedValue({ applied: true });

    await expect(setNativeMenuState(MENU_STATE)).resolves.toEqual({ status: "applied" });
    expect(invokeMock).toHaveBeenCalledWith("set_native_menu_state", {
      request: MENU_STATE,
    });
  });

  it.each([
    [null, "invalid-response"],
    [{ applied: false }, "invalid-response"],
    [{ applied: true, path: "/private/document.draft" }, "invalid-response"],
  ])("rejects invalid state response %#", async (response, type) => {
    invokeMock.mockResolvedValue(response);

    await expect(setNativeMenuState(MENU_STATE)).resolves.toEqual({
      status: "error",
      error: { type },
    });
  });

  it("preserves the typed update failure without retaining details", async () => {
    invokeMock.mockRejectedValue({ code: "menu_update_failed", detail: "/private/path" });

    await expect(setNativeMenuState(MENU_STATE)).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "menu_update_failed" },
    });
  });

  it.each([
    "new_document",
    "open_document",
    "close_document",
    "save_document",
    "save_document_as",
    "export_docx",
  ] as const)("delivers the closed %s action", async (action) => {
    const unlisten = vi.fn();
    let deliver: ((event: { payload: unknown }) => void) | undefined;
    listenMock.mockImplementation(async (_name, callback) => {
      deliver = callback;
      return unlisten;
    });
    const onAction = vi.fn<(action: NativeMenuAction) => void>();
    const onError = vi.fn();

    await expect(listenToNativeMenuActions(onAction, onError)).resolves.toBe(unlisten);
    expect(listenMock).toHaveBeenCalledWith("draft://native-menu-action", expect.any(Function));
    deliver?.({ payload: { action } });

    expect(onAction).toHaveBeenCalledWith(action);
    expect(onError).not.toHaveBeenCalled();
  });

  it.each([
    null,
    { action: "unknown" },
    { action: "save_document", path: "/private/document.draft" },
  ])("rejects malformed event payload %#", async (payload) => {
    let deliver: ((event: { payload: unknown }) => void) | undefined;
    listenMock.mockImplementation(async (_name, callback) => {
      deliver = callback;
      return vi.fn();
    });
    const onAction = vi.fn();
    const onError = vi.fn();
    await listenToNativeMenuActions(onAction, onError);

    deliver?.({ payload });

    expect(onAction).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledWith({ type: "invalid-payload" });
  });
});
