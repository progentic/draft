import type { Editor } from "@tiptap/react";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const useFormattingReviewMock = vi.hoisted(() => vi.fn());

vi.mock("./useFormattingReview", () => ({
  useFormattingReview: useFormattingReviewMock,
}));

import type { FormattingReviewFinding } from "../../ipc/formattingReview";
import { FormattingReviewPanel } from "./FormattingReviewPanel";

const run = vi.fn();
const invalidate = vi.fn();
const inspect = vi.fn();
const apply = vi.fn();
const dismiss = vi.fn();
const editor = {} as Editor;

describe("FormattingReviewPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useFormattingReviewMock.mockReturnValue({
      state: { phase: "idle" }, run, invalidate, inspect, apply, dismiss,
    });
  });

  it("exposes labeled style, run, and close controls", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(<FormattingReviewPanel editor={editor} isOpen onClose={onClose} />);

    expect(screen.getByRole("heading", { level: 2, name: "Formatting review" })).toBeTruthy();
    expect(screen.getByRole("radio", { name: "APA 7" }).getAttribute("checked")).not.toBeNull();
    expect(screen.getByRole("button", { name: "Close formatting review" })).toBeTruthy();

    await user.click(screen.getByRole("radio", { name: "MLA 9" }));
    await user.click(screen.getByRole("button", { name: "Check formatting" }));
    await user.click(screen.getByRole("button", { name: "Close formatting review" }));

    expect(invalidate).toHaveBeenCalledOnce();
    expect(run).toHaveBeenCalledWith("mla9");
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("keeps findings grouped and requires explicit review actions", async () => {
    const user = userEvent.setup();
    const heading = headingFinding();
    const citation = citationFinding();
    useFormattingReviewMock.mockReturnValue({
      state: readyState([heading, citation]), run, invalidate, inspect, apply, dismiss,
    });
    render(<FormattingReviewPanel editor={editor} isOpen onClose={vi.fn()} />);

    const structure = screen.getByRole("region", { name: "Structure findings" });
    const citations = screen.getByRole("region", { name: "Citations findings" });
    expect(within(structure).getByText("Heading 1")).toBeTruthy();
    expect(within(citations).getByText("Citation 1")).toBeTruthy();
    expect(within(citations).queryByRole("button", { name: /Apply H/ })).toBeNull();

    await user.click(within(structure).getByRole("button", { name: "Inspect" }));
    await user.click(within(structure).getByRole("button", { name: "Apply H1" }));
    await user.click(within(citations).getByRole("button", { name: `Dismiss ${citation.title}` }));

    expect(inspect).toHaveBeenCalledWith(heading);
    expect(apply).toHaveBeenCalledWith(heading, 1);
    expect(dismiss).toHaveBeenCalledWith(citation);
    expect(
      within(citations).getByRole("button", { name: `Dismiss ${citation.title}` }).getAttribute(
        "type",
      ),
    ).toBe("button");
  });

  it("announces stale and typed failure states", () => {
    useFormattingReviewMock.mockReturnValue({
      state: { phase: "stale" }, run, invalidate, inspect, apply, dismiss,
    });
    const { rerender } = render(
      <FormattingReviewPanel editor={editor} isOpen onClose={vi.fn()} />,
    );
    expect(screen.getByRole("status").textContent).toContain("document changed");

    useFormattingReviewMock.mockReturnValue({
      state: { phase: "failed", error: { type: "command", code: "invalid_citekey" } },
      run, invalidate, inspect, apply, dismiss,
    });
    rerender(<FormattingReviewPanel editor={editor} isOpen onClose={vi.fn()} />);
    const retry = screen.getByRole("button", { name: "Check again" });
    expect(retry.getAttribute("type")).toBe("button");
    expect(screen.getByRole("alert").textContent).toBe(
      "DRAFT could not validate a citation. Correct or remove it, then check again.",
    );
    expect(screen.getByRole("alert").getAttribute("aria-atomic")).toBe("true");

    retry.focus();
    rerender(<FormattingReviewPanel editor={editor} isOpen onClose={vi.fn()} />);
    expect(document.activeElement).toBe(retry);
    expect(screen.getAllByRole("alert")).toHaveLength(1);
  });
});

function readyState(findings: FormattingReviewFinding[]) {
  return {
    phase: "ready" as const,
    review: { style: "apa7" as const, findings },
    snapshot: { request: { style: "apa7" as const, headings: [], citations: [] }, headings: [], citations: [] },
    generation: 1,
  };
}

function headingFinding(): FormattingReviewFinding {
  return {
    code: "first_heading_not_level_one",
    severity: "advice",
    target: { type: "heading", index: 0 },
    title: "Outline starts below level 1",
    explanation: "Review the outline start.",
    actions: [{ type: "inspect" }, { type: "apply_heading_level", level: 1 }, { type: "dismiss" }],
  };
}

function citationFinding(): FormattingReviewFinding {
  return {
    code: "citation_style_mismatch",
    severity: "warning",
    target: { type: "citation", index: 0 },
    title: "Citation style differs",
    explanation: "Review this citation.",
    actions: [{ type: "inspect" }, { type: "dismiss" }],
  };
}
