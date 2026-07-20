import {
  isDocumentEnvelopeError,
  isRecord,
  type DocumentEnvelopeError,
} from "./documentEnvelope";
import { isExternalFidelity, type ExternalFidelity } from "./externalDocument";

export type DocumentRegistryError = {
  code:
    | "already_open"
    | "not_open"
    | "external_source_unavailable"
    | "source_path_in_use"
    | "registry_unavailable";
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
  | {
      code:
        | "unsupported_file_location"
        | "unsupported_file_type"
        | "file_not_found"
        | "read_failed"
        | "malformed_json"
        | "invalid_text_encoding"
        | "text_too_large";
    }
  | { code: "invalid_envelope"; cause: DocumentEnvelopeError }
  | { code: "external_import"; cause: ExternalDocumentImportError }
  | { code: "registry"; cause: DocumentRegistryError };

export type ExternalDocumentImportError =
  | { code: "file_not_found" | "read_failed" | "package_too_large" }
  | { code: "docx"; cause: DocxImportError }
  | { code: "invalid_canonical_document"; cause: DocumentEnvelopeError };

export type DocxImportError =
  | {
      code: "malformed_package";
      fidelity: Extract<ExternalFidelity, { classification: "malformed_external_input" }>;
    }
  | {
      code: "unsafe_package";
      fidelity: Extract<ExternalFidelity, { classification: "unsafe" }>;
    }
  | {
      code: "unsupported_external_feature";
      fidelity: Extract<ExternalFidelity, { classification: "unsupported_external_feature" }>;
    }
  | {
      code: "lossy_import_denied";
      fidelity: Extract<ExternalFidelity, { classification: "lossy" }>;
    };

export type SaveDocumentCommandError =
  | {
      code:
        | "unsupported_file_location"
        | "invalid_operation"
        | "unsupported_format"
        | "invalid_target"
        | "serialization_failed"
        | "durability_uncertain";
    }
  | { code: "save_as_target"; cause: SaveAsTargetError }
  | { code: "write_failed"; cause: AtomicDocumentWriteError }
  | { code: "invalid_envelope"; cause: DocumentEnvelopeError }
  | { code: "registry"; cause: DocumentRegistryError }
  | { code: "docx_conversion"; cause: DocxExportError }
  | { code: "plain_text_conversion"; cause: PlainTextExportError };

export type SaveAsTargetError = {
  code:
    | "invalid_file_name"
    | "conflicting_extension"
    | "extension_mismatch"
    | "target_is_directory";
};

export type DocxExportError = {
  code:
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
};

export type PlainTextExportError = {
  code:
    | "durability_uncertain"
    | "invalid_document_structure"
    | "invalid_target"
    | "nesting_too_deep"
    | "output_too_large"
    | "too_many_nodes"
    | "write_failed";
};

export function isOpenDocumentCommandError(value: unknown): value is OpenDocumentCommandError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }

  return (
    isFieldlessOpenError(value) ||
    isExternalImportError(value) ||
    isInvalidEnvelopeError(value) ||
    isRegistryCommandError(value)
  );
}

function isExternalImportError(
  value: Record<string, unknown>,
): value is { code: "external_import"; cause: ExternalDocumentImportError } {
  return (
    value.code === "external_import" &&
    hasCodeAndCause(value) &&
    isExternalDocumentImportError(value.cause)
  );
}

function isExternalDocumentImportError(value: unknown): value is ExternalDocumentImportError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }
  if (
    hasOnlyCode(value) &&
    (value.code === "file_not_found" ||
      value.code === "read_failed" ||
      value.code === "package_too_large")
  ) {
    return true;
  }
  if (value.code === "docx" && hasCodeAndCause(value)) {
    return isDocxImportError(value.cause);
  }
  return (
    value.code === "invalid_canonical_document" &&
    hasCodeAndCause(value) &&
    isDocumentEnvelopeError(value.cause)
  );
}

function isDocxImportError(value: unknown): value is DocxImportError {
  if (!isRecord(value) || !hasCodeAndFidelity(value) || !isExternalFidelity(value.fidelity)) {
    return false;
  }
  switch (value.code) {
    case "malformed_package":
      return value.fidelity.classification === "malformed_external_input";
    case "unsafe_package":
      return value.fidelity.classification === "unsafe";
    case "unsupported_external_feature":
      return value.fidelity.classification === "unsupported_external_feature";
    case "lossy_import_denied":
      return value.fidelity.classification === "lossy";
    default:
      return false;
  }
}

export function isSaveDocumentCommandError(value: unknown): value is SaveDocumentCommandError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }

  return (
    isFieldlessSaveError(value) ||
    isSaveAsTargetFailure(value) ||
    isWriteFailure(value) ||
    isInvalidEnvelopeError(value) ||
    isRegistryCommandError(value) ||
    isConversionFailure(value)
  );
}

function isFieldlessOpenError(value: Record<string, unknown>): boolean {
  return (
    hasOnlyCode(value) &&
    (value.code === "unsupported_file_location" ||
      value.code === "unsupported_file_type" ||
      value.code === "file_not_found" ||
      value.code === "read_failed" ||
      value.code === "malformed_json" ||
      value.code === "invalid_text_encoding" ||
      value.code === "text_too_large")
  );
}

function isFieldlessSaveError(value: Record<string, unknown>): boolean {
  return (
    hasOnlyCode(value) &&
    (value.code === "unsupported_file_location" ||
      value.code === "invalid_operation" ||
      value.code === "unsupported_format" ||
      value.code === "invalid_target" ||
      value.code === "serialization_failed" ||
      value.code === "durability_uncertain")
  );
}

function isSaveAsTargetFailure(
  value: Record<string, unknown>,
): value is { code: "save_as_target"; cause: SaveAsTargetError } {
  return (
    value.code === "save_as_target" &&
    hasCodeAndCause(value) &&
    isSaveAsTargetError(value.cause)
  );
}

function isSaveAsTargetError(value: unknown): value is SaveAsTargetError {
  return (
    isRecord(value) &&
    hasOnlyCode(value) &&
    (value.code === "invalid_file_name" ||
      value.code === "conflicting_extension" ||
      value.code === "extension_mismatch" ||
      value.code === "target_is_directory")
  );
}

function isConversionFailure(
  value: Record<string, unknown>,
): value is
  | { code: "docx_conversion"; cause: DocxExportError }
  | { code: "plain_text_conversion"; cause: PlainTextExportError } {
  if (!hasCodeAndCause(value)) return false;
  if (value.code === "docx_conversion") return isDocxExportError(value.cause);
  return value.code === "plain_text_conversion" && isPlainTextExportError(value.cause);
}

export function isDocxExportError(value: unknown): value is DocxExportError {
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

function isPlainTextExportError(value: unknown): value is PlainTextExportError {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    (value.code === "durability_uncertain" ||
      value.code === "invalid_document_structure" ||
      value.code === "invalid_target" ||
      value.code === "nesting_too_deep" ||
      value.code === "output_too_large" ||
      value.code === "too_many_nodes" ||
      value.code === "write_failed")
  );
}

function isWriteFailure(
  value: Record<string, unknown>,
): value is { code: "write_failed"; cause: AtomicDocumentWriteError } {
  return (
    value.code === "write_failed" &&
    hasCodeAndCause(value) &&
    isAtomicDocumentWriteError(value.cause)
  );
}

export function isAtomicDocumentWriteError(
  value: unknown,
): value is AtomicDocumentWriteError {
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
  return (
    value.code === "registry" &&
    hasCodeAndCause(value) &&
    isDocumentRegistryError(value.cause)
  );
}

export function isDocumentRegistryError(value: unknown): value is DocumentRegistryError {
  return (
    isRecord(value) &&
    hasOnlyCode(value) &&
    (value.code === "already_open" ||
      value.code === "not_open" ||
      value.code === "external_source_unavailable" ||
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

function hasCodeAndFidelity(value: Record<string, unknown>): boolean {
  const keys = Object.keys(value);
  return keys.length === 2 && keys.includes("code") && keys.includes("fidelity");
}
