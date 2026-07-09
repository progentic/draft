import { invokeCommand } from "./client";
import {
  isDocumentId,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";
import { isSaveDocumentCommandError, type SaveDocumentCommandError } from "./documentErrors";

export interface SaveDocumentRequest {
  snapshot: DocumentEnvelopeSnapshot;
}

export type SaveDocumentClientError =
  | { type: "command"; error: SaveDocumentCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type SaveDocumentResult =
  | { status: "saved"; documentId: string }
  | { status: "cancelled" }
  | { status: "error"; error: SaveDocumentClientError };

type SaveDocumentArguments = {
  request: SaveDocumentRequest;
};

const COMMAND_NAME = "save_document";

/** Saves only the explicit editor snapshot supplied by the frontend. */
export async function saveDocument(snapshot: DocumentEnvelopeSnapshot): Promise<SaveDocumentResult> {
  const commandArguments: SaveDocumentArguments = { request: { snapshot } };

  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, commandArguments);
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(response: unknown): SaveDocumentResult {
  if (isCancelledResponse(response)) {
    return { status: "cancelled" };
  }

  if (isSavedResponse(response)) {
    return { status: "saved", documentId: response.documentId };
  }

  return { status: "error", error: { type: "invalid-response" } };
}

function clientErrorFrom(error: unknown): SaveDocumentClientError {
  if (isSaveDocumentCommandError(error)) {
    return { type: "command", error };
  }

  return { type: "transport" };
}

function isCancelledResponse(value: unknown): value is { status: "cancelled" } {
  return isRecord(value) && Object.keys(value).length === 1 && value.status === "cancelled";
}

function isSavedResponse(value: unknown): value is { status: "saved"; documentId: string } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 2 &&
    value.status === "saved" &&
    isDocumentId(value.documentId)
  );
}
