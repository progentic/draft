import { invokeCommand } from "./client";
import {
  type DocumentEnvelopeSnapshot,
  isDocumentEnvelopeError,
  isDocumentId,
  isRecord,
} from "./documentEnvelope";
import {
  type AtomicDocumentWriteError,
  type DocumentRegistryError,
  isAtomicDocumentWriteError,
  isDocumentRegistryError,
} from "./documentErrors";
import { type DocxExportErrorCode, isDocxExportError } from "./docxExport";
import type { SameFormatSaveDisposition } from "./externalDocument";

export type ExternalSaveDecision =
  "save_exact" | "accept_normalization" | "cancel";

export type ExternalSaveRecovery =
  | "confirm_normalization"
  | "save_as_draft"
  | "reopen_source"
  | "retry"
  | "none";

type DeniedExternalSaveDisposition = Exclude<
  SameFormatSaveDisposition,
  "no_changes" | "allowed_exact" | "allowed_after_accepted_normalization"
>;

export type ExternalSaveCommandError =
  | { code: "invalid_envelope" }
  | { code: "registry"; cause: DocumentRegistryError["code"] }
  | { code: "source_read"; cause: "read_failed" }
  | { code: "compilation"; cause: DocxExportErrorCode }
  | { code: "write_failed"; cause: AtomicDocumentWriteError["code"] }
  | { code: "replacement_rolled_back"; cause: ExternalSaveCommitFailureCode }
  | {
      code: "rollback_failed";
      cause: ExternalSaveCommitFailureCode;
      rollback: AtomicDocumentWriteError["code"];
    };

export type SaveExternalDocumentError =
  | { type: "command"; error: ExternalSaveCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type SaveExternalDocumentResult =
  | {
      status: "saved";
      documentId: string;
      displayName: string;
      bytesWritten: number;
      disposition: "allowed_exact" | "allowed_after_accepted_normalization";
    }
  | { status: "unchanged"; documentId: string; displayName: string }
  | {
      status: "confirmation_required";
      documentId: string;
      disposition: "allowed_after_accepted_normalization";
      recovery: "confirm_normalization";
    }
  | {
      status: "denied";
      documentId: string;
      disposition: DeniedExternalSaveDisposition;
      recovery: "save_as_draft" | "reopen_source";
    }
  | { status: "cancelled"; documentId: string }
  | {
      status: "error";
      error: SaveExternalDocumentError;
      recovery: ExternalSaveRecovery;
    };

type ExternalSaveCommitFailureCode = "durability_uncertain" | "registry";

const COMMAND_NAME = "save_external_document";

export async function saveExternalDocument(
  snapshot: DocumentEnvelopeSnapshot,
  decision: ExternalSaveDecision,
): Promise<SaveExternalDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, {
      request: { snapshot, decision },
    });
    return resultFromResponse(response);
  } catch (error: unknown) {
    return failureResult(error);
  }
}

function resultFromResponse(response: unknown): SaveExternalDocumentResult {
  if (
    isSavedResponse(response) ||
    isUnchangedResponse(response) ||
    isCancelledResponse(response)
  ) {
    return response;
  }
  if (isConfirmationResponse(response)) {
    return { ...response, recovery: "confirm_normalization" };
  }
  if (isDeniedResponse(response)) {
    return { ...response, recovery: recoveryForDenial(response.disposition) };
  }
  return invalidResponse();
}

function failureResult(error: unknown): SaveExternalDocumentResult {
  const commandError = parseCommandError(error);
  if (commandError) {
    return {
      status: "error",
      error: { type: "command", error: commandError },
      recovery: recoveryForCommandError(commandError),
    };
  }
  return transportFailure(error);
}

function invalidResponse(): SaveExternalDocumentResult {
  return {
    status: "error",
    error: { type: "invalid-response" },
    recovery: "retry",
  };
}

function transportFailure(error: unknown): SaveExternalDocumentResult {
  const isMalformedCommand = isRecord(error) && typeof error.code === "string";
  return {
    status: "error",
    error: { type: isMalformedCommand ? "invalid-response" : "transport" },
    recovery: "retry",
  };
}

function parseCommandError(value: unknown): ExternalSaveCommandError | null {
  if (!isRecord(value) || typeof value.code !== "string") {
    return null;
  }
  switch (value.code) {
    case "invalid_envelope":
      return parseInvalidEnvelope(value);
    case "registry":
      return parseRegistryFailure(value);
    case "source_read":
      return parseSourceReadFailure(value);
    case "compilation":
      return parseCompilationFailure(value);
    case "write_failed":
      return parseWriteFailure(value);
    case "replacement_rolled_back":
      return parseReplacementRollback(value);
    case "rollback_failed":
      return parseRollbackFailure(value);
    default:
      return null;
  }
}

function parseInvalidEnvelope(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  return hasExactKeys(value, ["cause", "code"]) &&
    isDocumentEnvelopeError(value.cause)
    ? { code: "invalid_envelope" }
    : null;
}

function parseRegistryFailure(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  return hasExactKeys(value, ["cause", "code"]) &&
    isDocumentRegistryError(value.cause)
    ? { code: "registry", cause: value.cause.code }
    : null;
}

function parseSourceReadFailure(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  return hasExactKeys(value, ["cause", "code"]) && isReadFailure(value.cause)
    ? { code: "source_read", cause: "read_failed" }
    : null;
}

function parseCompilationFailure(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  return hasExactKeys(value, ["cause", "code"]) &&
    isDocxExportError(value.cause)
    ? { code: "compilation", cause: value.cause.code }
    : null;
}

function parseWriteFailure(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  return hasExactKeys(value, ["cause", "code"]) &&
    isAtomicDocumentWriteError(value.cause)
    ? { code: "write_failed", cause: value.cause.code }
    : null;
}

function parseReplacementRollback(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  const cause = parseCommitFailure(value.cause);
  return hasExactKeys(value, ["cause", "code"]) && cause
    ? { code: "replacement_rolled_back", cause }
    : null;
}

function parseRollbackFailure(
  value: Record<string, unknown>,
): ExternalSaveCommandError | null {
  const cause = parseCommitFailure(value.cause);
  return hasExactKeys(value, ["cause", "code", "rollback"]) &&
    cause &&
    isAtomicDocumentWriteError(value.rollback)
    ? { code: "rollback_failed", cause, rollback: value.rollback.code }
    : null;
}

function parseCommitFailure(
  value: unknown,
): ExternalSaveCommitFailureCode | null {
  if (!isRecord(value) || typeof value.code !== "string") {
    return null;
  }
  if (hasExactKeys(value, ["code"]) && value.code === "durability_uncertain") {
    return value.code;
  }
  return hasExactKeys(value, ["cause", "code"]) &&
    value.code === "registry" &&
    isDocumentRegistryError(value.cause)
    ? value.code
    : null;
}

function recoveryForDenial(
  disposition: DeniedExternalSaveDisposition,
): "save_as_draft" | "reopen_source" {
  switch (disposition) {
    case "denied_source_missing":
    case "denied_source_changed":
      return "reopen_source";
    case "denied_unsupported_source_behavior":
    case "denied_read_only":
    case "denied_missing_provenance":
    case "denied_fidelity_unknown":
    case "denied_writer_unavailable":
      return "save_as_draft";
    default:
      return assertNever(disposition);
  }
}

function recoveryForCommandError(
  error: ExternalSaveCommandError,
): ExternalSaveRecovery {
  switch (error.code) {
    case "invalid_envelope":
      return "none";
    case "registry":
      return recoveryForRegistry(error.cause);
    case "source_read":
    case "write_failed":
    case "replacement_rolled_back":
      return "retry";
    case "compilation":
      return "save_as_draft";
    case "rollback_failed":
      return "reopen_source";
    default:
      return assertNever(error);
  }
}

function recoveryForRegistry(
  code: DocumentRegistryError["code"],
): ExternalSaveRecovery {
  switch (code) {
    case "not_open":
    case "external_source_unavailable":
      return "reopen_source";
    case "registry_unavailable":
      return "retry";
    case "already_open":
    case "source_path_in_use":
      return "none";
    default:
      return assertNever(code);
  }
}

function isSavedResponse(
  value: unknown,
): value is Extract<SaveExternalDocumentResult, { status: "saved" }> {
  return (
    isRecord(value) &&
    hasExactKeys(value, [
      "bytesWritten",
      "displayName",
      "disposition",
      "documentId",
      "status",
    ]) &&
    value.status === "saved" &&
    isDocumentId(value.documentId) &&
    isDisplayName(value.displayName) &&
    Number.isSafeInteger(value.bytesWritten) &&
    Number(value.bytesWritten) > 0 &&
    (value.disposition === "allowed_exact" ||
      value.disposition === "allowed_after_accepted_normalization")
  );
}

function isUnchangedResponse(
  value: unknown,
): value is Extract<SaveExternalDocumentResult, { status: "unchanged" }> {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["displayName", "documentId", "status"]) &&
    value.status === "unchanged" &&
    isDocumentId(value.documentId) &&
    isDisplayName(value.displayName)
  );
}

function isConfirmationResponse(value: unknown): value is {
  status: "confirmation_required";
  documentId: string;
  disposition: "allowed_after_accepted_normalization";
} {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["disposition", "documentId", "status"]) &&
    value.status === "confirmation_required" &&
    isDocumentId(value.documentId) &&
    value.disposition === "allowed_after_accepted_normalization"
  );
}

function isDeniedResponse(value: unknown): value is {
  status: "denied";
  documentId: string;
  disposition: DeniedExternalSaveDisposition;
} {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["disposition", "documentId", "status"]) &&
    value.status === "denied" &&
    isDocumentId(value.documentId) &&
    isDeniedDisposition(value.disposition)
  );
}

function isCancelledResponse(
  value: unknown,
): value is Extract<SaveExternalDocumentResult, { status: "cancelled" }> {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["documentId", "status"]) &&
    value.status === "cancelled" &&
    isDocumentId(value.documentId)
  );
}

function isDeniedDisposition(
  value: unknown,
): value is DeniedExternalSaveDisposition {
  return (
    value === "denied_unsupported_source_behavior" ||
    value === "denied_read_only" ||
    value === "denied_missing_provenance" ||
    value === "denied_fidelity_unknown" ||
    value === "denied_writer_unavailable" ||
    value === "denied_source_missing" ||
    value === "denied_source_changed"
  );
}

function isReadFailure(value: unknown): value is { code: "read_failed" } {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["code"]) &&
    value.code === "read_failed"
  );
}

function isDisplayName(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    !value.includes("/") &&
    !value.includes("\\") &&
    !value.includes("\0")
  );
}

function hasExactKeys(
  value: Record<string, unknown>,
  expected: string[],
): boolean {
  const keys = Object.keys(value).sort();
  return (
    keys.length === expected.length &&
    keys.every((key, index) => key === expected[index])
  );
}

function assertNever(value: never): never {
  throw new Error(`unreachable external save variant: ${String(value)}`);
}
