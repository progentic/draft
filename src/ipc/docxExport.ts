import { invokeCommand } from "./client";
import { isRecord, type DocumentEnvelopeSnapshot } from "./documentEnvelope";

export type DocxExportErrorCode =
  | "artifact_too_large"
  | "durability_uncertain"
  | "invalid_document_structure"
  | "invalid_target"
  | "nesting_too_deep"
  | "package_construction_failed"
  | "source_too_large"
  | "too_many_nodes"
  | "unsupported_citation"
  | "unsupported_document_content"
  | "write_failed";

export type ExportDocumentErrorCode =
  | "export"
  | "invalid_envelope"
  | "unsupported_file_location";

export type ExportDocumentClientError =
  | { type: "command"; code: ExportDocumentErrorCode; cause?: DocxExportErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type ExportDocumentResult =
  | { status: "exported"; bytesWritten: number }
  | { status: "cancelled" }
  | { status: "error"; error: ExportDocumentClientError };

const COMMAND_NAME = "export_document";

export async function exportDocument(
  snapshot: DocumentEnvelopeSnapshot,
): Promise<ExportDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, {
      request: { snapshot },
    });
    return exportResult(response);
  } catch (error: unknown) {
    return { status: "error", error: exportError(error) };
  }
}

function exportResult(response: unknown): ExportDocumentResult {
  if (isRecord(response) && Object.keys(response).length === 1 && response.status === "cancelled") {
    return { status: "cancelled" };
  }
  if (
    isRecord(response) &&
    Object.keys(response).length === 2 &&
    response.status === "exported" &&
    Number.isSafeInteger(response.bytesWritten) &&
    Number(response.bytesWritten) > 0
  ) {
    return { status: "exported", bytesWritten: Number(response.bytesWritten) };
  }
  return { status: "error", error: { type: "invalid-response" } };
}

function exportError(error: unknown): ExportDocumentClientError {
  if (!isRecord(error) || typeof error.code !== "string") {
    return { type: "transport" };
  }
  if (Object.keys(error).length === 1 && error.code === "unsupported_file_location") {
    return { type: "command", code: error.code };
  }
  if (
    Object.keys(error).length === 2 &&
    error.code === "invalid_envelope" &&
    isRecord(error.cause)
  ) {
    return { type: "command", code: "invalid_envelope" };
  }
  if (
    Object.keys(error).length === 2 &&
    error.code === "export" &&
    isDocxExportError(error.cause)
  ) {
    return { type: "command", code: "export", cause: error.cause.code };
  }
  return { type: "transport" };
}

export function isDocxExportError(value: unknown): value is { code: DocxExportErrorCode } {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    (value.code === "artifact_too_large" ||
      value.code === "durability_uncertain" ||
      value.code === "invalid_document_structure" ||
      value.code === "invalid_target" ||
      value.code === "nesting_too_deep" ||
      value.code === "package_construction_failed" ||
      value.code === "source_too_large" ||
      value.code === "too_many_nodes" ||
      value.code === "unsupported_citation" ||
      value.code === "unsupported_document_content" ||
      value.code === "write_failed")
  );
}
