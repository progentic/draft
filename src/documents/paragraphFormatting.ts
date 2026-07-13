export const PARAGRAPH_STYLE_SCHEMA_VERSION = 1 as const;
export const MIN_LINE_SPACING_HUNDREDTHS = 100;
export const MAX_LINE_SPACING_HUNDREDTHS = 300;
export const LINE_SPACING_INCREMENT = 5;
export const MAX_PARAGRAPH_SPACING_TWIPS = 2_880;
export const MAX_SPECIAL_INDENT_TWIPS = 1_440;

export type ParagraphAlignment = "left" | "center" | "right" | "justify";
export type SpecialIndentKind = "none" | "first_line" | "hanging";

export interface ParagraphStyle {
  schemaVersion: typeof PARAGRAPH_STYLE_SCHEMA_VERSION;
  alignment: ParagraphAlignment;
  lineSpacingHundredths: number;
  spaceBeforeTwips: number;
  spaceAfterTwips: number;
  leftIndentTwips: number;
  rightIndentTwips: number;
  specialIndent: {
    kind: SpecialIndentKind;
    twips: number;
  };
}

export type ParagraphStyleError =
  | { code: "unsupported_block" }
  | { code: "invalid_style_object" }
  | { code: "missing_style_field"; field: string }
  | { code: "unknown_style_field"; field: string }
  | { code: "invalid_style_schema_version" }
  | { code: "unsupported_style_schema_version"; found: number }
  | { code: "invalid_alignment" }
  | { code: "invalid_line_spacing" }
  | { code: "invalid_paragraph_spacing" }
  | { code: "invalid_paragraph_indent" }
  | { code: "invalid_special_indent_object" }
  | { code: "missing_special_indent_field"; field: string }
  | { code: "unknown_special_indent_field"; field: string }
  | { code: "invalid_special_indent_kind" }
  | { code: "invalid_special_indent_amount" };

const STYLE_FIELDS = [
  "schemaVersion",
  "alignment",
  "lineSpacingHundredths",
  "spaceBeforeTwips",
  "spaceAfterTwips",
  "leftIndentTwips",
  "rightIndentTwips",
  "specialIndent",
] as const;
const SPECIAL_INDENT_FIELDS = ["kind", "twips"] as const;

export function hasValidParagraphStyles(document: unknown): boolean {
  return validateNode(document);
}

export function parseParagraphStyle(value: unknown): ParagraphStyle | null {
  return isParagraphStyle(value) ? value : null;
}

export function omitUnsetParagraphStyles<T>(value: T): T {
  return omitUnsetValue(value) as T;
}

export function isParagraphStyleError(value: unknown): value is ParagraphStyleError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }
  if (isFieldError(value)) {
    return hasExactFields(value, ["code", "field"]) && typeof value.field === "string";
  }
  if (value.code === "unsupported_style_schema_version") {
    return hasExactFields(value, ["code", "found"]) && Number.isSafeInteger(value.found);
  }
  return hasExactFields(value, ["code"]) && FIELDLESS_ERROR_CODES.has(value.code);
}

function validateNode(value: unknown): boolean {
  if (!isRecord(value)) {
    return true;
  }
  return hasValidNodeStyle(value) && hasValidChildren(value.content);
}

function hasValidNodeStyle(node: Record<string, unknown>): boolean {
  if (!isRecord(node.attrs) || !("paragraphStyle" in node.attrs)) {
    return true;
  }
  return isSupportedBlock(node.type) && isParagraphStyle(node.attrs.paragraphStyle);
}

function hasValidChildren(value: unknown): boolean {
  return !Array.isArray(value) || value.every(validateNode);
}

function isParagraphStyle(value: unknown): value is ParagraphStyle {
  if (!isRecord(value) || !hasExactFields(value, STYLE_FIELDS)) {
    return false;
  }
  return (
    value.schemaVersion === PARAGRAPH_STYLE_SCHEMA_VERSION &&
    isAlignment(value.alignment) &&
    isLineSpacing(value.lineSpacingHundredths) &&
    isBoundedInteger(value.spaceBeforeTwips, MAX_PARAGRAPH_SPACING_TWIPS) &&
    isBoundedInteger(value.spaceAfterTwips, MAX_PARAGRAPH_SPACING_TWIPS) &&
    isBoundedInteger(value.leftIndentTwips, MAX_PARAGRAPH_SPACING_TWIPS) &&
    isBoundedInteger(value.rightIndentTwips, MAX_PARAGRAPH_SPACING_TWIPS) &&
    isSpecialIndent(value.specialIndent)
  );
}

function omitUnsetValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(omitUnsetValue);
  }
  if (!isRecord(value)) {
    return value;
  }
  return Object.fromEntries(
    Object.entries(value)
      .filter(([key, entry]) => key !== "paragraphStyle" || entry != null)
      .map(([key, entry]) => [key, omitUnsetValue(entry)]),
  );
}

function isSpecialIndent(value: unknown): value is ParagraphStyle["specialIndent"] {
  if (!isRecord(value) || !hasExactFields(value, SPECIAL_INDENT_FIELDS)) {
    return false;
  }
  if (!isSpecialIndentKind(value.kind) || !isBoundedInteger(value.twips, MAX_SPECIAL_INDENT_TWIPS)) {
    return false;
  }
  return value.kind !== "none" || value.twips === 0;
}

function isSupportedBlock(value: unknown): boolean {
  return value === "paragraph" || value === "heading";
}

function isAlignment(value: unknown): value is ParagraphAlignment {
  return value === "left" || value === "center" || value === "right" || value === "justify";
}

function isSpecialIndentKind(value: unknown): value is SpecialIndentKind {
  return value === "none" || value === "first_line" || value === "hanging";
}

function isLineSpacing(value: unknown): boolean {
  return (
    Number.isSafeInteger(value) &&
    Number(value) >= MIN_LINE_SPACING_HUNDREDTHS &&
    Number(value) <= MAX_LINE_SPACING_HUNDREDTHS &&
    Number(value) % LINE_SPACING_INCREMENT === 0
  );
}

function isBoundedInteger(value: unknown, maximum: number): boolean {
  return Number.isSafeInteger(value) && Number(value) >= 0 && Number(value) <= maximum;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactFields(value: Record<string, unknown>, fields: readonly string[]): boolean {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}

function isFieldError(value: Record<string, unknown>): boolean {
  return (
    value.code === "missing_style_field" ||
    value.code === "unknown_style_field" ||
    value.code === "missing_special_indent_field" ||
    value.code === "unknown_special_indent_field"
  );
}

const FIELDLESS_ERROR_CODES = new Set([
  "unsupported_block",
  "invalid_style_object",
  "invalid_style_schema_version",
  "invalid_alignment",
  "invalid_line_spacing",
  "invalid_paragraph_spacing",
  "invalid_paragraph_indent",
  "invalid_special_indent_object",
  "invalid_special_indent_kind",
  "invalid_special_indent_amount",
]);
