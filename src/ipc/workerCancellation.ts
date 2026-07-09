import { invokeCommand } from "./client";

export interface CancelWorkerRequest {
  workerId: string;
}

export type CancelWorkerClientError =
  | {
      type: "command";
      code: "invalid_worker_id" | "worker_not_found" | "registry_unavailable";
    }
  | { type: "invalid-response" }
  | { type: "transport" };

export type CancelWorkerResult =
  | { status: "cancellation-requested" }
  | { status: "already-ended" }
  | { status: "error"; error: CancelWorkerClientError };

type CancelWorkerArguments = {
  request: CancelWorkerRequest;
};

const COMMAND_NAME = "cancel_worker";

/** Requests cancellation for a Rust-owned worker without throwing raw IPC errors. */
export async function cancelWorker(request: CancelWorkerRequest): Promise<CancelWorkerResult> {
  const commandArguments: CancelWorkerArguments = { request };

  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, commandArguments);
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(response: unknown): CancelWorkerResult {
  if (!isCancelWorkerResponse(response)) {
    return { status: "error", error: { type: "invalid-response" } };
  }

  return response.status === "cancellation_requested"
    ? { status: "cancellation-requested" }
    : { status: "already-ended" };
}

function clientErrorFrom(error: unknown): CancelWorkerClientError {
  if (isRecord(error) && isCommandErrorCode(error.code)) {
    return { type: "command", code: error.code };
  }

  return { type: "transport" };
}

function isCancelWorkerResponse(
  value: unknown,
): value is { status: "cancellation_requested" | "already_ended" } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    (value.status === "cancellation_requested" || value.status === "already_ended")
  );
}

function isCommandErrorCode(value: unknown) {
  return (
    value === "invalid_worker_id" ||
    value === "worker_not_found" ||
    value === "registry_unavailable"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
