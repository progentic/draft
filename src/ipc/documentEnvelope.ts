export interface DocumentEnvelopeSnapshot {
  schema_version: 1;
  document_id: string;
  title: string;
  document: {
    type: "doc";
    content: unknown[];
    [key: string]: unknown;
  };
}

export type DocumentEnvelopeError =
  | { code: "invalid_envelope_object" }
  | { code: "unknown_envelope_field"; field: string }
  | { code: "missing_schema_version" }
  | { code: "invalid_schema_version" }
  | { code: "unsupported_schema_version"; found: number }
  | { code: "missing_document_id" }
  | { code: "invalid_document_id" }
  | { code: "missing_title" }
  | { code: "invalid_title" }
  | { code: "missing_document" }
  | { code: "invalid_document_root" }
  | { code: "invalid_document_content" };

const ENVELOPE_FIELDS = ["schema_version", "document_id", "title", "document"];
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/iu;

/** Checks Rust response data for UI safety without replacing Rust validation. */
export function isDocumentEnvelopeSnapshot(value: unknown): value is DocumentEnvelopeSnapshot {
  return (
    isRecord(value) &&
    hasExactFields(value, ENVELOPE_FIELDS) &&
    value.schema_version === 1 &&
    isDocumentId(value.document_id) &&
    typeof value.title === "string" &&
    value.title.trim().length > 0 &&
    isDocumentRoot(value.document)
  );
}

export function isDocumentEnvelopeError(value: unknown): value is DocumentEnvelopeError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }

  return hasValidEnvelopeErrorFields(value);
}

export function isDocumentId(value: unknown): value is string {
  return typeof value === "string" && UUID_PATTERN.test(value);
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isDocumentRoot(value: unknown): value is DocumentEnvelopeSnapshot["document"] {
  return isRecord(value) && value.type === "doc" && Array.isArray(value.content);
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}

function hasValidEnvelopeErrorFields(value: Record<string, unknown>): boolean {
  switch (value.code) {
    case "unknown_envelope_field":
      return hasExactFields(value, ["code", "field"]) && typeof value.field === "string";
    case "unsupported_schema_version":
      return hasExactFields(value, ["code", "found"]) && Number.isSafeInteger(value.found);
    default:
      return hasExactFields(value, ["code"]) && isFieldlessEnvelopeErrorCode(value.code);
  }
}

function isFieldlessEnvelopeErrorCode(value: unknown): boolean {
  return (
    value === "invalid_envelope_object" ||
    value === "missing_schema_version" ||
    value === "invalid_schema_version" ||
    value === "missing_document_id" ||
    value === "invalid_document_id" ||
    value === "missing_title" ||
    value === "invalid_title" ||
    value === "missing_document" ||
    value === "invalid_document_root" ||
    value === "invalid_document_content"
  );
}
