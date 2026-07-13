import { invokeCommand } from "./client";
import { isDocumentId, isRecord } from "./documentEnvelope";

export type CloseDocumentErrorCode =
  | "already_open"
  | "not_open"
  | "registry_unavailable"
  | "source_path_in_use";

export type CloseDocumentClientError =
  | { type: "command"; code: CloseDocumentErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type CloseDocumentResult =
  | { status: "closed"; documentId: string }
  | { status: "error"; error: CloseDocumentClientError };

const COMMAND_NAME = "close_document";

export async function closeDocument(documentId: string): Promise<CloseDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, {
      request: { documentId },
    });
    return closedResult(response);
  } catch (error: unknown) {
    return { status: "error", error: closeError(error) };
  }
}

function closedResult(response: unknown): CloseDocumentResult {
  if (
    isRecord(response) &&
    Object.keys(response).length === 2 &&
    response.status === "closed" &&
    isDocumentId(response.documentId)
  ) {
    return { status: "closed", documentId: response.documentId };
  }
  return { status: "error", error: { type: "invalid-response" } };
}

function closeError(error: unknown): CloseDocumentClientError {
  if (isCloseDocumentError(error)) {
    return { type: "command", code: error.code };
  }
  return { type: "transport" };
}

function isCloseDocumentError(value: unknown): value is { code: CloseDocumentErrorCode } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    (value.code === "already_open" ||
      value.code === "not_open" ||
      value.code === "registry_unavailable" ||
      value.code === "source_path_in_use")
  );
}
