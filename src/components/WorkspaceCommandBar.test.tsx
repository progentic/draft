import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import type { WorkspaceActions } from "../features/workspace-actions/useWorkspaceActions";
import { WorkspaceCommandBar } from "./WorkspaceCommandBar";

it("keeps New visible and gives icon-only document controls accessible names", () => {
  renderCommandBar();

  expect(screen.getByRole("button", { name: "New Document" }).textContent).toContain("New");
  for (const label of ["Open…", "Save", "Close"]) {
    const button = screen.getByRole("button", { name: label });
    expect(button.textContent).toBe("");
    expect(button.getAttribute("title")).toBe(label);
  }
});

it("routes primary and overflow actions through the shared dispatcher", async () => {
  const user = userEvent.setup();
  const actions = workspaceActions();
  renderCommandBar(actions);

  await user.click(screen.getByRole("button", { name: "New Document" }));
  await user.click(screen.getByRole("button", { name: "Open…" }));
  await user.click(screen.getByRole("button", { name: "More document actions" }));
  await user.click(screen.getByRole("menuitem", { name: "Save As…" }));

  expect(actions.dispatch).toHaveBeenNthCalledWith(1, "new_document");
  expect(actions.dispatch).toHaveBeenNthCalledWith(2, "open_document");
  expect(actions.dispatch).toHaveBeenNthCalledWith(3, "save_document_as");
});

it("supports overflow keyboard navigation and restores trigger focus", async () => {
  const user = userEvent.setup();
  renderCommandBar();
  const trigger = screen.getByRole("button", { name: "More document actions" });

  trigger.focus();
  await user.keyboard("{ArrowDown}");
  const menu = screen.getByRole("menu", { name: "More document actions" });
  await waitFor(() => expect(document.activeElement).toBe(
    within(menu).getByRole("menuitem", { name: "Save As…" }),
  ));
  await user.keyboard("{ArrowDown}");
  expect(document.activeElement).toBe(within(menu).getByRole("menuitem", { name: "Export DOCX…" }));
  await user.keyboard("{End}");
  expect(document.activeElement).toBe(within(menu).getByRole("menuitemcheckbox", { name: "Text checks" }));
  await user.keyboard("{Escape}");

  expect(document.activeElement).toBe(trigger);
  expect(screen.queryByRole("menu", { name: "More document actions" })).toBeNull();
});

it("skips disabled overflow actions and exposes active panel state", async () => {
  const user = userEvent.setup();
  const actions = workspaceActions({ save_document_as: false });
  renderCommandBar(actions, "references");

  await user.click(screen.getByRole("button", { name: "More document actions" }));
  const menu = screen.getByRole("menu", { name: "More document actions" });
  const saveAs = within(menu).getByRole("menuitem", { name: "Save As…" });
  const references = within(menu).getByRole("menuitemcheckbox", { name: "References" });

  expect((saveAs as HTMLButtonElement).disabled).toBe(true);
  await waitFor(() => expect(document.activeElement).toBe(
    within(menu).getByRole("menuitem", { name: "Export DOCX…" }),
  ));
  expect(references.getAttribute("aria-checked")).toBe("true");
});

function renderCommandBar(
  actions = workspaceActions(),
  activePanel: "references" | "text-review" | null = null,
) {
  return render(
    <WorkspaceCommandBar
      actions={actions}
      activePanel={activePanel}
      exportLabel="Export DOCX…"
    />,
  );
}

function workspaceActions(
  patch: Partial<WorkspaceActions["enabled"]> = {},
): WorkspaceActions {
  return {
    dispatch: vi.fn(),
    enabled: {
      new_document: true,
      open_document: true,
      close_document: true,
      save_document: true,
      save_document_as: true,
      export_docx: true,
      open_references: true,
      run_text_checks: true,
      ...patch,
    },
    feedback: "",
  };
}
