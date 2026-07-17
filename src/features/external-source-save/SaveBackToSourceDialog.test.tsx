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
        normalizations: ["alternate_heading_style_name"],
      }}
      onResolve={resolve}
    />,
  );

  const dialog = screen.getByRole("alertdialog", { name: "Replace the source DOCX?" });
  const confirm = within(dialog).getByRole("button", { name: "Replace" });
  const cancel = within(dialog).getByRole("button", { name: "Cancel" });
  expect(within(dialog).getByText("What will change")).toBeTruthy();
  expect(
    within(dialog).getByText(
      "Alternate heading style names will use DRAFT’s standard heading names.",
    ),
  ).toBeTruthy();
  expect(within(dialog).getByText(/No unsupported source content/)).toBeTruthy();
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
      confirmation={{
        displayName: "paper.docx",
        disposition: "allowed_exact",
        normalizations: [],
      }}
      onResolve={vi.fn()}
    />,
  );

  const dialog = screen.getByRole("alertdialog");
  expect(within(dialog).getByRole("button", { name: "Replace" })).toBeTruthy();
  expect(within(dialog).queryByText(/normalize/i)).toBeNull();
});

it("explains canonical page-break normalization", () => {
  render(
    <SaveBackToSourceDialog
      confirmation={{
        displayName: "paper.docx",
        disposition: "allowed_after_accepted_normalization",
        normalizations: ["pagination_control"],
      }}
      onResolve={vi.fn()}
    />,
  );

  expect(
    screen.getByText("Page-break-before formatting will use DRAFT’s standard page break."),
  ).toBeTruthy();
});
