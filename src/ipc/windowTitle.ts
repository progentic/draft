import { invokeCommand } from "./client";

export interface WindowTitleRequest {
  displayName: string | null;
  unsaved: boolean;
}

export type WindowTitleResult =
  | { status: "applied" }
  | { status: "error"; code: "invalid_title" | "window_update_failed" | "transport" };

const COMMAND_NAME = "set_window_title";

export async function setWindowTitle(request: WindowTitleRequest): Promise<WindowTitleResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request });
    return isAppliedResponse(response)
      ? { status: "applied" }
      : { status: "error", code: "transport" };
  } catch (error: unknown) {
    return { status: "error", code: errorCode(error) };
  }
}

function isAppliedResponse(value: unknown): value is { applied: true } {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.keys(value).length === 1 &&
    "applied" in value &&
    value.applied === true
  );
}

function errorCode(error: unknown): Extract<WindowTitleResult, { status: "error" }>["code"] {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error.code === "invalid_title" || error.code === "window_update_failed")
  ) {
    return error.code;
  }
  return "transport";
}
