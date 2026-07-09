import { beforeEach, describe, expect, it, vi } from "vitest";

type ReadyHandler = (event: { type: "ready"; version: string }) => void;
type ErrorHandler = (error: { type: "invalid-payload" }) => void;

const getRuntimeStatusMock = vi.hoisted(() => vi.fn());
const listenToRuntimeStatusMock = vi.hoisted(() => vi.fn());
const stopMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/runtimeStatus", () => ({
  getRuntimeStatus: getRuntimeStatusMock,
}));

vi.mock("../../ipc/runtimeStatusEvents", () => ({
  listenToRuntimeStatus: listenToRuntimeStatusMock,
}));

import { startRuntimeStatusSession } from "./runtimeStatusSession";

describe("startRuntimeStatusSession", () => {
  beforeEach(() => {
    getRuntimeStatusMock.mockReset();
    listenToRuntimeStatusMock.mockReset();
    stopMock.mockReset();
  });

  it("registers before invoking and resolves state from the typed event", async () => {
    const callOrder: string[] = [];
    const states: unknown[] = [];
    listenToRuntimeStatusMock.mockImplementation(
      async (onEvent: ReadyHandler, _onError: ErrorHandler) => {
        callOrder.push("listen");
        onEvent({ type: "ready", version: "0.1.0" });
        return stopMock;
      },
    );
    getRuntimeStatusMock.mockImplementation(async () => {
      callOrder.push("command");
      return { status: "ready", version: "0.1.0" };
    });

    const stop = await startRuntimeStatusSession((state) => states.push(state));
    stop();

    expect(callOrder).toEqual(["listen", "command"]);
    expect(states).toEqual([{ phase: "ready", version: "0.1.0" }]);
    expect(stopMock).toHaveBeenCalledOnce();
  });

  it("maps an invalid event payload to unavailable", async () => {
    const states: unknown[] = [];
    listenToRuntimeStatusMock.mockImplementation(
      async (_onEvent: ReadyHandler, onError: ErrorHandler) => {
        onError({ type: "invalid-payload" });
        return stopMock;
      },
    );
    getRuntimeStatusMock.mockResolvedValue({ status: "ready", version: "0.1.0" });

    await startRuntimeStatusSession((state) => states.push(state));

    expect(states).toEqual([{ phase: "unavailable", reason: "invalid-payload" }]);
  });

  it("maps command failure after listener setup and keeps cleanup", async () => {
    const states: unknown[] = [];
    listenToRuntimeStatusMock.mockResolvedValue(stopMock);
    getRuntimeStatusMock.mockResolvedValue({
      status: "error",
      error: { type: "command", code: "event_delivery_failed" },
    });

    const stop = await startRuntimeStatusSession((state) => states.push(state));
    stop();

    expect(states).toEqual([{ phase: "unavailable", reason: "command" }]);
    expect(stopMock).toHaveBeenCalledOnce();
  });

  it("maps listener setup failure without invoking the command", async () => {
    const states: unknown[] = [];
    listenToRuntimeStatusMock.mockRejectedValue(new Error("plugin unavailable"));

    const stop = await startRuntimeStatusSession((state) => states.push(state));
    stop();

    expect(states).toEqual([{ phase: "unavailable", reason: "transport" }]);
    expect(getRuntimeStatusMock).not.toHaveBeenCalled();
    expect(stopMock).not.toHaveBeenCalled();
  });
});
