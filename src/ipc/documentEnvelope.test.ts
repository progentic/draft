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
    { ...envelope(), references: [] },
  ])("rejects an invalid response mirror", (value) => {
    expect(isDocumentEnvelopeSnapshot(value)).toBe(false);
  });

  it("recognizes structured unsupported-version failures", () => {
    expect(isDocumentEnvelopeError({ code: "unsupported_schema_version", found: 2 })).toBe(true);
    expect(isDocumentEnvelopeError({ code: "unsupported_schema_version" })).toBe(false);
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
