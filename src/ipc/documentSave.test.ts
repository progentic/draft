import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { saveDocument, saveDocumentAs } from "./documentSave";
import type { DocumentEnvelopeSnapshot } from "./documentEnvelope";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";
const DISPLAY_NAME = "Research notes.draft";
const ORIGIN = "opened_draft" as const;

describe("document save IPC", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("sends a path-free authoritative Save request", async () => {
    invokeMock.mockResolvedValue(draftSaved(false));

    await expect(saveDocument(envelope(), DISPLAY_NAME, ORIGIN)).resolves.toEqual(
      draftSaved(false),
    );
    expect(invokeMock).toHaveBeenCalledWith("save_document", {
      request: {
        displayName: DISPLAY_NAME,
        mode: "save",
        origin: ORIGIN,
        snapshot: envelope(),
      },
    });
  });

  it.each(["draft", "docx", "txt"] as const)(
    "sends an explicit %s Save As format without a path",
    async (format) => {
      invokeMock.mockResolvedValue(
        format === "draft" ? draftSaved(true) : converted(format),
      );

      await saveDocumentAs(envelope(), DISPLAY_NAME, ORIGIN, format);

      expect(invokeMock).toHaveBeenCalledWith("save_document", {
        request: {
          displayName: DISPLAY_NAME,
          format,
          mode: "save_as",
          origin: ORIGIN,
          snapshot: envelope(),
        },
      });
      expect(JSON.stringify(invokeMock.mock.calls)).not.toContain("/private/");
    },
  );

  it("preserves converted-output identity semantics", async () => {
    invokeMock.mockResolvedValue(converted("docx"));

    await expect(
      saveDocumentAs(envelope(), DISPLAY_NAME, ORIGIN, "docx"),
    ).resolves.toEqual(converted("docx"));
  });

  it("preserves cancellation", async () => {
    invokeMock.mockResolvedValue({ status: "cancelled" });
    await expect(
      saveDocumentAs(envelope(), DISPLAY_NAME, ORIGIN, "txt"),
    ).resolves.toEqual({ status: "cancelled" });
  });

  it.each([
    { ...draftSaved(true), documentId: "not-a-uuid" },
    { ...draftSaved(true), displayName: "/private/research.draft" },
    { ...draftSaved(true), authoritativeIdentityChanged: false },
    { ...converted("docx"), outputFormat: "pdf" },
    { ...converted("txt"), dirtyStateChanged: true },
    { ...converted("txt"), path: "/private/output.txt" },
  ])("rejects malformed or path-bearing responses", async (response) => {
    invokeMock.mockResolvedValue(response);
    await expect(
      saveDocumentAs(envelope(), DISPLAY_NAME, ORIGIN, "txt"),
    ).resolves.toEqual({ status: "error", error: { type: "invalid-response" } });
  });

  it.each([
    { code: "registry", cause: { code: "registry_unavailable" } },
    { code: "invalid_target" },
    { code: "save_as_target", cause: { code: "extension_mismatch" } },
    { code: "write_failed", cause: { code: "replace_target" } },
    { code: "durability_uncertain" },
    { code: "docx_conversion", cause: { code: "unsupported_document_content", path: { indexes: [0] } } },
    { code: "plain_text_conversion", cause: { code: "output_too_large" } },
  ])("preserves typed command failure $code", async (error) => {
    invokeMock.mockRejectedValue(error);
    await expect(
      saveDocumentAs(envelope(), DISPLAY_NAME, ORIGIN, "docx"),
    ).resolves.toEqual({ status: "error", error: { type: "command", error } });
  });

  it("classifies malformed and unknown failures without leaking details", async () => {
    invokeMock.mockRejectedValue(new Error("private filesystem detail"));
    await expect(saveDocument(envelope(), DISPLAY_NAME, ORIGIN)).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});

function draftSaved(wasSaveAs: boolean) {
  return {
    status: "draft_saved",
    documentId: DOCUMENT_ID,
    displayName: DISPLAY_NAME,
    wasSaveAs,
    authoritativeIdentityChanged: wasSaveAs,
    dirtyStateCleared: true,
  } as const;
}

function converted(format: "docx" | "txt") {
  return {
    status: "converted_output",
    displayName: `Research notes.${format}`,
    outputFormat: format,
    bytesWritten: format === "docx" ? 42 : 0,
    authoritativeIdentityChanged: false,
    dirtyStateChanged: false,
  } as const;
}

function envelope(): DocumentEnvelopeSnapshot {
  return {
    schema_version: 2,
    document_id: DOCUMENT_ID,
    title: "Saved document",
    document: { type: "doc", content: [] },
  };
}
