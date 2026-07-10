import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const runFormattingReviewMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/formattingReview", async (importOriginal) => ({
  ...await importOriginal<typeof import("../../ipc/formattingReview")>(),
  runFormattingReview: runFormattingReviewMock,
}));

import type { FormattingReviewResult } from "../../ipc/formattingReview";
import { useFormattingReview } from "./useFormattingReview";

describe("useFormattingReview", () => {
  beforeEach(() => {
    runFormattingReviewMock.mockReset();
  });

  it("marks an in-flight result stale when the document changes", async () => {
    const response = deferred<FormattingReviewResult>();
    const editor = createEditor();
    runFormattingReviewMock.mockReturnValue(response.promise);
    const { result } = renderHook(() => useFormattingReview(editor));

    let run!: Promise<void>;
    act(() => { run = result.current.run("apa7"); });
    await waitFor(() => expect(result.current.state.phase).toBe("running"));
    act(() => { editor.commands.insertContent(" changed"); });
    expect(result.current.state.phase).toBe("stale");

    response.resolve(readyResult("First"));
    await act(async () => { await run; });
    expect(result.current.state.phase).toBe("stale");
    editor.destroy();
  });

  it("ignores an older run after a newer run becomes ready", async () => {
    const first = deferred<FormattingReviewResult>();
    const second = deferred<FormattingReviewResult>();
    const editor = createEditor();
    runFormattingReviewMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const { result } = renderHook(() => useFormattingReview(editor));

    let firstRun!: Promise<void>;
    let secondRun!: Promise<void>;
    act(() => { firstRun = result.current.run("apa7"); });
    act(() => { secondRun = result.current.run("mla9"); });
    second.resolve(readyResult("Current", "mla9"));
    await act(async () => { await secondRun; });
    expect(readyTitle(result.current.state)).toBe("Current");

    first.resolve(readyResult("Obsolete"));
    await act(async () => { await firstRun; });
    expect(readyTitle(result.current.state)).toBe("Current");
    editor.destroy();
  });
});

function createEditor() {
  return new Editor({
    extensions: [StarterKit.configure({ heading: { levels: [1, 2, 3] } })],
    content: { type: "doc", content: [{ type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Start" }] }] },
  });
}

function readyResult(title: string, style: "apa7" | "mla9" = "apa7"): FormattingReviewResult {
  return {
    status: "ready",
    review: {
      style,
      findings: [{
        code: "first_heading_not_level_one",
        severity: "advice",
        target: { type: "heading", index: 0 },
        title,
        explanation: "Review the outline start.",
        actions: [{ type: "inspect" }, { type: "apply_heading_level", level: 1 }, { type: "dismiss" }],
      }],
    },
  };
}

function readyTitle(state: ReturnType<typeof useFormattingReview>["state"]) {
  return state.phase === "ready" ? state.review.findings[0]?.title : undefined;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}
