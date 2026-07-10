import { describe, expect, it } from "vitest";

import {
  hasValidCitationNodes,
  isCitationNodeError,
  validateCitationNodeAttributes,
} from "./citationNode";

describe("citation node mirror", () => {
  it("accepts and normalizes the version 1 attrs", () => {
    expect(validateCitationNodeAttributes(validAttrs())).toEqual({
      valid: true,
      attrs: validAttrs(),
    });
  });

  it.each([
    [null, "invalid_citation_attrs_object"],
    [{ citekey: "smith2025", render_style: "apa7" }, "missing_schema_version"],
    [{ ...validAttrs(), schema_version: "1" }, "invalid_schema_version"],
    [{ ...validAttrs(), schema_version: 2 }, "unsupported_schema_version"],
    [{ ...validAttrs(), citekey: "smith.2025" }, "invalid_citekey"],
    [{ ...validAttrs(), render_style: "apa6" }, "unsupported_render_style"],
    [{ ...validAttrs(), reference: {} }, "unknown_citation_attr"],
  ])("rejects malformed attrs with %s", (value, code) => {
    const result = validateCitationNodeAttributes(value);
    expect(result.valid).toBe(false);
    expect(result.valid ? undefined : result.error.code).toBe(code);
  });

  it("finds invalid citations nested in Tiptap content", () => {
    expect(hasValidCitationNodes(documentWith(validAttrs()))).toBe(true);
    expect(hasValidCitationNodes(documentWith({ ...validAttrs(), schema_version: 2 }))).toBe(false);
    expect(hasValidCitationNodes(documentWith(validAttrs(), { content: [] }))).toBe(false);
  });

  it("leaves unrelated Tiptap nodes opaque", () => {
    expect(
      hasValidCitationNodes({
        type: "doc",
        content: [{ type: "future_node", attrs: { arbitrary: true } }],
      }),
    ).toBe(true);
  });

  it("recognizes every structured error family", () => {
    expect(isCitationNodeError({ code: "missing_citekey" })).toBe(true);
    expect(
      isCitationNodeError({ code: "unknown_citation_attr", field: "reference" }),
    ).toBe(true);
    expect(isCitationNodeError({ code: "unsupported_schema_version", found: 2 })).toBe(true);
    expect(isCitationNodeError({ code: "unsupported_schema_version", found: -1 })).toBe(false);
    expect(isCitationNodeError({ code: "missing_citekey", extra: true })).toBe(false);
  });
});

function validAttrs() {
  return {
    schema_version: 1 as const,
    citekey: "smith2025",
    render_style: "apa7" as const,
  };
}

function documentWith(attrs: unknown, extra: Record<string, unknown> = {}) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      content: [{ type: "citation", attrs, ...extra }],
    }],
  };
}
