import { invokeCommand } from "./client";
import { listenToEvent, type StopEventListener } from "./eventClient";

export type NativeMenuAction =
  | "new_document"
  | "open_document"
  | "close_document"
  | "save_document"
  | "save_document_as"
  | "save_back_to_source";

export interface NativeMenuState {
  canNew: boolean;
  canOpen: boolean;
  canClose: boolean;
  canSave: boolean;
  canSaveAs: boolean;
  canSaveBack: boolean;
}

export type NativeMenuClientError =
  | { type: "command"; code: "menu_update_failed" }
  | { type: "invalid-payload" | "invalid-response" | "transport" };

export type NativeMenuStateResult =
  | { status: "applied" }
  | { status: "error"; error: NativeMenuClientError };

const EVENT_NAME = "draft://native-menu-action";
const COMMAND_NAME = "set_native_menu_state";
const ACTIONS: readonly NativeMenuAction[] = [
  "new_document",
  "open_document",
  "close_document",
  "save_document",
  "save_document_as",
  "save_back_to_source",
];

export async function listenToNativeMenuActions(
  onAction: (action: NativeMenuAction) => void,
  onError: (error: NativeMenuClientError) => void,
): Promise<StopEventListener> {
  return listenToEvent(EVENT_NAME, (payload) => deliverAction(payload, onAction, onError));
}

export async function setNativeMenuState(
  state: NativeMenuState,
): Promise<NativeMenuStateResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: state });
    return isAppliedResponse(response)
      ? { status: "applied" }
      : { status: "error", error: { type: "invalid-response" } };
  } catch (error: unknown) {
    return { status: "error", error: commandError(error) };
  }
}

function deliverAction(
  payload: unknown,
  onAction: (action: NativeMenuAction) => void,
  onError: (error: NativeMenuClientError) => void,
) {
  if (!isNativeMenuEvent(payload)) {
    onError({ type: "invalid-payload" });
    return;
  }
  onAction(payload.action);
}

function isNativeMenuEvent(value: unknown): value is { action: NativeMenuAction } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    typeof value.action === "string" &&
    ACTIONS.includes(value.action as NativeMenuAction)
  );
}

function isAppliedResponse(value: unknown): value is { applied: true } {
  return isRecord(value) && Object.keys(value).length === 1 && value.applied === true;
}

function commandError(value: unknown): NativeMenuClientError {
  return isRecord(value) && value.code === "menu_update_failed"
    ? { type: "command", code: "menu_update_failed" }
    : { type: "transport" };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
