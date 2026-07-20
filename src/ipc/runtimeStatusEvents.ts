import { listenToEvent, type StopEventListener } from "./eventClient";
import { isRuntimeStatusResponse } from "./runtimeStatus";

export interface RuntimeStatusReadyEvent {
  buildCommit: string;
  buildProfile: "debug" | "release";
  type: "ready";
  version: string;
}

export type RuntimeStatusEventClientError = { type: "invalid-payload" };
export type StopRuntimeStatusListener = StopEventListener;

const EVENT_NAME = "draft://runtime-status";

/** Registers a validated listener for Rust-owned runtime status updates. */
export async function listenToRuntimeStatus(
  onEvent: (event: RuntimeStatusReadyEvent) => void,
  onError: (error: RuntimeStatusEventClientError) => void,
): Promise<StopRuntimeStatusListener> {
  return listenToEvent(EVENT_NAME, (payload) => deliverPayload(payload, onEvent, onError));
}

function deliverPayload(
  payload: unknown,
  onEvent: (event: RuntimeStatusReadyEvent) => void,
  onError: (error: RuntimeStatusEventClientError) => void,
) {
  if (!isRuntimeStatusReadyEvent(payload)) {
    onError({ type: "invalid-payload" });
    return;
  }

  onEvent(payload);
}

function isRuntimeStatusReadyEvent(value: unknown): value is RuntimeStatusReadyEvent {
  return (
    isRecord(value) &&
    Object.keys(value).length === 4 &&
    value.type === "ready" &&
    isRuntimeStatusResponse({
      buildCommit: value.buildCommit,
      buildProfile: value.buildProfile,
      version: value.version,
    })
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
