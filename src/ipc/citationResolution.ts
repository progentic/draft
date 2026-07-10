import {
  isCitationNodeAttributes,
  isCitationNodeError,
  type CitationNodeAttributes,
  type CitationNodeError,
} from "../citations/citationNode";
import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export interface ResolvedCitation {
  schemaVersion: 1;
  citekey: string;
  renderStyle: "apa7";
  displayMarker: string;
}

export type CitationStoreError = {
  code: "unavailable" | "read_failed" | "corrupt_reference";
};

export type CitationResolutionCommandError =
  | { code: "invalid_citation"; cause: CitationNodeError }
  | { code: "reference_not_found" }
  | { code: "reference_store"; cause: CitationStoreError };

export type CitationResolutionClientError =
  | { type: "command"; error: CitationResolutionCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type CitationResolutionResult =
  | { status: "resolved"; citation: ResolvedCitation }
  | { status: "error"; error: CitationResolutionClientError };

const COMMAND_NAME = "resolve_citation";

/** Resolves valid attrs through Rust without receiving reference metadata. */
export async function resolveCitation(
  attrs: CitationNodeAttributes,
): Promise<CitationResolutionResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: { attrs } });
    return resultFromResponse(response);
  } catch (error: unknown) {
    return resultFromFailure(error);
  }
}

function resultFromResponse(response: unknown): CitationResolutionResult {
  return isResolvedCitation(response)
    ? { status: "resolved", citation: response }
    : { status: "error", error: { type: "invalid-response" } };
}

function resultFromFailure(error: unknown): CitationResolutionResult {
  return { status: "error", error: citationClientErrorFrom(error) };
}

export function citationClientErrorFrom(error: unknown): CitationResolutionClientError {
  return isCitationResolutionCommandError(error)
    ? { type: "command", error }
    : { type: "transport" };
}

function isResolvedCitation(value: unknown): value is ResolvedCitation {
  if (!isRecord(value) || !hasExactFields(value, RESOLUTION_FIELDS)) {
    return false;
  }
  const attrs = {
    schema_version: value.schemaVersion,
    citekey: value.citekey,
    render_style: value.renderStyle,
  };
  return isCitationNodeAttributes(attrs) && value.displayMarker === `[@${attrs.citekey}]`;
}

function isCitationResolutionCommandError(
  value: unknown,
): value is CitationResolutionCommandError {
  if (!isRecord(value) || typeof value.code !== "string") {
    return false;
  }
  if (value.code === "reference_not_found") {
    return hasExactFields(value, ["code"]);
  }
  if (!hasExactFields(value, ["code", "cause"])) {
    return false;
  }
  return (
    (value.code === "invalid_citation" && isCitationNodeError(value.cause)) ||
    (value.code === "reference_store" && isCitationStoreError(value.cause))
  );
}

function isCitationStoreError(value: unknown): value is CitationStoreError {
  return (
    isRecord(value) &&
    hasExactFields(value, ["code"]) &&
    (value.code === "unavailable" ||
      value.code === "read_failed" ||
      value.code === "corrupt_reference")
  );
}

const RESOLUTION_FIELDS = ["schemaVersion", "citekey", "renderStyle", "displayMarker"];

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
