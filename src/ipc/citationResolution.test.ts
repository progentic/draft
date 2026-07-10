import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { citationClientErrorFrom, resolveCitation } from "./citationResolution";

const ATTRS = {
  schema_version: 1 as const,
  citekey: "smith2025",
  render_style: "apa7" as const,
};

describe("resolveCitation", () => {
  beforeEach(() => invokeMock.mockReset());

  it("returns a validated resolved marker", async () => {
    invokeMock.mockResolvedValue({
      schemaVersion: 1,
      citekey: "smith2025",
      renderStyle: "apa7",
      displayMarker: "[@smith2025]",
    });

    await expect(resolveCitation(ATTRS)).resolves.toEqual({
      status: "resolved",
      citation: {
        schemaVersion: 1,
        citekey: "smith2025",
        renderStyle: "apa7",
        displayMarker: "[@smith2025]",
      },
    });
    expect(invokeMock).toHaveBeenCalledWith("resolve_citation", {
      request: { attrs: ATTRS },
    });
  });

  it.each([
    { ...ATTRS, displayMarker: "forged" },
    { schemaVersion: 1, citekey: "smith2025", renderStyle: "apa7" },
    { ...ATTRS, schemaVersion: 2, displayMarker: "[@smith2025]" },
  ])("rejects an invalid response", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(resolveCitation(ATTRS)).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each([
    { code: "reference_not_found" },
    { code: "invalid_citation", cause: { code: "missing_citekey" } },
    { code: "reference_store", cause: { code: "corrupt_reference" } },
  ])("preserves a typed command error", (error) => {
    expect(citationClientErrorFrom(error)).toEqual({
      type: "command",
      error,
    });
  });

  it("classifies unknown failures as transport errors", () => {
    expect(citationClientErrorFrom(new Error("private detail"))).toEqual({ type: "transport" });
  });
});
