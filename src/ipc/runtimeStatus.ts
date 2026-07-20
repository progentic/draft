import { invokeCommand } from "./client";

export type GetRuntimeStatusRequest = Record<string, never>;

export interface GetRuntimeStatusResponse {
  buildCommit: string;
  buildProfile: "debug" | "release";
  version: string;
}

export type RuntimeStatusClientError =
  | {
      type: "command";
      code:
        | "event_delivery_failed"
        | "invalid_application_version"
        | "invalid_build_metadata";
    }
  | { type: "invalid-response" }
  | { type: "transport" };

export type RuntimeStatusResult =
  | ({ status: "ready" } & GetRuntimeStatusResponse)
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

  return { status: "ready", ...response };
}

function clientErrorFrom(error: unknown): RuntimeStatusClientError {
  if (isRecord(error) && isCommandErrorCode(error.code)) {
    return { type: "command", code: error.code };
  }

  return { type: "transport" };
}

function isCommandErrorCode(value: unknown) {
  return (
    value === "event_delivery_failed" ||
    value === "invalid_application_version" ||
    value === "invalid_build_metadata"
  );
}

export function isRuntimeStatusResponse(value: unknown): value is GetRuntimeStatusResponse {
  return (
    isRecord(value) &&
    Object.keys(value).length === 3 &&
    isBuildCommit(value.buildCommit) &&
    isBuildProfile(value.buildProfile) &&
    typeof value.version === "string" &&
    value.version.trim().length > 0
  );
}

function isBuildCommit(value: unknown) {
  return (
    value === "development" ||
    (typeof value === "string" && /^[0-9a-f]{40}$/.test(value))
  );
}

function isBuildProfile(value: unknown): value is "debug" | "release" {
  return value === "debug" || value === "release";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
