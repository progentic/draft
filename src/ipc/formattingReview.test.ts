import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import {
  FORMATTING_STYLES,
  runFormattingReview,
  type FormattingReviewCommandErrorCode,
  type FormattingReviewRequest,
} from "./formattingReview";

describe("runFormattingReview", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokes the typed command and accepts its closed action policy", async () => {
    invokeMock.mockResolvedValue(validResponse());

    await expect(runFormattingReview(request())).resolves.toEqual({
      status: "ready",
      review: validResponse(),
    });
    expect(invokeMock).toHaveBeenCalledWith("run_formatting_review", {
      request: request(),
    });
  });

  it.each(FORMATTING_STYLES)("accepts the closed %s style identifier", async (style) => {
    const input = request(style);
    invokeMock.mockResolvedValue({ style, findings: [] });

    await expect(runFormattingReview(input)).resolves.toEqual({
      status: "ready",
      review: { style, findings: [] },
    });
  });

  it.each([
    { ...validResponse(), style: "apa7" },
    { ...validResponse(), unexpected: true },
    responseWith({ target: { type: "heading", index: 2 } }),
    responseWith({ actions: [{ type: "inspect" }, { type: "dismiss" }] }),
    responseWith({ title: "x".repeat(129) }),
  ])("rejects an invalid or over-authoritative response", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(runFormattingReview(request())).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each([
    "too_many_headings",
    "too_many_citations",
    "invalid_heading_level",
    "empty_heading_title",
    "heading_title_too_long",
    "invalid_citekey",
  ] satisfies FormattingReviewCommandErrorCode[])(
    "preserves the %s command error",
    async (code) => {
      invokeMock.mockRejectedValue({ code });

      await expect(runFormattingReview(request())).resolves.toEqual({
        status: "error",
        error: { type: "command", code },
      });
    },
  );

  it("classifies unknown failures without exposing details", async () => {
    invokeMock.mockRejectedValue(new Error("private document content"));

    await expect(runFormattingReview(request())).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});

function request(style: FormattingReviewRequest["style"] = "mla9"): FormattingReviewRequest {
  return {
    style,
    headings: [
      { level: 2, title: "Start" },
      { level: 4, title: "Detail" },
    ],
    citations: [{ citekey: "smith2025", renderStyle: "apa7" }],
  };
}

function validResponse() {
  return {
    style: "mla9",
    findings: [
      {
        code: "first_heading_not_level_one",
        severity: "advice",
        target: { type: "heading", index: 0 },
        title: "Outline starts below level 1",
        explanation: "Review the outline start.",
        actions: [
          { type: "inspect" },
          { type: "apply_heading_level", level: 1 },
          { type: "dismiss" },
        ],
      },
    ],
  };
}

function responseWith(overrides: Record<string, unknown>) {
  const response = validResponse();
  return {
    ...response,
    findings: [{ ...response.findings[0], ...overrides }],
  };
}
