import { isRecord } from "./documentEnvelope";

export interface ReferenceSummary {
  citekey: string;
  title: string;
}

export interface AddReferenceInput {
  citekey: string;
  title: string;
  author: string;
  year: number;
}

export type ReferenceLibraryErrorCode =
  | "duplicate_citekey"
  | "invalid_reference"
  | "read_failed"
  | "store_unavailable"
  | "write_failed";

export type ReferenceLibraryClientError =
  | { type: "command"; code: ReferenceLibraryErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type ReferenceLibraryResult<T> =
  | { status: "ready"; value: T }
  | { status: "error"; error: ReferenceLibraryClientError };

export function libraryError(error: unknown): ReferenceLibraryClientError {
  if (isReferenceLibraryError(error)) {
    return { type: "command", code: error.code };
  }
  return { type: "transport" };
}

export function isReferenceList(value: unknown): value is ReferenceSummary[] {
  return Array.isArray(value) && value.every(isReferenceSummary);
}

export function isReferenceSummary(value: unknown): value is ReferenceSummary {
  return (
    isRecord(value) &&
    Object.keys(value).length === 2 &&
    isNonBlank(value.citekey) &&
    isNonBlank(value.title)
  );
}

function isReferenceLibraryError(
  value: unknown,
): value is { code: ReferenceLibraryErrorCode } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    (value.code === "duplicate_citekey" ||
      value.code === "invalid_reference" ||
      value.code === "read_failed" ||
      value.code === "store_unavailable" ||
      value.code === "write_failed")
  );
}

function isNonBlank(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}
