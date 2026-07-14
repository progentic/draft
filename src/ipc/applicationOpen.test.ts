import { beforeEach, describe, expect, it, vi } from "vitest";

type RawEventHandler = (event: { payload: unknown }) => void;

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());
const stopMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

import {
  dismissApplicationOpenRequest,
  listenToApplicationOpenRequests,
  takeApplicationOpenRequest,
} from "./applicationOpen";

describe("application open boundary", () => {
  let eventHandler: RawEventHandler | undefined;

  beforeEach(() => {
    eventHandler = undefined;
    invokeMock.mockReset();
    listenMock.mockReset();
    stopMock.mockReset();
    listenMock.mockImplementation(async (_name: string, handler: RawEventHandler) => {
      eventHandler = handler;
      return stopMock;
    });
  });

  it("opens one queued document without sending a path", async () => {
    invokeMock.mockResolvedValue({
      status: "opened",
      result: { status: "opened_draft", envelope: envelope() },
    });

    await expect(takeApplicationOpenRequest()).resolves.toEqual({
      status: "open",
      result: { status: "opened_draft", envelope: envelope() },
    });
    expect(invokeMock).toHaveBeenCalledWith("open_application_document", {
      request: { disposition: "open" },
    });
  });

  it("preserves typed path-free queue and open failures", async () => {
    invokeMock.mockRejectedValueOnce({ code: "multiple_files_unsupported" });
    await expect(takeApplicationOpenRequest()).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "multiple_files_unsupported" },
    });

    invokeMock.mockRejectedValueOnce({
      code: "open",
      cause: { code: "file_not_found" },
    });
    await expect(takeApplicationOpenRequest()).resolves.toEqual({
      status: "error",
      error: { type: "open", error: { code: "file_not_found" } },
    });
  });

  it("dismisses a queued request without opening it", async () => {
    invokeMock.mockResolvedValue({ status: "dismissed" });

    await expect(dismissApplicationOpenRequest()).resolves.toBe(true);
    expect(invokeMock).toHaveBeenCalledWith("open_application_document", {
      request: { disposition: "dismiss" },
    });
  });

  it("delivers only path-free availability events", async () => {
    const onAvailable = vi.fn();
    const onError = vi.fn();
    const stop = await listenToApplicationOpenRequests(onAvailable, onError);

    eventHandler?.({ payload: { type: "available" } });
    expect(onAvailable).toHaveBeenCalledOnce();
    expect(onError).not.toHaveBeenCalled();

    eventHandler?.({ payload: { type: "available", path: "/tmp/notes.draft" } });
    expect(onError).toHaveBeenCalledWith({ type: "invalid-payload" });
    stop();
    expect(stopMock).toHaveBeenCalledOnce();
  });
});

function envelope() {
  return {
    schema_version: 2,
    document_id: "00000000-0000-4000-8000-000000000001",
    title: "Opened from Finder",
    document: { type: "doc", content: [{ type: "paragraph" }] },
  };
}
