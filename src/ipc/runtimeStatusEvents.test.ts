import { beforeEach, describe, expect, it, vi } from "vitest";

type RawEventHandler = (event: { payload: unknown }) => void;

const listenMock = vi.hoisted(() => vi.fn());
const stopMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

import { listenToRuntimeStatus } from "./runtimeStatusEvents";

describe("listenToRuntimeStatus", () => {
  let eventHandler: RawEventHandler | undefined;

  beforeEach(() => {
    eventHandler = undefined;
    listenMock.mockReset();
    stopMock.mockReset();
    listenMock.mockImplementation(async (_name: string, handler: RawEventHandler) => {
      eventHandler = handler;
      return stopMock;
    });
  });

  it("delivers a validated event and exposes listener cleanup", async () => {
    const onEvent = vi.fn();
    const onError = vi.fn();

    const stop = await listenToRuntimeStatus(onEvent, onError);
    eventHandler?.({ payload: { type: "ready", version: "0.1.0" } });
    stop();

    expect(listenMock).toHaveBeenCalledWith("draft://runtime-status", expect.any(Function));
    expect(onEvent).toHaveBeenCalledWith({ type: "ready", version: "0.1.0" });
    expect(onError).not.toHaveBeenCalled();
    expect(stopMock).toHaveBeenCalledOnce();
  });

  it("rejects unknown event payloads without forwarding them", async () => {
    const onEvent = vi.fn();
    const onError = vi.fn();

    await listenToRuntimeStatus(onEvent, onError);
    eventHandler?.({ payload: { type: "ready", version: "", extra: true } });

    expect(onEvent).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledWith({ type: "invalid-payload" });
  });

  it("preserves event-listener setup rejection for bounded session mapping", async () => {
    listenMock.mockRejectedValue(new Error("event plugin unavailable"));

    await expect(listenToRuntimeStatus(vi.fn(), vi.fn())).rejects.toThrow(
      "event plugin unavailable",
    );
  });
});
