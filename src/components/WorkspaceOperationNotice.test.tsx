import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { WorkspaceOperationNotice } from "./WorkspaceOperationNotice";

it("shows one accessible pending operation without permanent chrome", () => {
  const { rerender } = render(
    <WorkspaceOperationNotice message="Opening document…" pending />,
  );

  const notice = screen.getByRole("status");
  expect(notice.textContent).toBe("Opening document…");
  expect(notice.getAttribute("data-operation-state")).toBe("pending");

  rerender(<WorkspaceOperationNotice message="" pending={false} />);
  expect(screen.queryByRole("status")).toBeNull();
  expect(document.querySelector(".workspace-operation-notice--empty")).toBeTruthy();
});
