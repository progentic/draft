import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { closeDocument } from "./documentClose";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("closeDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokes the typed close command and validates the response", async () => {
    invokeMock.mockResolvedValue({ status: "closed", documentId: DOCUMENT_ID });

    await expect(closeDocument(DOCUMENT_ID)).resolves.toEqual({
      status: "closed",
      documentId: DOCUMENT_ID,
    });
    expect(invokeMock).toHaveBeenCalledWith("close_document", {
      request: { documentId: DOCUMENT_ID },
    });
  });

  it.each(["already_open", "not_open", "registry_unavailable", "source_path_in_use"])(
    "preserves the %s command error",
    async (code) => {
      invokeMock.mockRejectedValue({ code });

      await expect(closeDocument(DOCUMENT_ID)).resolves.toEqual({
        status: "error",
        error: { type: "command", code },
      });
    },
  );

  it("rejects malformed responses and sanitizes unknown failures", async () => {
    invokeMock.mockResolvedValue({ status: "closed", documentId: DOCUMENT_ID, path: "/private" });
    await expect(closeDocument(DOCUMENT_ID)).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockRejectedValue({ code: "future_error", detail: "/private" });
    await expect(closeDocument(DOCUMENT_ID)).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
