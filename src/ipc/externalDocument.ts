import { isRecord } from "./documentEnvelope";

export type ExternalDocumentFormat = "docx";

export type ExternalFeature =
  | "alternate_heading_style_name"
  | "at_least_line_spacing"
  | "contextual_spacing"
  | "exact_line_spacing"
  | "external_relationship"
  | "list_indentation"
  | "package_part"
  | "pagination_control"
  | "paragraph_border"
  | "paragraph_shading"
  | "paragraph_tab"
  | "run_formatting"
  | "unsupported_document_structure"
  | "unsupported_style_inheritance";

export type ExternalNormalizationFeature =
  | "alternate_heading_style_name"
  | "pagination_control";

export type ExternalSafetyReason =
  | "archive_entry_count"
  | "archive_entry_size"
  | "archive_path"
  | "archive_uncompressed_size"
  | "compression_ratio"
  | "duplicate_entry"
  | "encrypted_entry"
  | "package_size"
  | "relationship_target"
  | "symlink_entry"
  | "xml_node_count"
  | "xml_doctype"
  | "xml_depth"
  | "xml_entity"
  | "xml_size";

export type ExternalFidelity =
  | { classification: "exact" }
  | { classification: "canonically_normalized"; features: ExternalFeature[] }
  | { classification: "unsupported_preservable"; features: ExternalFeature[] }
  | { classification: "lossy"; features: ExternalFeature[] }
  | { classification: "malformed_external_input" }
  | { classification: "unsupported_external_feature"; features: ExternalFeature[] }
  | { classification: "unsafe"; reason: ExternalSafetyReason };

export type ImportedExternalFidelity = Extract<
  ExternalFidelity,
  {
    classification: "exact" | "canonically_normalized" | "unsupported_preservable";
  }
>;

export type SameFormatSaveDisposition =
  | "no_changes"
  | "allowed_exact"
  | "allowed_after_accepted_normalization"
  | "denied_unsupported_source_behavior"
  | "denied_read_only"
  | "denied_missing_provenance"
  | "denied_fidelity_unknown"
  | "denied_writer_unavailable"
  | "denied_source_missing"
  | "denied_source_changed";

export interface ExternalDocumentSummary {
  format: ExternalDocumentFormat;
  displayName: string;
  fidelity: ImportedExternalFidelity;
  sameFormatSave: SameFormatSaveDisposition;
}

const EXTERNAL_FEATURES: ExternalFeature[] = [
  "alternate_heading_style_name",
  "at_least_line_spacing",
  "contextual_spacing",
  "exact_line_spacing",
  "external_relationship",
  "list_indentation",
  "package_part",
  "pagination_control",
  "paragraph_border",
  "paragraph_shading",
  "paragraph_tab",
  "run_formatting",
  "unsupported_document_structure",
  "unsupported_style_inheritance",
];

const EXTERNAL_SAFETY_REASONS: ExternalSafetyReason[] = [
  "archive_entry_count",
  "archive_entry_size",
  "archive_path",
  "archive_uncompressed_size",
  "compression_ratio",
  "duplicate_entry",
  "encrypted_entry",
  "package_size",
  "relationship_target",
  "symlink_entry",
  "xml_node_count",
  "xml_doctype",
  "xml_depth",
  "xml_entity",
  "xml_size",
];

const SAME_FORMAT_SAVE_DISPOSITIONS: SameFormatSaveDisposition[] = [
  "no_changes",
  "allowed_exact",
  "allowed_after_accepted_normalization",
  "denied_unsupported_source_behavior",
  "denied_read_only",
  "denied_missing_provenance",
  "denied_fidelity_unknown",
  "denied_writer_unavailable",
  "denied_source_missing",
  "denied_source_changed",
];

export function isExternalDocumentSummary(value: unknown): value is ExternalDocumentSummary {
  return (
    isRecord(value) &&
    hasExactKeys(value, ["displayName", "fidelity", "format", "sameFormatSave"]) &&
    value.format === "docx" &&
    isDisplayName(value.displayName) &&
    isImportedExternalFidelity(value.fidelity) &&
    isSameFormatSaveDisposition(value.sameFormatSave)
  );
}

export function isExternalFidelity(value: unknown): value is ExternalFidelity {
  if (!isRecord(value) || typeof value.classification !== "string") {
    return false;
  }
  switch (value.classification) {
    case "exact":
    case "malformed_external_input":
      return hasExactKeys(value, ["classification"]);
    case "canonically_normalized":
    case "unsupported_preservable":
    case "lossy":
    case "unsupported_external_feature":
      return (
        hasExactKeys(value, ["classification", "features"]) &&
        isStableFeatureList(value.features)
      );
    case "unsafe":
      return (
        hasExactKeys(value, ["classification", "reason"]) &&
        isExternalSafetyReason(value.reason)
      );
    default:
      return false;
  }
}

export function isExternalNormalizationFeatureList(
  value: unknown,
): value is ExternalNormalizationFeature[] {
  if (!Array.isArray(value) || value.length === 0) {
    return false;
  }
  const allowed: ExternalNormalizationFeature[] = [
    "alternate_heading_style_name",
    "pagination_control",
  ];
  const positions = value.map((feature) => allowed.indexOf(feature));
  return positions.every(
    (position, index) =>
      position >= 0 && (index === 0 || positions[index - 1] < position),
  );
}

function isImportedExternalFidelity(value: unknown): value is ImportedExternalFidelity {
  return (
    isExternalFidelity(value) &&
    (value.classification === "exact" ||
      value.classification === "canonically_normalized" ||
      value.classification === "unsupported_preservable")
  );
}

function isStableFeatureList(value: unknown): value is ExternalFeature[] {
  if (!Array.isArray(value) || !value.every(isExternalFeature)) {
    return false;
  }
  const positions = value.map((feature) => EXTERNAL_FEATURES.indexOf(feature));
  return positions.every((position, index) => index === 0 || positions[index - 1] < position);
}

function isExternalFeature(value: unknown): value is ExternalFeature {
  return typeof value === "string" && EXTERNAL_FEATURES.includes(value as ExternalFeature);
}

function isExternalSafetyReason(value: unknown): value is ExternalSafetyReason {
  return (
    typeof value === "string" &&
    EXTERNAL_SAFETY_REASONS.includes(value as ExternalSafetyReason)
  );
}

function isSameFormatSaveDisposition(value: unknown): value is SameFormatSaveDisposition {
  return (
    typeof value === "string" &&
    SAME_FORMAT_SAVE_DISPOSITIONS.includes(value as SameFormatSaveDisposition)
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

function hasExactKeys(value: Record<string, unknown>, expected: string[]): boolean {
  const keys = Object.keys(value).sort();
  return keys.length === expected.length && keys.every((key, index) => key === expected[index]);
}
