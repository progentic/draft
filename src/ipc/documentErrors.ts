import {
  isDocumentEnvelopeError,
  isRecord,
  type DocumentEnvelopeError,
} from "./documentEnvelope";

export type DocumentRegistryError = {
  code: "already_open" | "not_open" | "source_path_in_use" | "registry_unavailable";
};

export type AtomicDocumentWriteError = {
  code:
    | "open_temporary_file"
    | "write_temporary_file"
    | "sync_temporary_file"
    | "replace_target"
    | "cleanup_temporary_file"
    | "sync_parent_directory";
};

export type OpenDocumentCommandError =
  | { code: "unsupported_file_location" | "file_not_found" | "read_failed" | "malformed_json" }
  | { code: "invalid_envelope"; cause: DocumentEnvelopeError }
  | { code: "registry"; cause: DocumentRegistryError };

export type SaveDocumentCommandError =
  | { code: "unsupported_file_location" | "serialization_failed" | "durability_uncertain" }
  | { code: "write_failed"; cause: AtomicDocumentWriteError }
  | { code: "invalid_envelope"; cause: DocumentEnvelopeError }
  | { code: "registry"; cause: DocumentRegistryError };

export function isOpenDocumentCommandError(value: unknown): value is OpenDocumentCommandError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }

  return (
    isFieldlessOpenError(value) ||
    isInvalidEnvelopeError(value) ||
    isRegistryCommandError(value)
  );
}

export function isSaveDocumentCommandError(value: unknown): value is SaveDocumentCommandError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }

  return (
    isFieldlessSaveError(value) ||
    isWriteFailure(value) ||
    isInvalidEnvelopeError(value) ||
    isRegistryCommandError(value)
  );
}

function isFieldlessOpenError(value: Record<string, unknown>): boolean {
  return (
    hasOnlyCode(value) &&
    (value.code === "unsupported_file_location" ||
      value.code === "file_not_found" ||
      value.code === "read_failed" ||
      value.code === "malformed_json")
  );
}

function isFieldlessSaveError(value: Record<string, unknown>): boolean {
  return (
    hasOnlyCode(value) &&
    (value.code === "unsupported_file_location" ||
      value.code === "serialization_failed" ||
      value.code === "durability_uncertain")
  );
}

function isWriteFailure(
  value: Record<string, unknown>,
): value is { code: "write_failed"; cause: AtomicDocumentWriteError } {
  return (
    value.code === "write_failed" && hasCodeAndCause(value) && isAtomicWriteError(value.cause)
  );
}

function isAtomicWriteError(value: unknown): value is AtomicDocumentWriteError {
  return (
    isRecord(value) &&
    hasOnlyCode(value) &&
    (value.code === "open_temporary_file" ||
      value.code === "write_temporary_file" ||
      value.code === "sync_temporary_file" ||
      value.code === "replace_target" ||
      value.code === "cleanup_temporary_file" ||
      value.code === "sync_parent_directory")
  );
}

function isInvalidEnvelopeError(
  value: Record<string, unknown>,
): value is { code: "invalid_envelope"; cause: DocumentEnvelopeError } {
  return (
    value.code === "invalid_envelope" &&
    hasCodeAndCause(value) &&
    isDocumentEnvelopeError(value.cause)
  );
}

function isRegistryCommandError(
  value: Record<string, unknown>,
): value is { code: "registry"; cause: DocumentRegistryError } {
  return value.code === "registry" && hasCodeAndCause(value) && isRegistryError(value.cause);
}

function isRegistryError(value: unknown): value is DocumentRegistryError {
  return (
    isRecord(value) &&
    hasOnlyCode(value) &&
    (value.code === "already_open" ||
      value.code === "not_open" ||
      value.code === "source_path_in_use" ||
      value.code === "registry_unavailable")
  );
}

function hasOnlyCode(value: Record<string, unknown>): boolean {
  return Object.keys(value).length === 1;
}

function hasCodeAndCause(value: Record<string, unknown>): boolean {
  const keys = Object.keys(value);
  return keys.length === 2 && keys.includes("code") && keys.includes("cause");
}
