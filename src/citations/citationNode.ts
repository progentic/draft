export const CITATION_NODE_SCHEMA_VERSION = 1 as const;

export interface CitationNodeAttributes {
  schema_version: typeof CITATION_NODE_SCHEMA_VERSION;
  citekey: string;
  render_style: "apa7";
}

export type CitationNodeError =
  | { code: "unknown_citation_node_field"; field: string }
  | { code: "missing_citation_attrs" }
  | { code: "invalid_citation_attrs_object" }
  | { code: "unknown_citation_attr"; field: string }
  | { code: "missing_schema_version" }
  | { code: "invalid_schema_version" }
  | { code: "unsupported_schema_version"; found: number }
  | { code: "missing_citekey" }
  | { code: "invalid_citekey" }
  | { code: "missing_render_style" }
  | { code: "unsupported_render_style" };

export type CitationNodeValidation =
  | { valid: true; attrs: CitationNodeAttributes }
  | { valid: false; error: CitationNodeError };

const ATTRIBUTE_FIELDS = ["schema_version", "citekey", "render_style"];
const NODE_FIELDS = ["type", "attrs"];
const CITEKEY_PATTERN = /^[A-Za-z0-9][A-Za-z0-9:_-]*$/u;
type FieldlessCitationErrorCode =
  | "missing_citation_attrs"
  | "invalid_citation_attrs_object"
  | "missing_schema_version"
  | "invalid_schema_version"
  | "missing_citekey"
  | "invalid_citekey"
  | "missing_render_style"
  | "unsupported_render_style";

/** Mirrors Rust validation so untrusted editor and IPC data fail safely. */
export function validateCitationNodeAttributes(value: unknown): CitationNodeValidation {
  if (!isRecord(value)) {
    return invalid("invalid_citation_attrs_object");
  }

  const unknownField = firstUnknownField(value, ATTRIBUTE_FIELDS);
  if (unknownField !== undefined) {
    return { valid: false, error: { code: "unknown_citation_attr", field: unknownField } };
  }

  return validateRequiredAttributes(value);
}

export function isCitationNodeAttributes(value: unknown): value is CitationNodeAttributes {
  return validateCitationNodeAttributes(value).valid;
}

export function hasValidCitationNodes(document: unknown): boolean {
  return validateDocumentNode(document);
}

export function isCitationNodeError(value: unknown): value is CitationNodeError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }
  return isFieldError(value) || isVersionError(value) || isFieldlessError(value);
}

function validateRequiredAttributes(value: Record<string, unknown>): CitationNodeValidation {
  const schemaError = validateSchemaVersion(value);
  if (schemaError !== undefined) {
    return { valid: false, error: schemaError };
  }
  const citekeyError = validateCitekey(value);
  if (citekeyError !== undefined) {
    return { valid: false, error: citekeyError };
  }
  return validateRenderStyle(value);
}

function validateSchemaVersion(value: Record<string, unknown>): CitationNodeError | undefined {
  if (!hasOwn(value, "schema_version")) {
    return { code: "missing_schema_version" };
  }
  if (!Number.isSafeInteger(value.schema_version) || Number(value.schema_version) < 0) {
    return { code: "invalid_schema_version" };
  }
  if (value.schema_version !== CITATION_NODE_SCHEMA_VERSION) {
    return { code: "unsupported_schema_version", found: Number(value.schema_version) };
  }
  return undefined;
}

function validateCitekey(value: Record<string, unknown>): CitationNodeError | undefined {
  if (!hasOwn(value, "citekey")) {
    return { code: "missing_citekey" };
  }
  if (typeof value.citekey !== "string" || !CITEKEY_PATTERN.test(value.citekey)) {
    return { code: "invalid_citekey" };
  }
  return undefined;
}

function validateRenderStyle(value: Record<string, unknown>): CitationNodeValidation {
  if (!hasOwn(value, "render_style")) {
    return invalid("missing_render_style");
  }
  if (value.render_style !== "apa7") {
    return invalid("unsupported_render_style");
  }
  return {
    valid: true,
    attrs: {
      schema_version: CITATION_NODE_SCHEMA_VERSION,
      citekey: value.citekey as string,
      render_style: "apa7",
    },
  };
}

function validateDocumentNode(value: unknown): boolean {
  if (!isRecord(value)) {
    return true;
  }
  if (value.type === "citation" && !isValidCitationNode(value)) {
    return false;
  }
  return !Array.isArray(value.content) || value.content.every(validateDocumentNode);
}

function isValidCitationNode(value: Record<string, unknown>): boolean {
  return firstUnknownField(value, NODE_FIELDS) === undefined && isCitationNodeAttributes(value.attrs);
}

function isFieldError(value: Record<string, unknown>): boolean {
  return (
    hasExactFields(value, ["code", "field"]) &&
    typeof value.field === "string" &&
    (value.code === "unknown_citation_node_field" || value.code === "unknown_citation_attr")
  );
}

function isVersionError(value: Record<string, unknown>): boolean {
  return (
    value.code === "unsupported_schema_version" &&
    hasExactFields(value, ["code", "found"]) &&
    Number.isSafeInteger(value.found) &&
    Number(value.found) >= 0
  );
}

function isFieldlessError(value: Record<string, unknown>): boolean {
  return hasExactFields(value, ["code"]) && FIELDLESS_ERROR_CODES.has(String(value.code));
}

const FIELDLESS_ERROR_CODES = new Set([
  "missing_citation_attrs",
  "invalid_citation_attrs_object",
  "missing_schema_version",
  "invalid_schema_version",
  "missing_citekey",
  "invalid_citekey",
  "missing_render_style",
  "unsupported_render_style",
]);

function invalid(code: FieldlessCitationErrorCode): CitationNodeValidation {
  return { valid: false, error: { code } };
}

function firstUnknownField(value: Record<string, unknown>, allowed: string[]) {
  return Object.keys(value)
    .sort()
    .find((field) => !allowed.includes(field));
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}

function hasOwn(value: Record<string, unknown>, field: string) {
  return Object.prototype.hasOwnProperty.call(value, field);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
