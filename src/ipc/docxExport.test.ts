import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { exportDocument } from "./docxExport";
import type { DocumentEnvelopeSnapshot } from "./documentEnvelope";

const SNAPSHOT: DocumentEnvelopeSnapshot = {
  schema_version: 2,
  document_id: "00000000-0000-4000-8000-000000000001",
  title: "Exported document",
  document: { type: "doc", content: [] },
};

describe("exportDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokes the Rust-owned export and validates completion", async () => {
    invokeMock.mockResolvedValue({ status: "exported", bytesWritten: 2048 });

    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "exported",
      bytesWritten: 2048,
    });
    expect(invokeMock).toHaveBeenCalledWith("export_document", {
      request: { snapshot: SNAPSHOT },
    });
  });

  it("preserves cancellation", async () => {
    invokeMock.mockResolvedValue({ status: "cancelled" });
    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({ status: "cancelled" });
  });

  it.each([
    "artifact_too_large",
    "durability_uncertain",
    "invalid_document_structure",
    "invalid_target",
    "nesting_too_deep",
    "package_construction_failed",
    "source_too_large",
    "too_many_nodes",
    "unsupported_citation",
    "unsupported_document_content",
    "write_failed",
  ])("preserves the %s export cause", async (cause) => {
    invokeMock.mockRejectedValue({ code: "export", cause: { code: cause } });

    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "export", cause },
    });
  });

  it("preserves bounded command failures without their details", async () => {
    invokeMock.mockRejectedValue({
      code: "invalid_envelope",
      cause: { code: "unsupported_schema_version", found: 3 },
    });
    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "invalid_envelope" },
    });

    invokeMock.mockRejectedValue({ code: "unsupported_file_location" });
    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "error",
      error: { type: "command", code: "unsupported_file_location" },
    });
  });

  it("rejects malformed responses and sanitizes unknown failures", async () => {
    invokeMock.mockResolvedValue({ status: "exported", bytesWritten: 0 });
    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });

    invokeMock.mockRejectedValue({ code: "future_error", path: "/private" });
    await expect(exportDocument(SNAPSHOT)).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
