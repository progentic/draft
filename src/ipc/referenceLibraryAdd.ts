import { invokeCommand } from "./client";
import {
  isReferenceSummary,
  libraryError,
  type AddReferenceInput,
  type ReferenceLibraryResult,
  type ReferenceSummary,
} from "./referenceLibrary";

const COMMAND_NAME = "add_reference";

export async function addReference(
  input: AddReferenceInput,
): Promise<ReferenceLibraryResult<ReferenceSummary>> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: input });
    return isReferenceSummary(response)
      ? { status: "ready", value: response }
      : { status: "error", error: { type: "invalid-response" } };
  } catch (error: unknown) {
    return { status: "error", error: libraryError(error) };
  }
}
