import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { FINDING_POLICIES, runTextAnalysis } from "./textAnalysis";

const FINDING = {
  code: "repeated_word",
  ...FINDING_POLICIES.repeated_word,
  startByte: 4,
  endByte: 12,
};

describe("runTextAnalysis", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("exposes exactly the five approved heuristic checks", () => {
    expect(Object.keys(FINDING_POLICIES)).toEqual([
      "repeated_word",
      "long_sentence",
      "all_caps_emphasis",
      "repeated_sentence_opener",
      "mixed_first_person",
    ]);
  });

  it("returns validated findings through the typed Rust boundary", async () => {
    invokeMock.mockResolvedValue({ result: { findings: [FINDING] } });

    await expect(runTextAnalysis("The the draft.")).resolves.toEqual({
      status: "ready",
      findings: [FINDING],
    });
    expect(invokeMock).toHaveBeenCalledWith("run_text_analysis", {
      request: { text: "The the draft." },
    });
  });

  it.each([
    "cancelled",
    "empty_text",
    "helper_failed",
    "invalid_output",
    "runtime_unavailable",
    "text_too_long",
    "timed_out",
    "worker_unavailable",
  ])("preserves the %s command error", async (code) => {
    invokeMock.mockRejectedValue({ code });

    await expect(runTextAnalysis("Draft")).resolves.toEqual({
      status: "error",
      error: { type: "command", code },
    });
  });

  it("rejects policy drift, extra findings, and private failures", async () => {
    invokeMock.mockResolvedValue({
      result: { findings: [{ ...FINDING, title: "Definitive error" }] },
    });
    await expect(runTextAnalysis("Draft")).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockResolvedValue({
      result: { findings: [{ ...FINDING, code: "semantic_quality" }] },
    });
    await expect(runTextAnalysis("Draft")).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockRejectedValue({ code: "future_error", detail: "/private" });
    await expect(runTextAnalysis("Draft")).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
