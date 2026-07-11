import { invokeCommand } from "./client";
import {
  connectivityClientErrorFrom,
  connectivityResultFrom,
  type ConnectivityMode,
  type ConnectivityModeResult,
} from "./connectivityMode";

const COMMAND_NAME = "set_connectivity_mode";

/** Replaces the effective Rust-owned connectivity mode for this session. */
export async function setConnectivityMode(
  mode: ConnectivityMode,
): Promise<ConnectivityModeResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: { mode } });
    const result = connectivityResultFrom(response);
    return result.status === "ready" && result.mode !== mode
      ? { status: "error", error: { type: "invalid-response" } }
      : result;
  } catch (error: unknown) {
    return { status: "error", error: connectivityClientErrorFrom(error) };
  }
}
