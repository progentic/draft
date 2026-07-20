import { describe, expect, it } from "vitest";

import {
  isExternalDocumentSummary,
  isExternalFidelity,
  isExternalNormalizationFeatureList,
} from "./externalDocument";

describe("external document DTO validation", () => {
  it.each([
    { classification: "exact" },
    {
      classification: "canonically_normalized",
      features: ["alternate_heading_style_name"],
    },
    {
      classification: "unsupported_preservable",
      features: ["paragraph_border", "paragraph_tab"],
    },
    { classification: "lossy", features: ["unsupported_document_structure"] },
    { classification: "malformed_external_input" },
    {
      classification: "unsupported_external_feature",
      features: ["exact_line_spacing"],
    },
    { classification: "unsafe", reason: "archive_path" },
  ])("accepts the closed $classification fidelity variant", (fidelity) => {
    expect(isExternalFidelity(fidelity)).toBe(true);
  });

  it.each([
    { classification: "future" },
    { classification: "exact", features: [] },
    { classification: "unsafe", reason: "private_path" },
    { classification: "lossy", features: ["unknown_feature"] },
    {
      classification: "unsupported_preservable",
      features: ["paragraph_tab", "paragraph_border"],
    },
    {
      classification: "unsupported_preservable",
      features: ["paragraph_border", "paragraph_border"],
    },
  ])("rejects malformed or unstable fidelity data", (fidelity) => {
    expect(isExternalFidelity(fidelity)).toBe(false);
  });

  it("accepts only path-free successful import summaries", () => {
    const summary = externalSummary();

    expect(isExternalDocumentSummary(summary)).toBe(true);
    expect(isExternalDocumentSummary({ ...summary, path: "/private/paper.docx" })).toBe(false);
    expect(isExternalDocumentSummary({ ...summary, rawXml: "<w:p/>" })).toBe(false);
    expect(isExternalDocumentSummary({ ...summary, displayName: "folder/paper.docx" })).toBe(
      false,
    );
    expect(
      isExternalDocumentSummary({
        ...summary,
        fidelity: { classification: "lossy", features: [] },
      }),
    ).toBe(false);
    expect(
      isExternalDocumentSummary({
        ...summary,
        fidelity: { classification: "lossy", features: ["footnote", "table_structure"] },
        sameFormatSave: "denied_unsupported_source_behavior",
      }),
    ).toBe(true);
  });

  it("accepts only stable supported normalization lists", () => {
    expect(isExternalNormalizationFeatureList(["pagination_control"])).toBe(true);
    expect(isExternalNormalizationFeatureList([
      "alternate_heading_style_name",
      "pagination_control",
    ])).toBe(true);
    expect(isExternalNormalizationFeatureList([])).toBe(false);
    expect(isExternalNormalizationFeatureList(["pagination_control", "pagination_control"]))
      .toBe(false);
    expect(isExternalNormalizationFeatureList([
      "pagination_control",
      "alternate_heading_style_name",
    ])).toBe(false);
  });
});

function externalSummary() {
  return {
    format: "docx",
    displayName: "paper.docx",
    fidelity: { classification: "exact" },
    sameFormatSave: "no_changes",
  };
}
