import { invokeCommand } from "./client";

export type GetRuntimeStatusRequest = Record<string, never>;

export interface GetRuntimeStatusResponse {
  version: string;
}

export type RuntimeStatusClientError =
  | { type: "command"; code: "invalid_application_version" }
  | { type: "invalid-response" }
  | { type: "transport" };

export type RuntimeStatusResult =
  | { status: "ready"; version: string }
  | { status: "error"; error: RuntimeStatusClientError };

type GetRuntimeStatusArguments = {
  request: GetRuntimeStatusRequest;
};

const COMMAND_NAME = "get_runtime_status";
const COMMAND_ARGUMENTS: GetRuntimeStatusArguments = { request: {} };

/** Returns validated status from the trusted Rust runtime without throwing. */
export async function getRuntimeStatus(): Promise<RuntimeStatusResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, COMMAND_ARGUMENTS);
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(response: unknown): RuntimeStatusResult {
  if (!isRuntimeStatusResponse(response)) {
    return { status: "error", error: { type: "invalid-response" } };
  }

  return { status: "ready", version: response.version };
}

function clientErrorFrom(error: unknown): RuntimeStatusClientError {
  if (isRecord(error) && error.code === "invalid_application_version") {
    return { type: "command", code: error.code };
  }

  return { type: "transport" };
}

function isRuntimeStatusResponse(value: unknown): value is GetRuntimeStatusResponse {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    typeof value.version === "string" &&
    value.version.trim().length > 0
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
