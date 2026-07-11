import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export type DiagnosticContractName =
  | "citation_node"
  | "document_envelope"
  | "pdf_import_job_store"
  | "python_helper_protocol"
  | "reference_record"
  | "reference_store";

export type DiagnosticSubsystemName =
  | "core_runtime"
  | "document_registry"
  | "network_client"
  | "pdf_import_job_store"
  | "python_helper"
  | "reference_store";

export type DiagnosticAvailability = "ready" | "not_checked";

export interface DiagnosticSnapshot {
  schemaVersion: 1;
  applicationVersion: string;
  contractVersions: Array<{ name: DiagnosticContractName; version: number }>;
  subsystems: Array<{ name: DiagnosticSubsystemName; status: DiagnosticAvailability }>;
}

export type DiagnosticSnapshotClientError =
  | {
      type: "command";
      code:
        | "invalid_application_version"
        | "snapshot_serialization_failed"
        | "snapshot_too_large";
    }
  | { type: "invalid-response" }
  | { type: "transport" };

export type DiagnosticSnapshotResult =
  | { status: "ready"; snapshot: DiagnosticSnapshot }
  | { status: "error"; error: DiagnosticSnapshotClientError };

const COMMAND_NAME = "get_diagnostic_snapshot";
const CONTRACT_NAMES: DiagnosticContractName[] = [
  "citation_node",
  "document_envelope",
  "pdf_import_job_store",
  "python_helper_protocol",
  "reference_record",
  "reference_store",
];
const SUBSYSTEMS: Array<{
  name: DiagnosticSubsystemName;
  status: DiagnosticAvailability;
}> = [
  { name: "core_runtime", status: "ready" },
  { name: "document_registry", status: "ready" },
  { name: "network_client", status: "ready" },
  { name: "pdf_import_job_store", status: "ready" },
  { name: "python_helper", status: "not_checked" },
  { name: "reference_store", status: "ready" },
];

/** Requests bounded local support metadata from the Rust core. */
export async function getDiagnosticSnapshot(): Promise<DiagnosticSnapshotResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: {} });
    return isDiagnosticSnapshot(response)
      ? { status: "ready", snapshot: response }
      : { status: "error", error: { type: "invalid-response" } };
  } catch (error: unknown) {
    return { status: "error", error: diagnosticClientErrorFrom(error) };
  }
}

function diagnosticClientErrorFrom(error: unknown): DiagnosticSnapshotClientError {
  return isRecord(error) && hasExactFields(error, ["code"]) && isCommandCode(error.code)
    ? { type: "command", code: error.code }
    : { type: "transport" };
}

function isDiagnosticSnapshot(value: unknown): value is DiagnosticSnapshot {
  return (
    isRecord(value) &&
    hasExactFields(value, [
      "schemaVersion",
      "applicationVersion",
      "contractVersions",
      "subsystems",
    ]) &&
    value.schemaVersion === 1 &&
    isApplicationVersion(value.applicationVersion) &&
    isContractVersions(value.contractVersions) &&
    isSubsystems(value.subsystems)
  );
}

function isApplicationVersion(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= 64 &&
    /^[0-9A-Za-z.+-]+$/.test(value)
  );
}

function isContractVersions(value: unknown): value is DiagnosticSnapshot["contractVersions"] {
  return (
    Array.isArray(value) &&
    value.length === CONTRACT_NAMES.length &&
    value.every((entry, index) => isContractVersion(entry, CONTRACT_NAMES[index]))
  );
}

function isContractVersion(value: unknown, expectedName: DiagnosticContractName) {
  return (
    isRecord(value) &&
    hasExactFields(value, ["name", "version"]) &&
    value.name === expectedName &&
    Number.isSafeInteger(value.version) &&
    Number(value.version) > 0
  );
}

function isSubsystems(value: unknown): value is DiagnosticSnapshot["subsystems"] {
  return (
    Array.isArray(value) &&
    value.length === SUBSYSTEMS.length &&
    value.every((entry, index) => isSubsystem(entry, SUBSYSTEMS[index]))
  );
}

function isSubsystem(
  value: unknown,
  expected: { name: DiagnosticSubsystemName; status: DiagnosticAvailability },
) {
  return (
    isRecord(value) &&
    hasExactFields(value, ["name", "status"]) &&
    value.name === expected.name &&
    value.status === expected.status
  );
}

function isCommandCode(value: unknown): value is Extract<
  DiagnosticSnapshotClientError,
  { type: "command" }
>["code"] {
  return (
    value === "invalid_application_version" ||
    value === "snapshot_serialization_failed" ||
    value === "snapshot_too_large"
  );
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
