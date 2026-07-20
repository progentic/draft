import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { openDocument } from "./documentOpen";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("openDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("returns a validated Rust-loaded envelope", async () => {
    invokeMock.mockResolvedValue({ status: "opened_draft", envelope: envelope() });

    await expect(openDocument()).resolves.toEqual({
      status: "opened_draft",
      envelope: envelope(),
    });
    expect(invokeMock).toHaveBeenCalledWith("open_document", { request: {} });
  });

  it("distinguishes an unsaved Rust-created text import", async () => {
    invokeMock.mockResolvedValue({
      status: "imported_text",
      envelope: envelope(),
      format: "markdown",
    });

    await expect(openDocument()).resolves.toEqual({
      status: "imported_text",
      envelope: envelope(),
      format: "markdown",
    });
  });

  it.each([undefined, "future_format", "md", null])(
    "rejects an imported text response with format %s",
    async (format) => {
      invokeMock.mockResolvedValue({
        status: "imported_text",
        envelope: envelope(),
        ...(format === undefined ? {} : { format }),
      });

      await expect(openDocument()).resolves.toEqual({
        status: "error",
        error: { type: "invalid-response" },
      });
    },
  );

  it("returns path-free DOCX fidelity from the Rust import boundary", async () => {
    invokeMock.mockResolvedValue({
      status: "imported_external",
      envelope: envelope(),
      external: externalSummary(),
    });

    await expect(openDocument()).resolves.toEqual({
      status: "imported_external",
      envelope: envelope(),
      external: externalSummary(),
    });
  });

  it.each([
    { ...externalSummary(), path: "/private/paper.docx" },
    { ...externalSummary(), sourceBytes: [1, 2, 3] },
    { ...externalSummary(), rawXml: "<w:p/>" },
    { ...externalSummary(), sameFormatSave: "future_disposition" },
    { ...externalSummary(), fidelity: { classification: "future" } },
  ])("rejects malformed or authority-bearing DOCX summaries", async (external) => {
    invokeMock.mockResolvedValue({
      status: "imported_external",
      envelope: envelope(),
      external,
    });

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it("preserves user cancellation", async () => {
    invokeMock.mockResolvedValue({ status: "cancelled" });

    await expect(openDocument()).resolves.toEqual({ status: "cancelled" });
  });

  it("rejects an invalid loaded envelope", async () => {
    invokeMock.mockResolvedValue({
      status: "opened_draft",
      envelope: { ...envelope(), schema_version: 3 },
    });

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it("preserves typed unsupported-version failures", async () => {
    const error = {
      code: "invalid_envelope",
      cause: { code: "unsupported_schema_version", found: 3 },
    };
    invokeMock.mockRejectedValue(error);

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "command", error },
    });
  });

  it.each(["unsupported_file_type", "invalid_text_encoding", "text_too_large"])(
    "preserves typed %s import failures",
    async (code) => {
      invokeMock.mockRejectedValue({ code });

      await expect(openDocument()).resolves.toEqual({
        status: "error",
        error: { type: "command", error: { code } },
      });
    },
  );

  it.each([
    {
      code: "external_import",
      cause: {
        code: "docx",
        cause: {
          code: "malformed_package",
          fidelity: { classification: "malformed_external_input" },
        },
      },
    },
    {
      code: "external_import",
      cause: {
        code: "docx",
        cause: {
          code: "unsafe_package",
          fidelity: { classification: "unsafe", reason: "archive_path" },
        },
      },
    },
    { code: "external_import", cause: { code: "package_too_large" } },
  ])("preserves a closed typed DOCX failure", async (error) => {
    invokeMock.mockRejectedValue(error);

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "command", error },
    });
  });

  it("rejects mismatched DOCX failure classifications", async () => {
    invokeMock.mockRejectedValue({
      code: "external_import",
      cause: {
        code: "docx",
        cause: {
          code: "unsafe_package",
          fidelity: { classification: "malformed_external_input" },
        },
      },
    });

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });

  it("classifies unknown failures without leaking details", async () => {
    invokeMock.mockRejectedValue(new Error("private path detail"));

    await expect(openDocument()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});

function envelope() {
  return {
    schema_version: 2,
    document_id: DOCUMENT_ID,
    title: "Opened document",
    document: { type: "doc", content: [] },
  };
}

function externalSummary() {
  return {
    format: "docx",
    displayName: "paper.docx",
    fidelity: { classification: "exact" },
    sameFormatSave: "no_changes",
  };
}
