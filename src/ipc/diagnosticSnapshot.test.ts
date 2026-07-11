import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { getDiagnosticSnapshot } from "./diagnosticSnapshot";

const validSnapshot = {
  schemaVersion: 1,
  applicationVersion: "0.1.0",
  contractVersions: [
    { name: "citation_node", version: 1 },
    { name: "document_envelope", version: 1 },
    { name: "pdf_import_job_store", version: 1 },
    { name: "python_helper_protocol", version: 1 },
    { name: "reference_record", version: 1 },
    { name: "reference_store", version: 1 },
  ],
  subsystems: [
    { name: "core_runtime", status: "ready" },
    { name: "document_registry", status: "ready" },
    { name: "network_client", status: "ready" },
    { name: "pdf_import_job_store", status: "ready" },
    { name: "python_helper", status: "not_checked" },
    { name: "reference_store", status: "ready" },
  ],
} as const;

describe("getDiagnosticSnapshot", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("returns the strict snapshot from the registered command", async () => {
    invokeMock.mockResolvedValue(validSnapshot);

    await expect(getDiagnosticSnapshot()).resolves.toEqual({
      status: "ready",
      snapshot: validSnapshot,
    });
    expect(invokeMock).toHaveBeenCalledWith("get_diagnostic_snapshot", { request: {} });
  });

  it.each([
    { ...validSnapshot, unexpected: true },
    { ...validSnapshot, applicationVersion: "" },
    {
      ...validSnapshot,
      contractVersions: [...validSnapshot.contractVersions].reverse(),
    },
    {
      ...validSnapshot,
      subsystems: validSnapshot.subsystems.map((entry) =>
        entry.name === "python_helper" ? { ...entry, status: "ready" } : entry,
      ),
    },
  ])("rejects an invalid response without exposing it", async (response) => {
    invokeMock.mockResolvedValue(response);

    await expect(getDiagnosticSnapshot()).resolves.toEqual({
      status: "error",
      error: { type: "invalid-response" },
    });
  });

  it.each([
    "invalid_application_version",
    "snapshot_serialization_failed",
    "snapshot_too_large",
  ] as const)("preserves the %s command error", async (code) => {
    invokeMock.mockRejectedValue({ code });

    await expect(getDiagnosticSnapshot()).resolves.toEqual({
      status: "error",
      error: { type: "command", code },
    });
  });

  it("maps unknown failures to transport without raw details", async () => {
    invokeMock.mockRejectedValue(new Error("private runtime detail"));

    await expect(getDiagnosticSnapshot()).resolves.toEqual({
      status: "error",
      error: { type: "transport" },
    });
  });
});
