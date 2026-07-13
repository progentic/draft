import { invokeCommand } from "./client";
import {
  isDocumentEnvelopeSnapshot,
  isRecord,
  type DocumentEnvelopeSnapshot,
} from "./documentEnvelope";

export type CreateDocumentClientError =
  | { type: "command"; code: "template_invalid" }
  | { type: "invalid-response" }
  | { type: "transport" };

export type CreateDocumentResult =
  | { status: "created"; envelope: DocumentEnvelopeSnapshot }
  | { status: "error"; error: CreateDocumentClientError };

const COMMAND_NAME = "create_document";

export async function createUnsavedDocument(): Promise<CreateDocumentResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: {} });
    return createdResult(response);
  } catch (error: unknown) {
    return { status: "error", error: createError(error) };
  }
}

function createdResult(response: unknown): CreateDocumentResult {
  if (
    isRecord(response) &&
    Object.keys(response).length === 2 &&
    response.status === "created" &&
    isDocumentEnvelopeSnapshot(response.envelope)
  ) {
    return { status: "created", envelope: response.envelope };
  }
  return { status: "error", error: { type: "invalid-response" } };
}

function createError(error: unknown): CreateDocumentClientError {
  if (
    isRecord(error) &&
    Object.keys(error).length === 1 &&
    error.code === "template_invalid"
  ) {
    return { type: "command", code: error.code };
  }
  return { type: "transport" };
}
