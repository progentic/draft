import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { SaveBackToSourceDialog } from "./SaveBackToSourceDialog";

it("explains normalized replacement and contains keyboard focus", async () => {
  const user = userEvent.setup();
  const resolve = vi.fn();
  const prior = document.createElement("button");
  document.body.append(prior);
  prior.focus();
  render(
    <SaveBackToSourceDialog
      confirmation={{
        displayName: "paper.docx",
        disposition: "allowed_after_accepted_normalization",
      }}
      onResolve={resolve}
    />,
  );

  const dialog = screen.getByRole("alertdialog", { name: "Replace the source DOCX?" });
  const confirm = within(dialog).getByRole("button", { name: "Accept and replace" });
  const cancel = within(dialog).getByRole("button", { name: "Keep source" });
  expect(within(dialog).getByText(/normalize its supported DOCX structure/)).toBeTruthy();
  expect(document.activeElement).toBe(confirm);

  cancel.focus();
  await user.tab();
  expect(document.activeElement).toBe(confirm);
  await user.keyboard("{Escape}");
  expect(resolve).toHaveBeenCalledWith("cancel");
});

it("uses exact-replacement copy without a normalization claim", () => {
  render(
    <SaveBackToSourceDialog
      confirmation={{ displayName: "paper.docx", disposition: "allowed_exact" }}
      onResolve={vi.fn()}
    />,
  );

  const dialog = screen.getByRole("alertdialog");
  expect(within(dialog).getByRole("button", { name: "Replace source" })).toBeTruthy();
  expect(within(dialog).queryByText(/normalize/i)).toBeNull();
});
