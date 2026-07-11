import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export type ConnectivityMode = "online" | "offline";

export interface ConnectivityModeResponse {
  mode: ConnectivityMode;
}

export type ConnectivityModeClientError =
  | { type: "command"; code: "connectivity_unavailable" }
  | { type: "invalid-response" }
  | { type: "transport" };

export type ConnectivityModeResult =
  | { status: "ready"; mode: ConnectivityMode }
  | { status: "error"; error: ConnectivityModeClientError };

const COMMAND_NAME = "get_connectivity_mode";

/** Reads the effective Rust-owned connectivity mode. */
export async function getConnectivityMode(): Promise<ConnectivityModeResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: {} });
    return connectivityResultFrom(response);
  } catch (error: unknown) {
    return { status: "error", error: connectivityClientErrorFrom(error) };
  }
}

export function connectivityResultFrom(response: unknown): ConnectivityModeResult {
  return isConnectivityModeResponse(response)
    ? { status: "ready", mode: response.mode }
    : { status: "error", error: { type: "invalid-response" } };
}

export function connectivityClientErrorFrom(error: unknown): ConnectivityModeClientError {
  return isRecord(error) && hasExactFields(error, ["code"]) &&
    error.code === "connectivity_unavailable"
    ? { type: "command", code: error.code }
    : { type: "transport" };
}

function isConnectivityModeResponse(value: unknown): value is ConnectivityModeResponse {
  return (
    isRecord(value) &&
    hasExactFields(value, ["mode"]) &&
    (value.mode === "online" || value.mode === "offline")
  );
}

function hasExactFields(value: Record<string, unknown>, fields: string[]) {
  const keys = Object.keys(value);
  return keys.length === fields.length && fields.every((field) => keys.includes(field));
}
