import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getConnectivityModeMock = vi.hoisted(() => vi.fn());
const setConnectivityModeMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/connectivityMode", async (importOriginal) => ({
  ...await importOriginal<typeof import("../../ipc/connectivityMode")>(),
  getConnectivityMode: getConnectivityModeMock,
}));

vi.mock("../../ipc/connectivityModeSet", () => ({
  setConnectivityMode: setConnectivityModeMock,
}));

import type { ConnectivityModeResult } from "../../ipc/connectivityMode";
import { useConnectivityMode } from "./useConnectivityMode";

describe("useConnectivityMode", () => {
  beforeEach(() => {
    getConnectivityModeMock.mockReset();
    setConnectivityModeMock.mockReset();
  });

  it("loads the effective Rust-owned mode", async () => {
    getConnectivityModeMock.mockResolvedValue(ready("online"));
    const { result } = renderHook(() => useConnectivityMode());

    await waitFor(() => expect(result.current.state).toEqual({ phase: "ready", mode: "online" }));
  });

  it("shows the prior mode while a change is pending", async () => {
    getConnectivityModeMock.mockResolvedValue(ready("online"));
    const change = deferred<ConnectivityModeResult>();
    setConnectivityModeMock.mockReturnValue(change.promise);
    const { result } = renderHook(() => useConnectivityMode());
    await waitFor(() => expect(result.current.state.phase).toBe("ready"));

    let request!: Promise<void>;
    act(() => { request = result.current.setMode("offline"); });
    expect(result.current.state).toEqual({ phase: "changing", mode: "online" });

    change.resolve(ready("offline"));
    await act(async () => { await request; });
    expect(result.current.state).toEqual({ phase: "ready", mode: "offline" });
  });

  it("keeps the prior effective mode when a change fails", async () => {
    getConnectivityModeMock.mockResolvedValue(ready("online"));
    setConnectivityModeMock.mockResolvedValue({
      status: "error",
      error: { type: "command", code: "connectivity_unavailable" },
    });
    const { result } = renderHook(() => useConnectivityMode());
    await waitFor(() => expect(result.current.state.phase).toBe("ready"));

    await act(async () => { await result.current.setMode("offline"); });

    expect(result.current.state).toEqual({
      phase: "failed",
      mode: "online",
      error: { type: "command", code: "connectivity_unavailable" },
    });
  });

  it("keeps the prior effective mode when a refresh fails", async () => {
    getConnectivityModeMock
      .mockResolvedValueOnce(ready("offline"))
      .mockResolvedValueOnce({ status: "error", error: { type: "transport" } });
    const { result } = renderHook(() => useConnectivityMode());
    await waitFor(() => expect(result.current.state.phase).toBe("ready"));

    await act(async () => { await result.current.refresh(); });

    expect(result.current.state).toEqual({
      phase: "failed",
      mode: "offline",
      error: { type: "transport" },
    });
  });

  it("ignores an older read after a newer refresh completes", async () => {
    const first = deferred<ConnectivityModeResult>();
    const second = deferred<ConnectivityModeResult>();
    getConnectivityModeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const { result } = renderHook(() => useConnectivityMode());

    let refresh!: Promise<void>;
    act(() => { refresh = result.current.refresh(); });
    second.resolve(ready("offline"));
    await act(async () => { await refresh; });
    expect(result.current.state).toEqual({ phase: "ready", mode: "offline" });

    first.resolve(ready("online"));
    await act(async () => { await first.promise; });
    expect(result.current.state).toEqual({ phase: "ready", mode: "offline" });
  });
});

function ready(mode: "online" | "offline"): ConnectivityModeResult {
  return { status: "ready", mode };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}
