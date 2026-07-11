import type { CitationNodeError } from "../../citations/citationNode";
import type {
  CitationResolutionClientError,
  CitationStoreError,
} from "../../ipc/citationResolution";
import type {
  ConnectivityMode,
  ConnectivityModeClientError,
} from "../../ipc/connectivityMode";
import type {
  FormattingReviewClientError,
  FormattingReviewCommandErrorCode,
} from "../../ipc/formattingReview";
import type { RuntimeUnavailableReason } from "../runtime-status/runtimeStatusSession";

export type FormattingPresentationError =
  | FormattingReviewClientError
  | { type: "invalid-citation" };

export type FailureDisposition = "retryable" | "actionable" | "terminal";

export interface FailurePresentation {
  title: string;
  disposition: FailureDisposition;
  message: string;
  retryLabel?: string;
  actionLabel?: string;
}

export interface CitationFailurePresentation extends FailurePresentation {
  state: "invalid" | "unavailable" | "failed";
}

type RuntimeCommandFailureCode = Extract<
  RuntimeUnavailableReason,
  { type: "command" }
>["code"];
type ConnectivityCommandFailureCode = Extract<
  ConnectivityModeClientError,
  { type: "command" }
>["code"];

const RUNTIME_COMMAND_MESSAGES = {
  invalid_application_version: actionable(
    "Unsupported application version",
    "DRAFT received an unsupported application version.",
  ),
  event_delivery_failed: retryable(
    "Core status update failed",
    "DRAFT could not deliver the core status event.",
  ),
} satisfies Record<RuntimeCommandFailureCode, FailurePresentation>;

const CONNECTIVITY_COMMAND_ACTIONS = {
  connectivity_unavailable: {
    read: "DRAFT could not read connectivity mode.",
    change: "DRAFT could not change connectivity mode.",
  },
} satisfies Record<ConnectivityCommandFailureCode, { read: string; change: string }>;

const FORMATTING_COMMAND_MESSAGES = {
  too_many_headings: actionable(
    "Too many headings",
    "This document has too many headings for one formatting check. Split it and check again.",
  ),
  too_many_citations: actionable(
    "Too many citations",
    "This document has too many citations for one formatting check. Split it and check again.",
  ),
  invalid_heading_level: actionable(
    "Heading level cannot be checked",
    "DRAFT could not validate a heading level. Correct the heading and check again.",
  ),
  empty_heading_title: actionable(
    "Heading title is missing",
    "DRAFT could not validate an empty heading. Add a title or remove it, then check again.",
  ),
  heading_title_too_long: actionable(
    "Heading title is too long",
    "DRAFT could not check a heading because its title is too long. Shorten it and check again.",
  ),
  invalid_citekey: actionable(
    "Citation cannot be checked",
    "DRAFT could not validate a citation. Correct or remove it, then check again.",
  ),
} satisfies Record<FormattingReviewCommandErrorCode, FailurePresentation>;

const CITATION_NODE_MESSAGES = {
  unknown_citation_node_field: terminal(
    "Citation data is invalid",
    "Citation data is invalid. Keep this citation unchanged.",
  ),
  missing_citation_attrs: terminal(
    "Citation data is incomplete",
    "Citation data is incomplete. Keep this citation unchanged.",
  ),
  invalid_citation_attrs_object: terminal(
    "Citation data is invalid",
    "Citation data is invalid. Keep this citation unchanged.",
  ),
  unknown_citation_attr: terminal(
    "Citation data is invalid",
    "Citation data is invalid. Keep this citation unchanged.",
  ),
  missing_schema_version: terminal(
    "Citation version is missing",
    "Citation version is missing. Keep this citation unchanged.",
  ),
  invalid_schema_version: terminal(
    "Citation version is invalid",
    "Citation version is invalid. Keep this citation unchanged.",
  ),
  unsupported_schema_version: terminal(
    "Citation version is unsupported",
    "Citation version is unsupported. Keep this citation unchanged.",
  ),
  missing_citekey: terminal(
    "Citation key is missing",
    "Citation key is missing. Keep this citation unchanged.",
  ),
  invalid_citekey: terminal(
    "Citation key is invalid",
    "Citation key is invalid. Keep this citation unchanged.",
  ),
  missing_render_style: terminal(
    "Citation style is missing",
    "Citation style is missing. Keep this citation unchanged.",
  ),
  unsupported_render_style: terminal(
    "Citation style is unsupported",
    "Citation style is unsupported. Keep this citation unchanged.",
  ),
} satisfies Record<CitationNodeError["code"], FailurePresentation>;

const CITATION_STORE_MESSAGES = {
  unavailable: retryable(
    "Citation unavailable",
    "DRAFT could not read the reference for this citation. Restart DRAFT before relying on it.",
  ),
  read_failed: retryable(
    "Citation unavailable",
    "DRAFT could not read the reference for this citation. Restart DRAFT before relying on it.",
  ),
  corrupt_reference: terminal(
    "Citation unavailable",
    "DRAFT could not validate the reference for this citation. Keep it unchanged.",
  ),
} satisfies Record<CitationStoreError["code"], FailurePresentation>;

export function runtimeFailureMessage(reason: RuntimeUnavailableReason) {
  return runtimeFailurePresentation(reason).message;
}

export function runtimeFailurePresentation(
  reason: RuntimeUnavailableReason,
): FailurePresentation {
  switch (reason.type) {
    case "command":
      return runtimeCommandFailurePresentation(reason.code);
    case "invalid-payload":
    case "invalid-response":
      return retryable("Core status invalid", "Core status invalid");
    case "transport":
      return retryable("Core unavailable", "Core unavailable");
    default:
      return assertNever(reason);
  }
}

export function runtimeCommandFailureMessage(code: string) {
  return runtimeCommandFailurePresentation(code).message;
}

export function runtimeCommandFailurePresentation(code: string): FailurePresentation {
  return isRuntimeCommandFailureCode(code)
    ? RUNTIME_COMMAND_MESSAGES[code]
    : retryable("Core status unavailable", "DRAFT could not read the core status.");
}

export function connectivityFailurePresentation(
  error: ConnectivityModeClientError,
  retainedMode?: ConnectivityMode,
) {
  const action = connectivityFailureAction(error, retainedMode !== undefined);
  const message = retainedMode === undefined
    ? `${action} Try again.`
    : `${action} DRAFT remains ${retainedMode}.`;
  const retryLabel = retainedMode === undefined
    ? "Retry connectivity status"
    : retainedMode === "online"
      ? "Work offline"
      : "Go online";
  return retryable(
    retainedMode === undefined ? "Connectivity mode unavailable" : "Connectivity mode change failed",
    message,
    retryLabel,
  );
}

export function formattingFailureMessage(error: FormattingPresentationError) {
  return formattingFailurePresentation(error).message;
}

export function formattingFailurePresentation(
  error: FormattingPresentationError,
): FailurePresentation {
  switch (error.type) {
    case "command":
      return FORMATTING_COMMAND_MESSAGES[error.code];
    case "invalid-citation":
      return actionable(
        "Citation cannot be checked",
        "DRAFT found a citation that cannot be checked. Correct or remove it, then check again.",
      );
    case "invalid-response":
      return retryable(
        "Formatting response is invalid",
        "DRAFT received an invalid formatting response. Check again.",
        "Check formatting again",
      );
    case "transport":
      return retryable(
        "Formatting review unavailable",
        "Formatting review could not reach the DRAFT core. Restart DRAFT, then check again.",
        "Check formatting again",
      );
    default:
      return assertNever(error);
  }
}

export function citationNodeFailureMessage(error: CitationNodeError) {
  return CITATION_NODE_MESSAGES[error.code].message;
}

export function citationFailurePresentation(
  error: CitationResolutionClientError,
): CitationFailurePresentation {
  switch (error.type) {
    case "invalid-response":
      return failedCitation(
        retryable(
          "Citation response is invalid",
          "DRAFT received an invalid citation response. Keep this citation unchanged.",
        ),
      );
    case "transport":
      return failedCitation(
        retryable(
          "Citation unavailable",
          "DRAFT could not check this citation. Restart DRAFT and try again.",
        ),
      );
    case "command":
      return citationCommandFailurePresentation(error.error);
    default:
      return assertNever(error);
  }
}

function connectivityFailureAction(error: ConnectivityModeClientError, isChange: boolean) {
  switch (error.type) {
    case "command": {
      const actions = CONNECTIVITY_COMMAND_ACTIONS[error.code];
      return isChange ? actions.change : actions.read;
    }
    case "invalid-response":
      return "DRAFT received an invalid connectivity response.";
    case "transport":
      return isChange
        ? "DRAFT could not reach the core to change connectivity mode."
        : "DRAFT could not reach the core to read connectivity mode.";
    default:
      return assertNever(error);
  }
}

function isRuntimeCommandFailureCode(code: string): code is RuntimeCommandFailureCode {
  return Object.prototype.hasOwnProperty.call(RUNTIME_COMMAND_MESSAGES, code);
}

function citationCommandFailurePresentation(
  error: Extract<CitationResolutionClientError, { type: "command" }>["error"],
): CitationFailurePresentation {
  switch (error.code) {
    case "reference_not_found":
      return {
        state: "unavailable",
        ...terminal("Citation unavailable", "DRAFT could not resolve this citation. Keep it unchanged."),
      };
    case "invalid_citation":
      return { state: "invalid", ...CITATION_NODE_MESSAGES[error.cause.code] };
    case "reference_store":
      return failedCitation(CITATION_STORE_MESSAGES[error.cause.code]);
    default:
      return assertNever(error);
  }
}

function retryable(title: string, message: string, retryLabel?: string): FailurePresentation {
  return { disposition: "retryable", title, message, ...(retryLabel ? { retryLabel } : {}) };
}

function actionable(title: string, message: string, actionLabel?: string): FailurePresentation {
  return { disposition: "actionable", title, message, ...(actionLabel ? { actionLabel } : {}) };
}

function terminal(title: string, message: string): FailurePresentation {
  return { disposition: "terminal", title, message };
}

function failedCitation(presentation: FailurePresentation): CitationFailurePresentation {
  return { state: "failed", ...presentation };
}

function assertNever(value: never): never {
  void value;
  throw new Error("Unmapped visible error");
}
