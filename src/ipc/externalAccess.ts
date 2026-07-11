import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export type ExternalAccessDestination =
  | "publisher"
  | "institutional"
  | "doi"
  | "google_scholar";

export type OpenExternalAccessRequest =
  | { destination: "publisher"; url: string }
  | { destination: "institutional"; url: string }
  | { destination: "doi"; doi: string }
  | { destination: "google_scholar"; query: string };

export interface OpenExternalAccessResponse {
  status: "opened";
  destination: ExternalAccessDestination;
}

export type ExternalAccessCommandErrorCode =
  | "invalid_url"
  | "invalid_doi"
  | "invalid_search_query"
  | "offline"
  | "connectivity_unavailable"
  | "browser_unavailable";

export type ExternalAccessClientError =
  | { type: "command"; code: ExternalAccessCommandErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type ExternalAccessResult =
  | { status: "opened"; destination: ExternalAccessDestination }
  | { status: "error"; error: ExternalAccessClientError };

const COMMAND_NAME = "open_external_access";

/** Requests a validated Rust-owned handoff to the default system browser. */
export async function openExternalAccess(
  request: OpenExternalAccessRequest,
): Promise<ExternalAccessResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request });
    return resultFromResponse(response, request.destination);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

function resultFromResponse(
  response: unknown,
  requestedDestination: ExternalAccessDestination,
): ExternalAccessResult {
  if (
    !isOpenExternalAccessResponse(response) ||
    response.destination !== requestedDestination
  ) {
    return { status: "error", error: { type: "invalid-response" } };
  }
  return { status: "opened", destination: response.destination };
}

function clientErrorFrom(error: unknown): ExternalAccessClientError {
  if (isRecord(error) && hasExactFields(error, ["code"]) && isCommandErrorCode(error.code)) {
    return { type: "command", code: error.code };
  }
  return { type: "transport" };
}

function isOpenExternalAccessResponse(value: unknown): value is OpenExternalAccessResponse {
  return (
    isRecord(value) &&
    hasExactFields(value, ["status", "destination"]) &&
    value.status === "opened" &&
    isExternalAccessDestination(value.destination)
  );
}

function isExternalAccessDestination(value: unknown): value is ExternalAccessDestination {
  return (
    value === "publisher" ||
    value === "institutional" ||
    value === "doi" ||
    value === "google_scholar"
  );
}

function isCommandErrorCode(value: unknown): value is ExternalAccessCommandErrorCode {
  return (
    value === "invalid_url" ||
    value === "invalid_doi" ||
    value === "invalid_search_query" ||
    value === "offline" ||
    value === "connectivity_unavailable" ||
    value === "browser_unavailable"
  );
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
