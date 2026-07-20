import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import type { DocumentEnvelopeSnapshot } from "./documentEnvelope";
import { saveExternalDocument } from "./externalDocumentSave";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("saveExternalDocument", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it.each(["inspect", "save_exact", "accept_normalization", "cancel"] as const)(
    "sends the path-free %s decision through the typed command",
    async (decision) => {
      invokeMock.mockResolvedValue({
        status: "cancelled",
        documentId: DOCUMENT_ID,
      });

      await saveExternalDocument(envelope(), decision);

      expect(invokeMock).toHaveBeenCalledWith("save_external_document", {
        request: { snapshot: envelope(), decision },
      });
      expect(JSON.stringify(invokeMock.mock.calls)).not.toContain("path");
      expect(JSON.stringify(invokeMock.mock.calls)).not.toContain(
        "fingerprint",
      );
    },
  );

  it.each([
    ["no_changes", [], "none"],
    ["allowed_exact", [], "none"],
    [
      "allowed_after_accepted_normalization",
      ["alternate_heading_style_name"],
      "confirm_normalization",
    ],
    ["denied_unsupported_source_behavior", [], "save_as_draft"],
    ["denied_source_changed", [], "reopen_source"],
  ] as const)(
    "maps %s eligibility with typed normalization data",
    async (disposition, normalizations, recovery) => {
      invokeMock.mockResolvedValue({
        status: "eligibility",
        documentId: DOCUMENT_ID,
        displayName: "paper.docx",
        disposition,
        normalizations,
      });

      await expect(saveExternalDocument(envelope(), "inspect")).resolves.toEqual({
        status: "eligibility",
        documentId: DOCUMENT_ID,
        displayName: "paper.docx",
        disposition,
        normalizations,
        recovery,
      });
    },
  );

  it.each([
    ["allowed_exact", ["alternate_heading_style_name"]],
    ["allowed_after_accepted_normalization", []],
    ["allowed_after_accepted_normalization", ["future_normalization"]],
  ])("rejects a %s eligibility response with unmapped normalization data", async (
    disposition,
    normalizations,
  ) => {
    invokeMock.mockResolvedValue({
      status: "eligibility",
      documentId: DOCUMENT_ID,
      displayName: "paper.docx",
      disposition,
      normalizations,
    });

    await expect(saveExternalDocument(envelope(), "inspect")).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
      recovery: "reopen_source",
    });
  });

  it.each([
    ["allowed_exact", "save_exact"],
    ["allowed_after_accepted_normalization", "accept_normalization"],
  ] as const)(
    "accepts a %s completed replacement",
    async (disposition, decision) => {
      invokeMock.mockResolvedValue({
        status: "saved",
        documentId: DOCUMENT_ID,
        displayName: "paper.docx",
        bytesWritten: 2048,
        disposition,
      });

      await expect(saveExternalDocument(envelope(), decision)).resolves.toEqual(
        {
          status: "saved",
          documentId: DOCUMENT_ID,
          displayName: "paper.docx",
          bytesWritten: 2048,
          disposition,
        },
      );
    },
  );

  it("preserves unchanged source identity without claiming a write", async () => {
    invokeMock.mockResolvedValue({
      status: "unchanged",
      documentId: DOCUMENT_ID,
      displayName: "paper.docx",
    });

    await expect(
      saveExternalDocument(envelope(), "save_exact"),
    ).resolves.toEqual({
      status: "unchanged",
      documentId: DOCUMENT_ID,
      displayName: "paper.docx",
    });
  });

  it("requires explicit acceptance for known normalization", async () => {
    invokeMock.mockResolvedValue({
      status: "confirmation_required",
      documentId: DOCUMENT_ID,
      disposition: "allowed_after_accepted_normalization",
    });

    await expect(
      saveExternalDocument(envelope(), "save_exact"),
    ).resolves.toEqual({
      status: "confirmation_required",
      documentId: DOCUMENT_ID,
      disposition: "allowed_after_accepted_normalization",
      recovery: "confirm_normalization",
    });
  });

  it.each([
    ["denied_unsupported_source_behavior", "save_as_draft"],
    ["denied_read_only", "save_as_draft"],
    ["denied_missing_provenance", "save_as_draft"],
    ["denied_fidelity_unknown", "save_as_draft"],
    ["denied_writer_unavailable", "save_as_draft"],
    ["denied_source_missing", "reopen_source"],
    ["denied_source_changed", "reopen_source"],
  ] as const)(
    "maps %s to bounded %s recovery",
    async (disposition, recovery) => {
      invokeMock.mockResolvedValue({
        status: "denied",
        documentId: DOCUMENT_ID,
        disposition,
      });

      await expect(
        saveExternalDocument(envelope(), "save_exact"),
      ).resolves.toEqual({
        status: "denied",
        documentId: DOCUMENT_ID,
        disposition,
        recovery,
      });
    },
  );

  it("preserves cancellation without a recovery claim", async () => {
    invokeMock.mockResolvedValue({
      status: "cancelled",
      documentId: DOCUMENT_ID,
    });

    await expect(saveExternalDocument(envelope(), "cancel")).resolves.toEqual({
      status: "cancelled",
      documentId: DOCUMENT_ID,
    });
  });

  it.each([
    [invalidEnvelopeError(), "none"],
    [registryError("not_open"), "reopen_source"],
    [registryError("external_source_unavailable"), "reopen_source"],
    [registryError("registry_unavailable"), "retry"],
    [registryError("already_open"), "none"],
    [registryError("source_path_in_use"), "none"],
    [{ code: "source_read", cause: { code: "read_failed" } }, "retry"],
    [
      { code: "compilation", cause: { code: "unsupported_document_content" } },
      "save_as_draft",
    ],
    [{ code: "write_failed", cause: { code: "replace_target" } }, "retry"],
    [
      {
        code: "replacement_rolled_back",
        cause: { code: "durability_uncertain" },
      },
      "retry",
    ],
    [
      {
        code: "replacement_rolled_back",
        cause: { code: "registry", cause: { code: "registry_unavailable" } },
      },
      "retry",
    ],
    [
      {
        code: "rollback_failed",
        cause: { code: "durability_uncertain" },
        rollback: { code: "replace_target" },
      },
      "reopen_source",
    ],
  ] as const)(
    "preserves a typed failure with %s recovery",
    async (error, recovery) => {
      invokeMock.mockRejectedValue(error);

      const result = await saveExternalDocument(envelope(), "save_exact");

      expect(result).toMatchObject({ status: "error", recovery });
      expect(result).not.toHaveProperty("error.error.path");
      expect(result).not.toHaveProperty("error.error.fingerprint");
    },
  );

  it.each([
    {
      status: "eligibility",
      documentId: DOCUMENT_ID,
      displayName: "/private/paper.docx",
      disposition: "allowed_exact",
      normalizations: [],
    },
    {
      status: "eligibility",
      documentId: DOCUMENT_ID,
      displayName: "paper.docx",
      disposition: "future_disposition",
      normalizations: [],
    },
    {
      status: "saved",
      documentId: DOCUMENT_ID,
      displayName: "/private/paper.docx",
      bytesWritten: 4,
      disposition: "allowed_exact",
    },
    {
      status: "saved",
      documentId: DOCUMENT_ID,
      displayName: "paper.docx",
      bytesWritten: 0,
      disposition: "allowed_exact",
    },
    {
      status: "confirmation_required",
      documentId: DOCUMENT_ID,
      disposition: "allowed_exact",
    },
    { status: "denied", documentId: DOCUMENT_ID, disposition: "allowed_exact" },
    {
      status: "cancelled",
      documentId: DOCUMENT_ID,
      path: "/private/paper.docx",
    },
  ])("rejects malformed or path-bearing response %#", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(
      saveExternalDocument(envelope(), "save_exact"),
    ).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
      recovery: "reopen_source",
    });
  });

  it.each([
    {
      code: "source_read",
      cause: { code: "read_failed", path: "/private/paper.docx" },
    },
    { code: "write_failed", cause: { code: "unknown_stage" } },
    { code: "rollback_failed", cause: { code: "durability_uncertain" } },
    { code: "future_external_error" },
  ])("rejects malformed command failure %#", async (error) => {
    invokeMock.mockRejectedValue(error);

    await expect(
      saveExternalDocument(envelope(), "save_exact"),
    ).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
      recovery: "reopen_source",
    });
  });

  it("classifies non-command rejection as transport failure", async () => {
    invokeMock.mockRejectedValue(new Error("private detail"));

    await expect(
      saveExternalDocument(envelope(), "save_exact"),
    ).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
      recovery: "reopen_source",
    });
  });
});

function envelope(): DocumentEnvelopeSnapshot {
  return {
    schema_version: 2,
    document_id: DOCUMENT_ID,
    title: "Imported document",
    document: { type: "doc", content: [] },
  };
}

function invalidEnvelopeError() {
  return {
    code: "invalid_envelope",
    cause: { code: "invalid_document_root" },
  } as const;
}

function registryError(
  code:
    | "already_open"
    | "not_open"
    | "external_source_unavailable"
    | "source_path_in_use"
    | "registry_unavailable",
) {
  return { code: "registry", cause: { code } } as const;
}
