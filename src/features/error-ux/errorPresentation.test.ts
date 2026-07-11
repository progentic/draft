import { describe, expect, it } from "vitest";

import type { CitationNodeError } from "../../citations/citationNode";
import type { CitationResolutionClientError } from "../../ipc/citationResolution";
import type { ConnectivityModeClientError } from "../../ipc/connectivityMode";
import type { FormattingReviewCommandErrorCode } from "../../ipc/formattingReview";
import {
  citationFailurePresentation,
  citationNodeFailureMessage,
  connectivityFailurePresentation,
  formattingFailureMessage,
  formattingFailurePresentation,
  runtimeCommandFailureMessage,
  runtimeCommandFailurePresentation,
  runtimeFailureMessage,
  runtimeFailurePresentation,
} from "./errorPresentation";

const RUNTIME_COMMAND_CASES = {
  invalid_application_version: "DRAFT received an unsupported application version.",
  event_delivery_failed: "DRAFT could not deliver the core status event.",
} as const;

const FORMATTING_COMMAND_CODES = [
  "too_many_headings",
  "too_many_citations",
  "invalid_heading_level",
  "empty_heading_title",
  "heading_title_too_long",
  "invalid_citekey",
] satisfies FormattingReviewCommandErrorCode[];

const CITATION_NODE_ERRORS = [
  { code: "unknown_citation_node_field", field: "extra" },
  { code: "missing_citation_attrs" },
  { code: "invalid_citation_attrs_object" },
  { code: "unknown_citation_attr", field: "extra" },
  { code: "missing_schema_version" },
  { code: "invalid_schema_version" },
  { code: "unsupported_schema_version", found: 2 },
  { code: "missing_citekey" },
  { code: "invalid_citekey" },
  { code: "missing_render_style" },
  { code: "unsupported_render_style" },
] satisfies CitationNodeError[];

const CITATION_NODE_DISPOSITIONS = {
  unknown_citation_node_field: "terminal",
  missing_citation_attrs: "terminal",
  invalid_citation_attrs_object: "terminal",
  unknown_citation_attr: "terminal",
  missing_schema_version: "terminal",
  invalid_schema_version: "terminal",
  unsupported_schema_version: "terminal",
  missing_citekey: "terminal",
  invalid_citekey: "terminal",
  missing_render_style: "terminal",
  unsupported_render_style: "terminal",
} satisfies Record<CitationNodeError["code"], "terminal">;

describe("error presentation policy", () => {
  it.each(Object.entries(RUNTIME_COMMAND_CASES))(
    "maps runtime command code %s",
    (code, message) => {
      expect(runtimeCommandFailureMessage(code)).toBe(message);
      expect(runtimeCommandFailurePresentation(code).disposition).toMatch(
        /actionable|retryable/,
      );
      expect(runtimeCommandFailurePresentation(code).title).not.toBe("");
    },
  );

  it("keeps runtime invalid, transport, and unknown fallbacks distinct", () => {
    expect(runtimeFailureMessage({ type: "invalid-response" })).toBe("Core status invalid");
    expect(runtimeFailureMessage({ type: "invalid-payload" })).toBe("Core status invalid");
    expect(runtimeFailureMessage({ type: "transport" })).toBe("Core unavailable");
    expect(runtimeCommandFailureMessage("unknown_command_failure")).toBe(
      "DRAFT could not read the core status.",
    );
    expect(runtimeFailurePresentation({ type: "transport" }).disposition).toBe("retryable");
  });

  it.each([
    [{ type: "command", code: "connectivity_unavailable" }, "could not read"],
    [{ type: "invalid-response" }, "invalid connectivity response"],
    [{ type: "transport" }, "could not reach the core to read"],
  ] satisfies Array<[ConnectivityModeClientError, string]>) (
    "maps connectivity read failure %j",
    (error, phrase) => {
      expect(connectivityFailurePresentation(error).message).toContain(phrase);
      expect(connectivityFailurePresentation(error).message).toContain("Try again.");
      expect(connectivityFailurePresentation(error).disposition).toBe("retryable");
      expect(connectivityFailurePresentation(error).retryLabel).toBe(
        "Retry connectivity status",
      );
    },
  );

  it.each([
    [{ type: "command", code: "connectivity_unavailable" }, "could not change"],
    [{ type: "invalid-response" }, "invalid connectivity response"],
    [{ type: "transport" }, "could not reach the core to change"],
  ] satisfies Array<[ConnectivityModeClientError, string]>) (
    "maps connectivity change failure %j",
    (error, phrase) => {
      expect(connectivityFailurePresentation(error, "online").message).toContain(phrase);
      expect(connectivityFailurePresentation(error, "online").message).toContain(
        "DRAFT remains online.",
      );
      expect(connectivityFailurePresentation(error, "online").disposition).toBe("retryable");
      expect(connectivityFailurePresentation(error, "online").retryLabel).toBe("Work offline");
    },
  );

  it.each(FORMATTING_COMMAND_CODES)("maps formatting command code %s", (code) => {
    const message = formattingFailureMessage({ type: "command", code });
    expect(message).toContain("check");
    expect(formattingFailurePresentation({ type: "command", code }).disposition).toBe(
      "actionable",
    );
  });

  it("keeps formatting collection, response, and transport failures distinct", () => {
    expect(formattingFailureMessage({ type: "invalid-citation" })).toContain("citation");
    expect(formattingFailureMessage({ type: "invalid-response" })).toContain("invalid");
    expect(formattingFailureMessage({ type: "transport" })).toContain("could not reach");
    expect(formattingFailurePresentation({ type: "invalid-response" }).disposition).toBe(
      "retryable",
    );
    expect(formattingFailurePresentation({ type: "invalid-response" }).retryLabel).toBe(
      "Check formatting again",
    );
    expect(formattingFailurePresentation({ type: "command", code: "invalid_citekey" }).actionLabel)
      .toBeUndefined();
  });

  it.each(CITATION_NODE_ERRORS)("maps citation node error $code", (error) => {
    expect(citationNodeFailureMessage(error)).toMatch(/Citation|citation/);
  });

  it.each(citationClientErrors())(
    "maps citation client failure $name",
    ({ disposition, error, state, text }) => {
      const presentation = citationFailurePresentation(error);
      expect(presentation.state).toBe(state);
      expect(presentation.message).toContain(text);
      expect(presentation.disposition).toBe(disposition);
      expect(presentation.title).not.toBe("");
      expect(presentation.actionLabel).toBeUndefined();
    },
  );
});

function citationClientErrors(): Array<{
  name: string;
  error: CitationResolutionClientError;
  state: "invalid" | "unavailable" | "failed";
  disposition: "retryable" | "actionable" | "terminal";
  text: string;
}> {
  return [
    {
      name: "missing reference",
      error: { type: "command", error: { code: "reference_not_found" } },
      state: "unavailable",
      disposition: "terminal",
      text: "could not resolve",
    },
    ...CITATION_NODE_ERRORS.map((cause) => ({
      name: cause.code,
      error: { type: "command" as const, error: { code: "invalid_citation" as const, cause } },
      state: "invalid" as const,
      disposition: CITATION_NODE_DISPOSITIONS[cause.code],
      text: "Citation",
    })),
    ...(["unavailable", "read_failed", "corrupt_reference"] as const).map((code) => ({
      name: code,
      error: {
        type: "command" as const,
        error: { code: "reference_store" as const, cause: { code } },
      },
      state: "failed" as const,
      disposition: code === "corrupt_reference" ? "terminal" as const : "retryable" as const,
      text: "reference for this citation",
    })),
    {
      name: "invalid response",
      error: { type: "invalid-response" },
      state: "failed",
      disposition: "retryable",
      text: "invalid citation response",
    },
    {
      name: "transport",
      error: { type: "transport" },
      state: "failed",
      disposition: "retryable",
      text: "could not check",
    },
  ];
}
