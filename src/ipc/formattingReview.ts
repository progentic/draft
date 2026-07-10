import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export const FORMATTING_STYLES = [
  "apa7",
  "mla9",
  "chicago17_author_date",
] as const;
export const DEFAULT_FORMATTING_STYLE: FormattingStyle = "apa7";

export type FormattingStyle = (typeof FORMATTING_STYLES)[number];

export interface FormattingHeadingInput {
  level: number;
  title: string;
}

export interface FormattingCitationInput {
  citekey: string;
  renderStyle: FormattingStyle;
}

export interface FormattingReviewRequest {
  style: FormattingStyle;
  headings: FormattingHeadingInput[];
  citations: FormattingCitationInput[];
}

export type FormattingFindingCode =
  | "first_heading_not_level_one"
  | "heading_level_skipped"
  | "citation_style_mismatch";

export type FormattingSeverity = "advice" | "warning";

export type FormattingTarget =
  | { type: "heading"; index: number }
  | { type: "citation"; index: number };

export type FormattingAction =
  | { type: "inspect" }
  | { type: "apply_heading_level"; level: number }
  | { type: "dismiss" };

export interface FormattingReviewFinding {
  code: FormattingFindingCode;
  severity: FormattingSeverity;
  target: FormattingTarget;
  title: string;
  explanation: string;
  actions: FormattingAction[];
}

export interface FormattingReviewResponse {
  style: FormattingStyle;
  findings: FormattingReviewFinding[];
}

export type FormattingReviewCommandErrorCode =
  | "too_many_headings"
  | "too_many_citations"
  | "invalid_heading_level"
  | "empty_heading_title"
  | "heading_title_too_long"
  | "invalid_citekey";

export type FormattingReviewClientError =
  | { type: "command"; code: FormattingReviewCommandErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type FormattingReviewResult =
  | { status: "ready"; review: FormattingReviewResponse }
  | { status: "error"; error: FormattingReviewClientError };

const COMMAND_NAME = "run_formatting_review";
const MAX_FINDING_TITLE_LENGTH = 128;
const MAX_FINDING_EXPLANATION_LENGTH = 512;
const COMMAND_ERROR_CODES = new Set<FormattingReviewCommandErrorCode>([
  "too_many_headings",
  "too_many_citations",
  "invalid_heading_level",
  "empty_heading_title",
  "heading_title_too_long",
  "invalid_citekey",
]);

/** Runs the bounded Rust formatting review and validates its closed response. */
export async function runFormattingReview(
  request: FormattingReviewRequest,
): Promise<FormattingReviewResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request });
    return isFormattingReviewResponse(response, request)
      ? { status: "ready", review: response }
      : { status: "error", error: { type: "invalid-response" } };
  } catch (error: unknown) {
    return { status: "error", error: formattingReviewClientErrorFrom(error) };
  }
}

function formattingReviewClientErrorFrom(error: unknown): FormattingReviewClientError {
  return isFormattingReviewCommandError(error)
    ? { type: "command", code: error.code }
    : { type: "transport" };
}

function isFormattingReviewResponse(
  value: unknown,
  request: FormattingReviewRequest,
): value is FormattingReviewResponse {
  return (
    isRecord(value) &&
    hasExactFields(value, ["style", "findings"]) &&
    value.style === request.style &&
    Array.isArray(value.findings) &&
    value.findings.length <= request.headings.length + request.citations.length &&
    value.findings.every((finding) => isFormattingFinding(finding, request))
  );
}

function isFormattingFinding(
  value: unknown,
  request: FormattingReviewRequest,
): value is FormattingReviewFinding {
  if (!isRecord(value) || !hasExactFields(value, FINDING_FIELDS)) {
    return false;
  }
  if (!isFormattingFindingCode(value.code) || !isFormattingTarget(value.target, request)) {
    return false;
  }
  return (
    value.severity === expectedSeverity(value.code) &&
    isBoundedText(value.title, MAX_FINDING_TITLE_LENGTH) &&
    isBoundedText(value.explanation, MAX_FINDING_EXPLANATION_LENGTH) &&
    isExpectedTarget(value.code, value.target) &&
    isExpectedActions(value.actions, value.code, value.target, request)
  );
}

function isFormattingTarget(
  value: unknown,
  request: FormattingReviewRequest,
): value is FormattingTarget {
  if (!isRecord(value) || !hasExactFields(value, ["type", "index"])) {
    return false;
  }
  if (!Number.isSafeInteger(value.index) || Number(value.index) < 0) {
    return false;
  }
  const index = Number(value.index);
  return value.type === "heading"
    ? index < request.headings.length
    : value.type === "citation" && index < request.citations.length;
}

function isExpectedTarget(code: FormattingFindingCode, target: FormattingTarget) {
  return code === "citation_style_mismatch"
    ? target.type === "citation"
    : target.type === "heading";
}

function isExpectedActions(
  value: unknown,
  code: FormattingFindingCode,
  target: FormattingTarget,
  request: FormattingReviewRequest,
) {
  if (!Array.isArray(value)) {
    return false;
  }
  const expectedLevel = expectedHeadingLevel(code, target, request);
  if (expectedLevel === undefined) {
    return hasActionTypes(value, ["inspect", "dismiss"]);
  }
  return (
    value.length === 3 &&
    isAction(value[0], "inspect") &&
    isApplyAction(value[1], expectedLevel) &&
    isAction(value[2], "dismiss")
  );
}

function expectedHeadingLevel(
  code: FormattingFindingCode,
  target: FormattingTarget,
  request: FormattingReviewRequest,
) {
  if (target.type !== "heading") {
    return undefined;
  }
  if (code === "first_heading_not_level_one" && target.index === 0) {
    return 1;
  }
  if (code !== "heading_level_skipped" || target.index === 0) {
    return undefined;
  }
  return request.headings[target.index - 1]?.level + 1;
}

function hasActionTypes(value: unknown[], types: string[]) {
  return value.length === types.length && value.every((action, index) => isAction(action, types[index]));
}

function isAction(value: unknown, type: string) {
  return isRecord(value) && hasExactFields(value, ["type"]) && value.type === type;
}

function isApplyAction(value: unknown, level: number) {
  return (
    isRecord(value) &&
    hasExactFields(value, ["type", "level"]) &&
    value.type === "apply_heading_level" &&
    value.level === level
  );
}

function expectedSeverity(code: FormattingFindingCode): FormattingSeverity {
  return code === "first_heading_not_level_one" ? "advice" : "warning";
}

function isFormattingFindingCode(value: unknown): value is FormattingFindingCode {
  return (
    value === "first_heading_not_level_one" ||
    value === "heading_level_skipped" ||
    value === "citation_style_mismatch"
  );
}

function isFormattingReviewCommandError(
  value: unknown,
): value is { code: FormattingReviewCommandErrorCode } {
  return (
    isRecord(value) &&
    hasExactFields(value, ["code"]) &&
    typeof value.code === "string" &&
    COMMAND_ERROR_CODES.has(value.code as FormattingReviewCommandErrorCode)
  );
}

function isBoundedText(value: unknown, maximumLength: number) {
  return typeof value === "string" && value.length > 0 && value.length <= maximumLength;
}

const FINDING_FIELDS = [
  "code",
  "severity",
  "target",
  "title",
  "explanation",
  "actions",
];

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
