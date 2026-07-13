import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { createUnsavedDocument } from "./documentCreate";

const ENVELOPE = {
  schema_version: 1,
  document_id: "00000000-0000-4000-8000-000000000001",
  title: "Untitled document",
  document: { type: "doc", content: [] },
};

describe("createUnsavedDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("returns the Rust-owned initial envelope", async () => {
    invokeMock.mockResolvedValue({ status: "created", envelope: ENVELOPE });

    await expect(createUnsavedDocument()).resolves.toEqual({
      status: "created",
      envelope: ENVELOPE,
    });
    expect(invokeMock).toHaveBeenCalledWith("create_document", { request: {} });
  });

  it("preserves its typed command error", async () => {
    invokeMock.mockRejectedValue({ code: "template_invalid" });
    await expect(createUnsavedDocument()).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "template_invalid" },
    });
  });

  it("rejects malformed responses and unknown failures", async () => {
    invokeMock.mockResolvedValue({
      status: "created",
      envelope: { ...ENVELOPE, document_id: "frontend" },
    });
    await expect(createUnsavedDocument()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockRejectedValue({ code: "future_error", detail: "/private" });
    await expect(createUnsavedDocument()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
