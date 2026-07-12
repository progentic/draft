import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { addReference } from "./referenceLibraryAdd";
import { listReferences } from "./referenceLibraryList";

const INPUT = { citekey: "ada2026", title: "Notes", author: "Ada", year: 2026 };
const SUMMARY = { citekey: "ada2026", title: "Notes" };

describe("reference library clients", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("lists validated summaries through the Rust command", async () => {
    invokeMock.mockResolvedValue([SUMMARY]);

    await expect(listReferences()).resolves.toEqual({ status: "ready", value: [SUMMARY] });
    expect(invokeMock).toHaveBeenCalledWith("list_references", { request: {} });
  });

  it("adds a manual reference without expanding the visible record", async () => {
    invokeMock.mockResolvedValue(SUMMARY);

    await expect(addReference(INPUT)).resolves.toEqual({ status: "ready", value: SUMMARY });
    expect(invokeMock).toHaveBeenCalledWith("add_reference", { request: INPUT });
  });

  it.each([
    "duplicate_citekey",
    "invalid_reference",
    "read_failed",
    "store_unavailable",
    "write_failed",
  ])("preserves the %s command error", async (code) => {
    invokeMock.mockRejectedValue({ code });

    await expect(listReferences()).resolves.toEqual({
      status: "error",
      error: { type: "command", code },
    });
  });

  it("rejects expanded summaries and sanitizes unknown failures", async () => {
    invokeMock.mockResolvedValue([{ ...SUMMARY, path: "/private" }]);
    await expect(listReferences()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockRejectedValue({ code: "future_error", detail: "/private" });
    await expect(listReferences()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
