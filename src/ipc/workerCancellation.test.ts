import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { cancelWorker } from "./workerCancellation";

const WORKER_ID = "00000000-0000-4000-8000-000000000001";

describe("cancelWorker", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it.each([
    ["cancellation_requested", { status: "cancellation-requested" }],
    ["already_ended", { status: "already-ended" }],
  ] as const)("maps the %s response", async (status, expected) => {
    invokeMock.mockResolvedValue({ status });

    await expect(cancelWorker({ workerId: WORKER_ID })).resolves.toEqual(expected);
    expect(invokeMock).toHaveBeenCalledWith("cancel_worker", {
      request: { workerId: WORKER_ID },
    });
  });

  it("rejects an unknown response shape", async () => {
    invokeMock.mockResolvedValue({ status: "cancelled", extra: true });

    await expect(cancelWorker({ workerId: WORKER_ID })).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each(["invalid_worker_id", "worker_not_found", "registry_unavailable"] as const)(
    "preserves the %s command error",
    async (code) => {
      invokeMock.mockRejectedValue({ code });

      await expect(cancelWorker({ workerId: WORKER_ID })).resolves.toEqual({
        status: "error",
        error: { type: "command", code },
      });
    },
  );

  it("classifies unknown failures without leaking details", async () => {
    invokeMock.mockRejectedValue(new Error("private transport detail"));

    await expect(cancelWorker({ workerId: WORKER_ID })).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
