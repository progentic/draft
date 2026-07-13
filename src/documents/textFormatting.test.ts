import { describe, expect, it } from "vitest";

import {
  FONT_FAMILIES,
  fontFamilyCss,
  hasValidTextFormatting,
  isFontFamilyId,
  isFontSizePoints,
  isTextFormatError,
} from "./textFormatting";

describe("text formatting contract", () => {
  it("keeps the complete canonical font allowlist stable", () => {
    expect(FONT_FAMILIES.map(({ id, label }) => [id, label])).toEqual([
      ["arial", "Arial"],
      ["avenir_next", "Avenir Next"],
      ["baskerville", "Baskerville"],
      ["courier_new", "Courier New"],
      ["georgia", "Georgia"],
      ["helvetica", "Helvetica"],
      ["menlo", "Menlo"],
      ["palatino", "Palatino"],
      ["times_new_roman", "Times New Roman"],
      ["trebuchet_ms", "Trebuchet MS"],
      ["verdana", "Verdana"],
    ]);
    expect(FONT_FAMILIES.map(({ id }) => fontFamilyCss(id))).toEqual(
      FONT_FAMILIES.map(({ label }) => `"${label}"`),
    );
  });

  it("accepts only canonical families and bounded integer point sizes", () => {
    expect(isFontFamilyId("georgia")).toBe(true);
    expect(isFontFamilyId("url(evil)")).toBe(false);
    expect(isFontSizePoints(8)).toBe(true);
    expect(isFontSizePoints(72)).toBe(true);
    expect(isFontSizePoints(0)).toBe(false);
    expect(isFontSizePoints(8.5)).toBe(false);
  });

  it("validates nested font marks without rejecting unrelated marks", () => {
    expect(hasValidTextFormatting(documentWithMarks([
      { type: "bold" },
      { type: "fontFamily", attrs: { family: "arial" } },
      { type: "fontSize", attrs: { points: 15 } },
    ]))).toBe(true);
    expect(hasValidTextFormatting(documentWithMarks([
      { type: "fontFamily", attrs: { family: "Comic Sans MS" } },
    ]))).toBe(false);
    expect(hasValidTextFormatting(documentWithMarks([
      { type: "fontSize", attrs: { points: "12" } },
    ]))).toBe(false);
  });

  it.each([null, "bold", 1, [], {}, { attrs: {} }, { type: 1 }])(
    "rejects malformed mark shape %j",
    (mark) => {
      expect(hasValidTextFormatting(documentWithMarks([mark]))).toBe(false);
    },
  );

  it.each([
    { type: "fontFamily", attrs: null },
    { type: "fontFamily", attrs: { family: "url(evil)" } },
    { type: "fontFamily", attrs: { family: "arial", css: "serif" } },
    { type: "fontFamily", attrs: { family: "arial" }, style: "serif" },
    { type: "fontSize", attrs: [] },
    { type: "fontSize", attrs: { points: 0 } },
    { type: "fontSize", attrs: { points: -1 } },
    { type: "fontSize", attrs: { points: 8.5 } },
    { type: "fontSize", attrs: { points: 73 } },
    { type: "fontSize", attrs: { points: 12, unit: "pt" } },
    { type: "fontSize", attrs: { points: 12 }, style: "12px" },
  ])("rejects malformed font mark %j", (mark) => {
    expect(hasValidTextFormatting(documentWithMarks([mark]))).toBe(false);
  });

  it("recognizes only bounded typed Rust formatting failures", () => {
    expect(isTextFormatError({ code: "invalid_font_size" })).toBe(true);
    expect(isTextFormatError({ code: "unknown_font_attr", field: "css" })).toBe(true);
    expect(isTextFormatError({ code: "unknown_font_attr", field: "css", path: "/tmp" })).toBe(false);
  });
});

function documentWithMarks(marks: unknown[]) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      content: [{ type: "text", text: "Text", marks }],
    }],
  };
}
