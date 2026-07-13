import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const saveExternalDocumentMock = vi.hoisted(() => vi.fn());

vi.mock("../../ipc/externalDocumentSave", () => ({
  saveExternalDocument: saveExternalDocumentMock,
}));

import type { ExternalDocumentSummary } from "../../ipc/externalDocument";
import { useExternalSourceSave } from "./useExternalSourceSave";

const DOCUMENT_ID = "00000000-0000-4000-8000-000000000001";

describe("external source save workflow", () => {
  beforeEach(() => saveExternalDocumentMock.mockReset());

  it("inspects and confirms exact replacement without exposing a path", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_exact"))
      .mockResolvedValueOnce(saved("allowed_exact"));
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation).toEqual({
      displayName: "paper.docx",
      disposition: "allowed_exact",
    }));
    act(() => result.current.resolve("confirm"));

    await waitFor(() => expect(options.onSaved).toHaveBeenCalledWith(
      DOCUMENT_ID,
      "paper.docx",
    ));
    expect(saveExternalDocumentMock).toHaveBeenNthCalledWith(1, snapshot(), "inspect");
    expect(saveExternalDocumentMock).toHaveBeenNthCalledWith(2, snapshot(), "save_exact");
    expect(JSON.stringify(saveExternalDocumentMock.mock.calls)).not.toMatch(/path|xml/i);
  });

  it("requires explicit acceptance for normalized replacement", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_after_accepted_normalization"))
      .mockResolvedValueOnce(saved("allowed_after_accepted_normalization"));
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation?.disposition).toBe(
      "allowed_after_accepted_normalization",
    ));
    act(() => result.current.resolve("confirm"));

    await waitFor(() => expect(saveExternalDocumentMock).toHaveBeenLastCalledWith(
      snapshot(),
      "accept_normalization",
    ));
  });

  it("cancels through the typed boundary without accepting saved state", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_exact"))
      .mockResolvedValueOnce({ status: "cancelled", documentId: DOCUMENT_ID });
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation).not.toBeNull());
    act(() => result.current.resolve("cancel"));

    await waitFor(() => expect(options.onFeedback).toHaveBeenLastCalledWith(
      "Save Back cancelled. The source was not changed.",
    ));
    expect(saveExternalDocumentMock).toHaveBeenLastCalledWith(snapshot(), "cancel");
    expect(options.onSaved).not.toHaveBeenCalled();
  });

  it("does not claim cancellation when Rust returns an unexpected outcome", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_exact"))
      .mockResolvedValueOnce(saved("allowed_exact"));
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation).not.toBeNull());
    act(() => result.current.resolve("cancel"));

    await waitFor(() => expect(options.onFeedback).toHaveBeenLastCalledWith(
      "DRAFT could not confirm source safety. Save as a DRAFT document instead.",
    ));
    expect(options.onFeedback).not.toHaveBeenCalledWith(
      "Save Back cancelled. The source was not changed.",
    );
    expect(options.onSaved).not.toHaveBeenCalled();
  });

  it("blocks stale external sources with bounded recovery", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock.mockResolvedValue(
      eligibility("denied_source_changed", "reopen_source"),
    );
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());

    await waitFor(() => expect(result.current.enabled).toBe(false));
    expect(result.current.unavailableReason).toBe(
      "The source changed outside DRAFT. Reopen it before saving back.",
    );
    expect(result.current.unavailableReason).not.toMatch(/[/\\]|xml|fingerprint/i);
    expect(options.onSaved).not.toHaveBeenCalled();
  });

  it("preserves modified state and offers bounded retry after a safe write failure", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_exact"))
      .mockResolvedValueOnce({
        status: "error",
        error: {
          type: "command",
          error: { code: "write_failed", cause: "replace_target" },
        },
        recovery: "retry",
      });
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation).not.toBeNull());
    act(() => result.current.resolve("confirm"));

    await waitFor(() => expect(options.onFeedback).toHaveBeenLastCalledWith(
      "DRAFT could not replace the source. The original was preserved. Try again.",
    ));
    expect(result.current.enabled).toBe(true);
    expect(options.onSaved).not.toHaveBeenCalled();
  });

  it("blocks retry when the source final state is uncertain", async () => {
    const options = workflowOptions();
    saveExternalDocumentMock
      .mockResolvedValueOnce(eligibility("allowed_exact"))
      .mockResolvedValueOnce({
        status: "error",
        error: { type: "transport" },
        recovery: "reopen_source",
      });
    const { result } = renderHook(() => useExternalSourceSave(options));

    act(() => result.current.request());
    await waitFor(() => expect(result.current.confirmation).not.toBeNull());
    act(() => result.current.resolve("confirm"));

    await waitFor(() => expect(result.current.unavailableReason).toBe(
      "DRAFT could not confirm the source’s final state. Reopen it before continuing.",
    ));
    expect(result.current.enabled).toBe(false);
    expect(options.onSaved).not.toHaveBeenCalled();
  });

  it("rejects an eligibility result made stale by a newer editor revision", async () => {
    const pending = deferred<ReturnType<typeof eligibility>>();
    const initial = workflowOptions();
    saveExternalDocumentMock.mockReturnValue(pending.promise);
    const { result, rerender } = renderHook(
      ({ options }) => useExternalSourceSave(options),
      { initialProps: { options: initial } },
    );

    act(() => result.current.request());
    rerender({ options: { ...initial, revision: 2 } });
    pending.resolve(eligibility("allowed_exact"));

    await waitFor(() => expect(initial.onFeedback).toHaveBeenLastCalledWith(
      "The document changed. Choose Save Back to Source again.",
    ));
    expect(result.current.confirmation).toBeNull();
    expect(initial.onSaved).not.toHaveBeenCalled();
  });

  it("keeps unsupported source behavior visibly unavailable", () => {
    const options = workflowOptions({
      external: {
        ...externalSummary(),
        fidelity: {
          classification: "unsupported_preservable",
          features: ["paragraph_border"],
        },
        sameFormatSave: "denied_unsupported_source_behavior",
      },
    });
    const { result } = renderHook(() => useExternalSourceSave(options));

    expect(result.current.visible).toBe(true);
    expect(result.current.enabled).toBe(false);
    expect(result.current.unavailableReason).toContain("cannot replace safely");
    act(() => result.current.request());
    expect(saveExternalDocumentMock).not.toHaveBeenCalled();
  });
});

function workflowOptions(
  patch: Partial<Parameters<typeof useExternalSourceSave>[0]> = {},
) {
  return {
    external: externalSummary(),
    modified: true,
    operation: "ready",
    revision: 1,
    snapshot,
    onFeedback: vi.fn(),
    onOperation: vi.fn(),
    onSaved: vi.fn(),
    ...patch,
  };
}

function externalSummary(): ExternalDocumentSummary {
  return {
    format: "docx",
    displayName: "paper.docx",
    fidelity: { classification: "exact" },
    sameFormatSave: "no_changes",
  };
}

function snapshot() {
  return {
    schema_version: 2 as const,
    document_id: DOCUMENT_ID,
    title: "Paper",
    document: {
      type: "doc" as const,
      content: [{ type: "paragraph", content: [{ type: "text", text: "Edited" }] }],
    },
  };
}

function eligibility(
  disposition:
    | "allowed_exact"
    | "allowed_after_accepted_normalization"
    | "denied_source_changed",
  recovery: "none" | "confirm_normalization" | "reopen_source" = "none",
) {
  return {
    status: "eligibility" as const,
    documentId: DOCUMENT_ID,
    displayName: "paper.docx",
    disposition,
    recovery,
  };
}

function saved(
  disposition: "allowed_exact" | "allowed_after_accepted_normalization",
) {
  return {
    status: "saved" as const,
    documentId: DOCUMENT_ID,
    displayName: "paper.docx",
    bytesWritten: 1024,
    disposition,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolver) => {
    resolve = resolver;
  });
  return { promise, resolve };
}
