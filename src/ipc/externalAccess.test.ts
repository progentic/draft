import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { openExternalAccess, type OpenExternalAccessRequest } from "./externalAccess";

const REQUESTS: OpenExternalAccessRequest[] = [
  { destination: "publisher", url: "https://publisher.example/article" },
  { destination: "institutional", url: "https://library.example/item" },
  { destination: "doi", doi: "10.1000/example" },
  { destination: "google_scholar", query: "example research title" },
];

describe("openExternalAccess", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it.each(REQUESTS)("invokes the typed $destination handoff", async (request) => {
    invokeMock.mockResolvedValue({ status: "opened", destination: request.destination });

    await expect(openExternalAccess(request)).resolves.toEqual({
      status: "opened",
      destination: request.destination,
    });
    expect(invokeMock).toHaveBeenCalledWith("open_external_access", { request });
  });

  it.each([
    { status: "opened", destination: "unknown" },
    { status: "opened", destination: "doi" },
    { status: "opened", destination: "publisher", url: "https://private.example" },
    { status: "failed", destination: "publisher" },
  ])("rejects an invalid response", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(openExternalAccess(REQUESTS[0])).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each([
    "invalid_url",
    "invalid_doi",
    "invalid_search_query",
    "browser_unavailable",
  ] as const)("preserves the %s command error", async (code) => {
    invokeMock.mockRejectedValue({ code });

    await expect(openExternalAccess(REQUESTS[0])).resolves.toEqual({
      status: "error",
      error: { type: "command", code },
    });
  });

  it("classifies unknown failures without retaining browser details", async () => {
    invokeMock.mockRejectedValue(new Error("browser executable detail"));

    await expect(openExternalAccess(REQUESTS[0])).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
