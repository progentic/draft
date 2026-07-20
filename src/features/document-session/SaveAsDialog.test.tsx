import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { SaveAsDialog } from "./SaveAsDialog";

describe("SaveAsDialog", () => {
  it("offers only the three supported formats and excludes PDF", () => {
    render(<SaveAsDialog open onResolve={vi.fn()} />);

    const format = screen.getByRole("combobox", { name: "File format" });
    expect(within(format).getAllByRole("option").map((option) => option.textContent)).toEqual([
      "DRAFT Document (.draft)",
      "Word Document (.docx)",
      "Plain Text (.txt)",
    ]);
    expect(screen.queryByText(/pdf/i)).toBeNull();
  });

  it("resolves the explicitly selected converted format", async () => {
    const user = userEvent.setup();
    const onResolve = vi.fn();
    render(<SaveAsDialog open onResolve={onResolve} />);

    const format = screen.getByRole("combobox", { name: "File format" });
    await user.selectOptions(format, "docx");
    expect(screen.getByText(/active DRAFT document and Unsaved state do not change/i))
      .toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Continue" }));

    expect(onResolve).toHaveBeenCalledOnce();
    expect(onResolve).toHaveBeenCalledWith("docx");
  });

  it("contains keyboard focus and cancels with Escape", async () => {
    const user = userEvent.setup();
    const onResolve = vi.fn();
    render(<SaveAsDialog open onResolve={onResolve} />);

    const format = screen.getByRole("combobox", { name: "File format" });
    await vi.waitFor(() => expect(document.activeElement).toBe(format));
    const cancel = screen.getByRole("button", { name: "Cancel" });
    cancel.focus();
    fireEvent.keyDown(cancel, { key: "Tab", code: "Tab", keyCode: 9 });
    expect(document.activeElement).toBe(format);
    await user.keyboard("{Escape}");

    expect(onResolve).toHaveBeenCalledOnce();
    expect(onResolve).toHaveBeenCalledWith("cancel");
  });

  it("restores the invoking control after the dialog closes", async () => {
    const prior = document.createElement("button");
    document.body.append(prior);
    prior.focus();
    const { rerender } = render(<SaveAsDialog open onResolve={vi.fn()} />);
    await vi.waitFor(() => {
      expect(document.activeElement).toBe(
        screen.getByRole("combobox", { name: "File format" }),
      );
    });

    rerender(<SaveAsDialog open={false} onResolve={vi.fn()} />);

    expect(document.activeElement).toBe(prior);
    prior.remove();
  });
});
