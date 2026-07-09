import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { saveDocument } from "./documentSave";
import type { DocumentEnvelopeSnapshot } from "./documentEnvelope";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("saveDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("sends the explicit immutable snapshot to Rust", async () => {
    const snapshot = envelope();
    invokeMock.mockResolvedValue({ status: "saved", documentId: DOCUMENT_ID });

    await expect(saveDocument(snapshot)).resolves.toEqual({
      status: "saved",
      documentId: DOCUMENT_ID,
    });
    expect(invokeMock).toHaveBeenCalledWith("save_document", { request: { snapshot } });
  });

  it("preserves user cancellation", async () => {
    invokeMock.mockResolvedValue({ status: "cancelled" });

    await expect(saveDocument(envelope())).resolves.toEqual({ status: "cancelled" });
  });

  it("rejects an invalid save response", async () => {
    invokeMock.mockResolvedValue({ status: "saved", documentId: "not-a-uuid" });

    await expect(saveDocument(envelope())).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it("preserves typed registry failures", async () => {
    const error = { code: "registry", cause: { code: "registry_unavailable" } };
    invokeMock.mockRejectedValue(error);

    await expect(saveDocument(envelope())).resolves.toEqual({
      status: "error",
      error: { type: "command", error },
    });
  });

  it("preserves source-path ownership failures", async () => {
    const error = { code: "registry", cause: { code: "source_path_in_use" } };
    invokeMock.mockRejectedValue(error);

    await expect(saveDocument(envelope())).resolves.toEqual({
      status: "error",
      error: { type: "command", error },
    });
  });

  it("classifies unknown failures without leaking details", async () => {
    invokeMock.mockRejectedValue(new Error("private filesystem detail"));

    await expect(saveDocument(envelope())).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});

function envelope(): DocumentEnvelopeSnapshot {
  return {
    schema_version: 1,
    document_id: DOCUMENT_ID,
    title: "Saved document",
    document: { type: "doc", content: [] },
  };
}
