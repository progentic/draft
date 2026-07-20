import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { getRuntimeStatus } from "./runtimeStatus";

const BUILD = {
  buildCommit: "0123456789abcdef0123456789abcdef01234567",
  buildProfile: "release" as const,
  version: "0.1.0",
};

describe("getRuntimeStatus", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokes the registered command with its typed request envelope", async () => {
    invokeMock.mockResolvedValue(BUILD);

    const result = await getRuntimeStatus();

    expect(invokeMock).toHaveBeenCalledWith("get_runtime_status", { request: {} });
    expect(result).toEqual({ status: "ready", ...BUILD });
  });

  it("rejects an invalid response shape", async () => {
    invokeMock.mockResolvedValue({ version: "", unexpected: true });

    await expect(getRuntimeStatus()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each([
    "event_delivery_failed",
    "invalid_application_version",
    "invalid_build_metadata",
  ] as const)(
    "preserves the %s command-specific error code",
    async (code) => {
      invokeMock.mockRejectedValue({ code });

      await expect(getRuntimeStatus()).resolves.toEqual({
        status: "error",
        error: { type: "command", code },
      });
    },
  );

  it("maps unknown failures to a transport error without leaking details", async () => {
    invokeMock.mockRejectedValue(new Error("Tauri runtime is unavailable"));

    await expect(getRuntimeStatus()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
