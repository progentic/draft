export const FONT_FAMILIES = [
  { id: "arial", label: "Arial", css: "Arial, sans-serif" },
  { id: "georgia", label: "Georgia", css: "Georgia, serif" },
  { id: "times_new_roman", label: "Times New Roman", css: '"Times New Roman", serif' },
  { id: "courier_new", label: "Courier New", css: '"Courier New", monospace' },
] as const;

export const MIN_FONT_SIZE_POINTS = 8;
export const MAX_FONT_SIZE_POINTS = 72;

export type FontFamilyId = (typeof FONT_FAMILIES)[number]["id"];

export type TextFormatError =
  | { code: "invalid_mark_object" }
  | { code: "missing_mark_type" }
  | { code: "invalid_mark_type" }
  | { code: "unknown_font_mark_field"; field: string }
  | { code: "missing_attrs" }
  | { code: "invalid_attrs_object" }
  | { code: "unknown_font_attr"; field: string }
  | { code: "missing_font_family" }
  | { code: "unsupported_font_family" }
  | { code: "missing_font_size" }
  | { code: "invalid_font_size" };

export function isFontFamilyId(value: unknown): value is FontFamilyId {
  return FONT_FAMILIES.some((family) => family.id === value);
}

export function isFontSizePoints(value: unknown): value is number {
  return (
    Number.isInteger(value) &&
    Number(value) >= MIN_FONT_SIZE_POINTS &&
    Number(value) <= MAX_FONT_SIZE_POINTS
  );
}

export function fontFamilyCss(value: unknown): string | null {
  return FONT_FAMILIES.find((family) => family.id === value)?.css ?? null;
}

export function hasValidTextFormatting(value: unknown): boolean {
  return validateNode(value);
}

export function isTextFormatError(value: unknown): value is TextFormatError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }
  if (value.code === "unknown_font_mark_field" || value.code === "unknown_font_attr") {
    return hasExactFields(value, ["code", "field"]) && typeof value.field === "string";
  }
  return hasExactFields(value, ["code"]) && isFieldlessErrorCode(value.code);
}

function validateNode(value: unknown): boolean {
  if (!isRecord(value)) {
    return true;
  }
  if (!validateMarks(value.marks)) {
    return false;
  }
  return !Array.isArray(value.content) || value.content.every(validateNode);
}

function validateMarks(value: unknown): boolean {
  if (!Array.isArray(value)) {
    return true;
  }
  return value.every(validateMark);
}

function validateMark(value: unknown): boolean {
  if (!isRecord(value)) {
    return false;
  }
  if (typeof value.type !== "string") {
    return false;
  }
  if (value.type === "fontFamily") {
    return validateFontFamilyMark(value);
  }
  if (value.type === "fontSize") {
    return validateFontSizeMark(value);
  }
  return true;
}

function validateFontFamilyMark(mark: Record<string, unknown>): boolean {
  return (
    hasExactFields(mark, ["type", "attrs"]) &&
    isRecord(mark.attrs) &&
    hasExactFields(mark.attrs, ["family"]) &&
    isFontFamilyId(mark.attrs.family)
  );
}

function validateFontSizeMark(mark: Record<string, unknown>): boolean {
  return (
    hasExactFields(mark, ["type", "attrs"]) &&
    isRecord(mark.attrs) &&
    hasExactFields(mark.attrs, ["points"]) &&
    isFontSizePoints(mark.attrs.points)
  );
}

function isFieldlessErrorCode(value: unknown): boolean {
  return (
    value === "invalid_mark_object" ||
    value === "missing_mark_type" ||
    value === "invalid_mark_type" ||
    value === "missing_attrs" ||
    value === "invalid_attrs_object" ||
    value === "missing_font_family" ||
    value === "unsupported_font_family" ||
    value === "missing_font_size" ||
    value === "invalid_font_size"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
