import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { getConnectivityMode } from "./connectivityMode";
import { setConnectivityMode } from "./connectivityModeSet";

describe("connectivity mode clients", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("reads the Rust-owned session mode", async () => {
    invokeMock.mockResolvedValue({ mode: "online" });

    await expect(getConnectivityMode()).resolves.toEqual({ status: "ready", mode: "online" });
    expect(invokeMock).toHaveBeenCalledWith("get_connectivity_mode", { request: {} });
  });

  it.each(["online", "offline"] as const)("sets the closed %s mode", async (mode) => {
    invokeMock.mockResolvedValue({ mode });

    await expect(setConnectivityMode(mode)).resolves.toEqual({ status: "ready", mode });
    expect(invokeMock).toHaveBeenCalledWith("set_connectivity_mode", { request: { mode } });
  });

  it.each([
    null,
    { mode: "automatic" },
    { mode: "online", persisted: true },
  ])("rejects an invalid response", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(getConnectivityMode()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it("rejects a set response that does not match the requested mode", async () => {
    invokeMock.mockResolvedValue({ mode: "online" });

    await expect(setConnectivityMode("offline")).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it("preserves the closed command error", async () => {
    invokeMock.mockRejectedValue({ code: "connectivity_unavailable" });

    await expect(getConnectivityMode()).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "connectivity_unavailable" },
    });
  });

  it("classifies unknown failures without retaining details", async () => {
    invokeMock.mockRejectedValue(new Error("private transport detail"));

    await expect(setConnectivityMode("offline")).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
