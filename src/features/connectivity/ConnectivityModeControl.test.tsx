import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ConnectivityModeControl } from "./ConnectivityModeControl";

describe("ConnectivityModeControl", () => {
  it("requests offline mode from the labeled online toggle", async () => {
    const user = userEvent.setup();
    const onSetMode = vi.fn();
    render(
      <ConnectivityModeControl
        state={{ phase: "ready", mode: "online" }}
        onRefresh={vi.fn()}
        onSetMode={onSetMode}
      />,
    );

    const toggle = screen.getByRole("button", { name: "Work offline" });
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    expect(toggle.textContent).toContain("Online");
    await user.click(toggle);
    expect(onSetMode).toHaveBeenCalledWith("offline");
  });

  it("requests online mode from the pressed offline toggle", async () => {
    const user = userEvent.setup();
    const onSetMode = vi.fn();
    render(
      <ConnectivityModeControl
        state={{ phase: "ready", mode: "offline" }}
        onRefresh={vi.fn()}
        onSetMode={onSetMode}
      />,
    );

    const toggle = screen.getByRole("button", { name: "Go online" });
    expect(toggle.getAttribute("aria-pressed")).toBe("true");
    await user.click(toggle);
    expect(onSetMode).toHaveBeenCalledWith("online");
  });

  it("keeps the effective mode visible and announces a failed change", () => {
    render(
      <ConnectivityModeControl
        state={{ phase: "failed", mode: "online", error: { type: "transport" } }}
        onRefresh={vi.fn()}
        onSetMode={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "Work offline" }).textContent).toContain(
      "Online - change failed",
    );
    expect(screen.getByRole("alert").textContent).toBe(
      "DRAFT could not reach the core to change connectivity mode. DRAFT remains online.",
    );
    expect(screen.getByRole("alert").getAttribute("aria-atomic")).toBe("true");
  });

  it("offers a retry when the effective mode cannot be read", async () => {
    const user = userEvent.setup();
    const onRefresh = vi.fn();
    const { rerender } = render(
      <ConnectivityModeControl
        state={{ phase: "failed", error: { type: "invalid-response" } }}
        onRefresh={onRefresh}
        onSetMode={vi.fn()}
      />,
    );

    const retry = screen.getByRole("button", { name: "Retry connectivity status" });
    expect(retry.getAttribute("type")).toBe("button");
    expect(screen.getByRole("alert").textContent).toContain("invalid connectivity response");

    retry.focus();
    rerender(
      <ConnectivityModeControl
        state={{ phase: "failed", error: { type: "invalid-response" } }}
        onRefresh={onRefresh}
        onSetMode={vi.fn()}
      />,
    );
    expect(document.activeElement).toBe(retry);
    expect(screen.getAllByRole("alert")).toHaveLength(1);

    await user.click(retry);
    expect(onRefresh).toHaveBeenCalledOnce();
  });
});
