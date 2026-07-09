import { invokeCommand } from "./client";
import {
  isDocumentEnvelopeSnapshot,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";
import { isOpenDocumentCommandError, type OpenDocumentCommandError } from "./documentErrors";

export type OpenDocumentRequest = Record<string, never>;

export type OpenDocumentClientError =
  | { type: "command"; error: OpenDocumentCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type OpenDocumentResult =
  | { status: "opened"; envelope: DocumentEnvelopeSnapshot }
  | { status: "cancelled" }
  | { status: "error"; error: OpenDocumentClientError };

type OpenDocumentArguments = {
  request: OpenDocumentRequest;
};

const COMMAND_NAME = "open_document";
const COMMAND_ARGUMENTS: OpenDocumentArguments = { request: {} };

/** Opens a Rust-selected document without exposing filesystem APIs to the UI. */
export async function openDocument(): Promise<OpenDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, COMMAND_ARGUMENTS);
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(response: unknown): OpenDocumentResult {
  if (isCancelledResponse(response)) {
    return { status: "cancelled" };
  }

  if (isOpenedResponse(response)) {
    return { status: "opened", envelope: response.envelope };
  }

  return { status: "error", error: { type: "invalid-response" } };
}

function clientErrorFrom(error: unknown): OpenDocumentClientError {
  if (isOpenDocumentCommandError(error)) {
    return { type: "command", error };
  }

  return { type: "transport" };
}

function isCancelledResponse(value: unknown): value is { status: "cancelled" } {
  return isRecord(value) && Object.keys(value).length === 1 && value.status === "cancelled";
}

function isOpenedResponse(
  value: unknown,
): value is { status: "opened"; envelope: DocumentEnvelopeSnapshot } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 2 &&
    value.status === "opened" &&
    isDocumentEnvelopeSnapshot(value.envelope)
  );
}
