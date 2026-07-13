import { describe, expect, it } from "vitest";

import {
  hasValidParagraphStyles,
  isParagraphStyleError,
  omitUnsetParagraphStyles,
  parseParagraphStyle,
} from "./paragraphFormatting";

describe("paragraph-formatting mirror", () => {
  it("accepts the exact supported style on paragraphs and headings", () => {
    for (const type of ["paragraph", "heading"]) {
      expect(hasValidParagraphStyles(documentWith(type, style()))).toBe(true);
    }
  });

  it.each([
    null,
    {},
    { ...style(), css: "evil" },
    { ...style(), schemaVersion: 2 },
    { ...style(), alignment: "distributed" },
    { ...style(), lineSpacingHundredths: 101 },
    { ...style(), lineSpacingHundredths: 100.5 },
    { ...style(), spaceBeforeTwips: -1 },
    { ...style(), rightIndentTwips: 2_881 },
    { ...style(), specialIndent: { kind: "none", twips: 1 } },
    { ...style(), specialIndent: { kind: "hanging", twips: 1_441 } },
    { ...style(), specialIndent: { kind: "both", twips: 0 } },
  ])("rejects malformed or unsupported style %#", (paragraphStyle) => {
    expect(hasValidParagraphStyles(documentWith("paragraph", paragraphStyle))).toBe(false);
  });

  it("rejects paragraph style on unsupported blocks", () => {
    expect(hasValidParagraphStyles(documentWith("listItem", style()))).toBe(false);
  });

  it("rejects every missing field and malformed nested shape", () => {
    for (const field of Object.keys(style())) {
      const value = { ...style() } as Record<string, unknown>;
      delete value[field];
      expect(hasValidParagraphStyles(documentWith("paragraph", value))).toBe(false);
    }

    for (const specialIndent of [
      { kind: "none" },
      { twips: 0 },
      { kind: "none", twips: 0, extra: true },
      [],
      null,
    ]) {
      expect(hasValidParagraphStyles(documentWith("paragraph", {
        ...style(),
        specialIndent,
      }))).toBe(false);
    }
  });

  it("recognizes every structured Rust error family", () => {
    expect(isParagraphStyleError({ code: "unknown_style_field", field: "css" })).toBe(true);
    expect(isParagraphStyleError({ code: "unsupported_style_schema_version", found: 2 })).toBe(true);
    expect(isParagraphStyleError({ code: "invalid_line_spacing" })).toBe(true);
    expect(isParagraphStyleError({ code: "future_error" })).toBe(false);
  });

  it("parses canonical values and removes only unset editor attributes", () => {
    expect(parseParagraphStyle(style())).toEqual(style());
    expect(omitUnsetParagraphStyles({
      type: "paragraph",
      attrs: { paragraphStyle: null, preserved: true },
      content: [{ type: "text", text: "Text" }],
    })).toEqual({
      type: "paragraph",
      attrs: { preserved: true },
      content: [{ type: "text", text: "Text" }],
    });
  });
});

function style() {
  return {
    schemaVersion: 1,
    alignment: "left",
    lineSpacingHundredths: 100,
    spaceBeforeTwips: 0,
    spaceAfterTwips: 0,
    leftIndentTwips: 0,
    rightIndentTwips: 0,
    specialIndent: { kind: "none", twips: 0 },
  };
}

function documentWith(type: string, paragraphStyle: unknown) {
  return {
    type: "doc",
    content: [{ type, attrs: { paragraphStyle } }],
  };
}
