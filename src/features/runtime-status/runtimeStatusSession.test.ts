import { beforeEach, describe, expect, it, vi } from "vitest";

type ReadyHandler = (event: typeof READY_EVENT) => void;
type ErrorHandler = (error: { type: "invalid-payload" }) => void;

const getRuntimeStatusMock = vi.hoisted(() => vi.fn());
const listenToRuntimeStatusMock = vi.hoisted(() => vi.fn());
const stopMock = vi.hoisted(() => vi.fn());
const READY_EVENT = {
  type: "ready" as const,
  buildCommit: "0123456789abcdef0123456789abcdef01234567",
  buildProfile: "release" as const,
  version: "0.1.0",
};
const READY_RESULT = {
  status: "ready" as const,
  buildCommit: READY_EVENT.buildCommit,
  buildProfile: READY_EVENT.buildProfile,
  version: READY_EVENT.version,
};

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
        onEvent(READY_EVENT);
        return stopMock;
      },
    );
    getRuntimeStatusMock.mockImplementation(async () => {
      callOrder.push("command");
      return READY_RESULT;
    });

    const stop = await startRuntimeStatusSession((state) => states.push(state));
    stop();

    expect(callOrder).toEqual(["listen", "command"]);
    expect(states).toEqual([
      {
        phase: "ready",
        buildCommit: READY_EVENT.buildCommit,
        buildProfile: "release",
        version: "0.1.0",
      },
    ]);
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
    getRuntimeStatusMock.mockResolvedValue(READY_RESULT);

    await startRuntimeStatusSession((state) => states.push(state));

    expect(states).toEqual([
      { phase: "unavailable", reason: { type: "invalid-payload" } },
    ]);
  });

  it.each([
    "event_delivery_failed",
    "invalid_application_version",
    "invalid_build_metadata",
  ] as const)(
    "preserves the %s command failure after listener setup",
    async (code) => {
      const states: unknown[] = [];
      listenToRuntimeStatusMock.mockResolvedValue(stopMock);
      getRuntimeStatusMock.mockResolvedValue({
        status: "error",
        error: { type: "command", code },
      });

      const stop = await startRuntimeStatusSession((state) => states.push(state));
      stop();

      expect(states).toEqual([
        { phase: "unavailable", reason: { type: "command", code } },
      ]);
      expect(stopMock).toHaveBeenCalledOnce();
    },
  );

  it("maps listener setup failure without invoking the command", async () => {
    const states: unknown[] = [];
    listenToRuntimeStatusMock.mockRejectedValue(new Error("plugin unavailable"));

    const stop = await startRuntimeStatusSession((state) => states.push(state));
    stop();

    expect(states).toEqual([{ phase: "unavailable", reason: { type: "transport" } }]);
    expect(getRuntimeStatusMock).not.toHaveBeenCalled();
    expect(stopMock).not.toHaveBeenCalled();
  });
});
