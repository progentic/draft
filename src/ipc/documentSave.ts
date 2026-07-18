import { invokeCommand } from "./client";
import {
  isDocumentId,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";
import { isSaveDocumentCommandError, type SaveDocumentCommandError } from "./documentErrors";

export const SAVE_AS_FORMATS = ["draft", "docx", "txt"] as const;

export type SaveAsFormat = (typeof SAVE_AS_FORMATS)[number];
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
  | {
      status: "draft_saved";
      documentId: string;
      displayName: string;
      wasSaveAs: boolean;
      authoritativeIdentityChanged: boolean;
      dirtyStateCleared: true;
    }
  | {
      status: "converted_output";
      displayName: string;
      outputFormat: Exclude<SaveAsFormat, "draft">;
      bytesWritten: number;
      authoritativeIdentityChanged: false;
      dirtyStateChanged: false;
    }
  | { status: "cancelled" }
  | { status: "error"; error: SaveDocumentClientError };

interface SaveDocumentRequest {
  displayName: string;
  format?: SaveAsFormat;
  mode: SaveDocumentMode;
  origin: SaveDocumentOrigin;
  snapshot: DocumentEnvelopeSnapshot;
}

const COMMAND_NAME = "save_document";

export function saveDocument(
  snapshot: DocumentEnvelopeSnapshot,
  displayName: string,
  origin: SaveDocumentOrigin,
): Promise<SaveDocumentResult> {
  return invokeSave({ displayName, mode: "save", origin, snapshot });
}

export function saveDocumentAs(
  snapshot: DocumentEnvelopeSnapshot,
  displayName: string,
  origin: SaveDocumentOrigin,
  format: SaveAsFormat,
): Promise<SaveDocumentResult> {
  return invokeSave({ displayName, format, mode: "save_as", origin, snapshot });
}

async function invokeSave(request: SaveDocumentRequest): Promise<SaveDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request });
    return resultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(response: unknown): SaveDocumentResult {
  if (isCancelledResponse(response)) return { status: "cancelled" };
  if (isDraftSavedResponse(response)) return response;
  if (isConvertedOutputResponse(response)) return response;
  return { status: "error", error: { type: "invalid-response" } };
}

function clientErrorFrom(error: unknown): SaveDocumentClientError {
  return isSaveDocumentCommandError(error)
    ? { type: "command", error }
    : { type: "transport" };
}

function isCancelledResponse(value: unknown): value is { status: "cancelled" } {
  return isRecord(value) && Object.keys(value).length === 1 && value.status === "cancelled";
}

function isDraftSavedResponse(
  value: unknown,
): value is Extract<SaveDocumentResult, { status: "draft_saved" }> {
  return (
    isRecord(value) &&
    hasExactKeys(value, [
      "authoritativeIdentityChanged",
      "dirtyStateCleared",
      "displayName",
      "documentId",
      "status",
      "wasSaveAs",
    ]) &&
    value.status === "draft_saved" &&
    isDocumentId(value.documentId) &&
    isDisplayName(value.displayName) &&
    typeof value.wasSaveAs === "boolean" &&
    value.authoritativeIdentityChanged === value.wasSaveAs &&
    value.dirtyStateCleared === true
  );
}

function isConvertedOutputResponse(
  value: unknown,
): value is Extract<SaveDocumentResult, { status: "converted_output" }> {
  return (
    isRecord(value) &&
    hasExactKeys(value, [
      "authoritativeIdentityChanged",
      "bytesWritten",
      "dirtyStateChanged",
      "displayName",
      "outputFormat",
      "status",
    ]) &&
    value.status === "converted_output" &&
    isDisplayName(value.displayName) &&
    (value.outputFormat === "docx" || value.outputFormat === "txt") &&
    Number.isSafeInteger(value.bytesWritten) &&
    Number(value.bytesWritten) >= 0 &&
    value.authoritativeIdentityChanged === false &&
    value.dirtyStateChanged === false
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

function hasExactKeys(value: Record<string, unknown>, expected: string[]) {
  const keys = Object.keys(value).sort();
  return keys.length === expected.length && keys.every((key, index) => key === expected[index]);
}
