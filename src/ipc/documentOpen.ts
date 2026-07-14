import { invokeCommand } from "./client";
import {
  isDocumentEnvelopeSnapshot,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";
import { isOpenDocumentCommandError, type OpenDocumentCommandError } from "./documentErrors";
import {
  isExternalDocumentSummary,
  type ExternalDocumentSummary,
} from "./externalDocument";

export type OpenDocumentRequest = Record<string, never>;

export type OpenDocumentClientError =
  | { type: "command"; error: OpenDocumentCommandError }
  | { type: "invalid-response" }
  | { type: "transport" };

export type OpenDocumentResult =
  | { status: "opened_draft"; envelope: DocumentEnvelopeSnapshot }
  | {
      status: "imported_text";
      envelope: DocumentEnvelopeSnapshot;
      format: "markdown" | "plain_text";
    }
  | {
      status: "imported_external";
      envelope: DocumentEnvelopeSnapshot;
      external: ExternalDocumentSummary;
    }
  | { status: "cancelled" }
  | { status: "error"; error: OpenDocumentClientError };

type OpenDocumentArguments = {
  request: OpenDocumentRequest;
};

const COMMAND_NAME = "open_document";
const COMMAND_ARGUMENTS: OpenDocumentArguments = { request: {} };

/** Opens a Rust-selected document without exposing filesystem APIs to the UI. */
export async function openDocument(): Promise<OpenDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, COMMAND_ARGUMENTS);
    return openDocumentResultFromResponse(response);
  } catch (error: unknown) {
    return { status: "error", error: clientErrorFrom(error) };
  }
}

export function openDocumentResultFromResponse(response: unknown): OpenDocumentResult {
  if (isCancelledResponse(response)) {
    return { status: "cancelled" };
  }

  if (isDocumentResponse(response, "opened_draft")) {
    return { status: "opened_draft", envelope: response.envelope };
  }

  if (isImportedTextResponse(response)) {
    return {
      status: "imported_text",
      envelope: response.envelope,
      format: response.format,
    };
  }

  if (isImportedExternalResponse(response)) {
    return {
      status: "imported_external",
      envelope: response.envelope,
      external: response.external,
    };
  }

  return { status: "error", error: { type: "invalid-response" } };
}

function isImportedExternalResponse(value: unknown): value is {
  status: "imported_external";
  envelope: DocumentEnvelopeSnapshot;
  external: ExternalDocumentSummary;
} {
  return (
    isRecord(value) &&
    Object.keys(value).length === 3 &&
    value.status === "imported_external" &&
    isDocumentEnvelopeSnapshot(value.envelope) &&
    isExternalDocumentSummary(value.external)
  );
}

function isImportedTextResponse(value: unknown): value is {
  status: "imported_text";
  envelope: DocumentEnvelopeSnapshot;
  format: "markdown" | "plain_text";
} {
  return (
    isRecord(value) &&
    Object.keys(value).length === 3 &&
    value.status === "imported_text" &&
    isDocumentEnvelopeSnapshot(value.envelope) &&
    (value.format === "markdown" || value.format === "plain_text")
  );
}

function clientErrorFrom(error: unknown): OpenDocumentClientError {
  if (isOpenDocumentCommandError(error)) {
    return { type: "command", error };
  }

  return { type: "transport" };
}

function isCancelledResponse(value: unknown): value is { status: "cancelled" } {
  return isRecord(value) && Object.keys(value).length === 1 && value.status === "cancelled";
}

function isDocumentResponse(
  value: unknown,
  status: "opened_draft",
): value is {
  status: "opened_draft";
  envelope: DocumentEnvelopeSnapshot;
} {
  return (
    isRecord(value) &&
    Object.keys(value).length === 2 &&
    value.status === status &&
    isDocumentEnvelopeSnapshot(value.envelope)
  );
}
