import { invokeCommand } from "./client";
import {
  isDocumentId,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";
import { isSaveDocumentCommandError, type SaveDocumentCommandError } from "./documentErrors";

export interface SaveDocumentRequest {
  displayName: string;
  mode: SaveDocumentMode;
  origin: SaveDocumentOrigin;
  snapshot: DocumentEnvelopeSnapshot;
}

export type SaveDocumentMode = "save" | "save_as";
export type SaveDocumentOrigin =
  | "imported_external"
  | "imported_text"
  | "new"
  | "opened_draft";

export type SaveDocumentClientError =
  | { type: "command"; error: SaveDocumentCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type SaveDocumentResult =
  | { status: "saved"; documentId: string; displayName: string; wasSaveAs: boolean }
  | { status: "cancelled" }
  | { status: "error"; error: SaveDocumentClientError };

type SaveDocumentArguments = {
  request: SaveDocumentRequest;
};

const COMMAND_NAME = "save_document";

/** Saves only the explicit editor snapshot supplied by the frontend. */
export async function saveDocument(
  snapshot: DocumentEnvelopeSnapshot,
  mode: SaveDocumentMode,
  displayName: string,
  origin: SaveDocumentOrigin,
): Promise<SaveDocumentResult> {
  const commandArguments: SaveDocumentArguments = {
    request: { displayName, mode, origin, snapshot },
  };

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
    return {
      status: "saved",
      documentId: response.documentId,
      displayName: response.displayName,
      wasSaveAs: response.wasSaveAs,
    };
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

function isSavedResponse(value: unknown): value is {
  status: "saved";
  documentId: string;
  displayName: string;
  wasSaveAs: boolean;
} {
  return (
    isRecord(value) &&
    Object.keys(value).length === 4 &&
    value.status === "saved" &&
    isDocumentId(value.documentId) &&
    isDisplayName(value.displayName) &&
    typeof value.wasSaveAs === "boolean"
  );
}

function isDisplayName(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    !value.includes("/") &&
    !value.includes("\\")
  );
}
