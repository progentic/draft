import {
  isDocumentEnvelopeError,
  isRecord,
  type DocumentEnvelopeError,
} from "./documentEnvelope";

export type DocumentRegistryError = {
  code: "already_open" | "not_open" | "source_path_in_use" | "registry_unavailable";
};

export type OpenDocumentCommandError =
  | { code: "unsupported_file_location" | "file_not_found" | "read_failed" | "malformed_json" }
  | { code: "invalid_envelope"; cause: DocumentEnvelopeError }
  | { code: "registry"; cause: DocumentRegistryError };

export type SaveDocumentCommandError =
  | { code: "unsupported_file_location" | "serialization_failed" | "write_failed" }
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
      value.code === "write_failed")
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
