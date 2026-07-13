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
    { ...envelope(), document: documentWithFont("fontFamily", { family: "injected" }) },
    { ...envelope(), document: documentWithFont("fontSize", { points: 7 }) },
    { ...envelope(), document: documentWithMarks([null]) },
    { ...envelope(), document: documentWithMarks(["fontFamily"]) },
    { ...envelope(), document: documentWithMarks([1]) },
    { ...envelope(), document: documentWithMarks([[]]) },
    { ...envelope(), document: documentWithMarks([{}]) },
    { ...envelope(), document: documentWithMarks([{ type: 1 }]) },
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

  it("recognizes structured text-format failures", () => {
    expect(isDocumentEnvelopeError({
      code: "invalid_text_format",
      path: "document.content[0].content[0].marks[0]",
      cause: { code: "invalid_font_size" },
    })).toBe(true);
    expect(isDocumentEnvelopeError({
      code: "invalid_text_format",
      path: "document.content[0].content[0].marks[0]",
      cause: { code: "unknown" },
    })).toBe(false);
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

function documentWithFont(type: string, attrs: Record<string, unknown>) {
  return documentWithMarks([{ type, attrs }]);
}

function documentWithMarks(marks: unknown[]) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      content: [{ type: "text", text: "Text", marks }],
    }],
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
