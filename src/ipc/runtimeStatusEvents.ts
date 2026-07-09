import { listenToEvent, type StopEventListener } from "./eventClient";

export interface RuntimeStatusReadyEvent {
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
    Object.keys(value).length === 2 &&
    value.type === "ready" &&
    typeof value.version === "string" &&
    value.version.trim().length > 0
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
