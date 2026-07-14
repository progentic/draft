import { invokeCommand } from "./client";
import { listenToEvent, type StopEventListener } from "./eventClient";
import {
  openDocumentResultFromResponse,
  type OpenDocumentResult,
} from "./documentOpen";
import { isOpenDocumentCommandError, type OpenDocumentCommandError } from "./documentErrors";
import { isRecord } from "./documentEnvelope";

export type ApplicationOpenClientError =
  | {
      type: "command";
      code:
        | "multiple_files_unsupported"
        | "queue_unavailable"
        | "unsupported_file_location";
    }
  | { type: "open"; error: OpenDocumentCommandError }
  | { type: "invalid-response" | "transport" };

export type ApplicationOpenResult =
  | { status: "none" }
  | { status: "open"; result: OpenDocumentResult }
  | { status: "error"; error: ApplicationOpenClientError };

export type ApplicationOpenEventError =
  | { type: "invalid-payload" }
  | { type: "queue-unavailable" };

const COMMAND_NAME = "open_application_document";
const EVENT_NAME = "draft://application-open";

export async function takeApplicationOpenRequest(): Promise<ApplicationOpenResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, {
      request: { disposition: "open" },
    });
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

export async function dismissApplicationOpenRequest(): Promise<boolean> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, {
      request: { disposition: "dismiss" },
    });
    return (
      isRecord(response) &&
      Object.keys(response).length === 1 &&
      (response.status === "dismissed" || response.status === "none")
    );
  } catch {
    return false;
  }
}

export async function listenToApplicationOpenRequests(
  onAvailable: () => void,
  onError: (error: ApplicationOpenEventError) => void,
): Promise<StopEventListener> {
  return listenToEvent(EVENT_NAME, (payload) => deliverEvent(payload, onAvailable, onError));
}

function resultFromResponse(response: unknown): ApplicationOpenResult {
  if (isRecord(response) && Object.keys(response).length === 1 && response.status === "none") {
    return { status: "none" };
  }
  if (isRecord(response) && Object.keys(response).length === 2 && response.status === "opened") {
    const result = openDocumentResultFromResponse(response.result);
    return result.status === "error" && result.error.type === "invalid-response"
      ? { status: "error", error: { type: "invalid-response" } }
      : { status: "open", result };
  }
  return { status: "error", error: { type: "invalid-response" } };
}

function clientErrorFrom(value: unknown): ApplicationOpenClientError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return { type: "transport" };
  }
  if (
    Object.keys(value).length === 1 &&
    (value.code === "multiple_files_unsupported" ||
      value.code === "queue_unavailable" ||
      value.code === "unsupported_file_location")
  ) {
    return { type: "command", code: value.code };
  }
  if (
    Object.keys(value).length === 2 &&
    value.code === "open" &&
    isOpenDocumentCommandError(value.cause)
  ) {
    return { type: "open", error: value.cause };
  }
  return { type: "transport" };
}

function deliverEvent(
  payload: unknown,
  onAvailable: () => void,
  onError: (error: ApplicationOpenEventError) => void,
) {
  if (!isRecord(payload) || Object.keys(payload).length !== 1) {
    onError({ type: "invalid-payload" });
    return;
  }
  if (payload.type === "available") {
    onAvailable();
    return;
  }
  onError(
    payload.type === "queue_unavailable"
      ? { type: "queue-unavailable" }
      : { type: "invalid-payload" },
  );
}
