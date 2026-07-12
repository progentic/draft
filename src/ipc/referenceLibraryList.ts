import { invokeCommand } from "./client";
import {
  isReferenceList,
  libraryError,
  type ReferenceLibraryResult,
  type ReferenceSummary,
} from "./referenceLibrary";

const COMMAND_NAME = "list_references";

export async function listReferences(): Promise<ReferenceLibraryResult<ReferenceSummary[]>> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: {} });
    return isReferenceList(response)
      ? { status: "ready", value: response }
      : { status: "error", error: { type: "invalid-response" } };
  } catch (error: unknown) {
    return { status: "error", error: libraryError(error) };
  }
}
