import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { setWindowTitle } from "./windowTitle";

describe("setWindowTitle", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("sends basename-level presentation state through the typed boundary", async () => {
    invokeMock.mockResolvedValue({ applied: true });

    await expect(
      setWindowTitle({ displayName: "Paper.draft", unsaved: true }),
    ).resolves.toEqual({ status: "applied" });
    expect(invokeMock).toHaveBeenCalledWith("set_window_title", {
      request: { displayName: "Paper.draft", unsaved: true },
    });
  });

  it.each(["invalid_title", "window_update_failed"] as const)(
    "preserves typed %s failures",
    async (code) => {
      invokeMock.mockRejectedValue({ code });
      await expect(
        setWindowTitle({ displayName: "Paper.draft", unsaved: false }),
      ).resolves.toEqual({ status: "error", code });
    },
  );

  it("contains unknown failures without exposing detail", async () => {
    invokeMock.mockRejectedValue(new Error("/private/path"));
    await expect(setWindowTitle({ displayName: null, unsaved: false })).resolves.toEqual({
      status: "error",
      code: "transport",
    });
  });
});
