import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { WorkspaceStatusBar } from "./WorkspaceStatusBar";

it("presents document, operation, connectivity, and recovery state", async () => {
  const user = userEvent.setup();
  const setMode = vi.fn();
  render(
    <WorkspaceStatusBar
      connectivityState={{ phase: "ready", mode: "online" }}
      documentStatus="Imported, unsaved"
      exportPending
      operation="ready"
      runtimeStatus={{
        buildCommit: "0123456789abcdef0123456789abcdef01234567",
        buildProfile: "release",
        phase: "ready",
        version: "0.1.0",
      }}
      onRefreshConnectivity={vi.fn()}
      onSetConnectivityMode={setMode}
    />,
  );

  const status = screen.getByLabelText("Workspace status");
  expect(within(status).getByLabelText("Document state").textContent).toContain(
    "Imported, unsaved",
  );
  expect(within(status).getByLabelText("Background operation").textContent).toContain(
    "Exporting",
  );
  expect(within(status).getByLabelText("Application build").textContent).toBe(
    "v0.1.0 · 01234567",
  );
  await user.click(within(status).getByRole("button", { name: "Work offline" }));
  expect(setMode).toHaveBeenCalledWith("offline");
});
