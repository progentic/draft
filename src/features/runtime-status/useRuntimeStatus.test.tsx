import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

type StateHandler = (state: { phase: "ready"; version: string }) => void;

const startRuntimeStatusSessionMock = vi.hoisted(() => vi.fn());
const stopMock = vi.hoisted(() => vi.fn());

vi.mock("./runtimeStatusSession", () => ({
  startRuntimeStatusSession: startRuntimeStatusSessionMock,
}));

import { useRuntimeStatus } from "./useRuntimeStatus";

describe("useRuntimeStatus", () => {
  let stateHandler: StateHandler | undefined;

  beforeEach(() => {
    stateHandler = undefined;
    startRuntimeStatusSessionMock.mockReset();
    stopMock.mockReset();
    startRuntimeStatusSessionMock.mockImplementation(async (handler: StateHandler) => {
      stateHandler = handler;
      return stopMock;
    });
  });

  it("publishes event-driven state and removes the listener on unmount", async () => {
    const { result, unmount } = renderHook(() => useRuntimeStatus());
    expect(result.current).toEqual({ phase: "checking" });

    await waitFor(() => expect(startRuntimeStatusSessionMock).toHaveBeenCalledOnce());
    act(() => stateHandler?.({ phase: "ready", version: "0.1.0" }));
    expect(result.current).toEqual({ phase: "ready", version: "0.1.0" });

    await act(async () => Promise.resolve());
    unmount();

    expect(stopMock).toHaveBeenCalledOnce();
  });
});
