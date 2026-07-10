import { describe, expect, it } from "vitest";

import { isDocumentEnvelopeError, isDocumentEnvelopeSnapshot } from "./documentEnvelope";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("document envelope mirror", () => {
  it("accepts the implemented version 1 shape", () => {
    expect(isDocumentEnvelopeSnapshot(envelope())).toBe(true);
  });

  it.each([
    { ...envelope(), schema_version: 2 },
    { ...envelope(), document_id: "not-a-uuid" },
    { ...envelope(), title: " " },
    { ...envelope(), document: { type: "paragraph", content: [] } },
    { ...envelope(), document: documentWithCitation({ schema_version: 2 }) },
    { ...envelope(), references: [] },
  ])("rejects an invalid response mirror", (value) => {
    expect(isDocumentEnvelopeSnapshot(value)).toBe(false);
  });

  it("recognizes structured unsupported-version failures", () => {
    expect(isDocumentEnvelopeError({ code: "unsupported_schema_version", found: 2 })).toBe(true);
    expect(isDocumentEnvelopeError({ code: "unsupported_schema_version" })).toBe(false);
  });

  it("recognizes structured citation failures", () => {
    expect(
      isDocumentEnvelopeError({
        code: "invalid_citation_node",
        path: "document.content[0]",
        cause: { code: "missing_citekey" },
      }),
    ).toBe(true);
    expect(
      isDocumentEnvelopeError({
        code: "invalid_citation_node",
        path: "document.content[0]",
        cause: { code: "unknown" },
      }),
    ).toBe(false);
  });
});

function envelope() {
  return {
    schema_version: 1,
    document_id: DOCUMENT_ID,
    title: "Document",
    document: { type: "doc", content: [] },
  };
}

function documentWithCitation(attrs: Record<string, unknown>) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      content: [{
        type: "citation",
        attrs: { citekey: "smith2025", render_style: "apa7", ...attrs },
      }],
    }],
  };
}
