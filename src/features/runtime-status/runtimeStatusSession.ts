import { getRuntimeStatus, type RuntimeStatusClientError } from "../../ipc/runtimeStatus";
import {
  listenToRuntimeStatus,
  type RuntimeStatusEventClientError,
  type StopRuntimeStatusListener,
} from "../../ipc/runtimeStatusEvents";

export type RuntimeUnavailableReason =
  | RuntimeStatusClientError["type"]
  | RuntimeStatusEventClientError["type"];

export type RuntimeConnectionState =
  | { phase: "checking" }
  | { phase: "ready"; version: string }
  | { phase: "unavailable"; reason: RuntimeUnavailableReason };

type RuntimeStateListener = (state: RuntimeConnectionState) => void;

const STOP_NOOP = () => {};

export async function startRuntimeStatusSession(
  onState: RuntimeStateListener,
): Promise<StopRuntimeStatusListener> {
  let stopListener: StopRuntimeStatusListener = STOP_NOOP;

  try {
    stopListener = await registerRuntimeStatusListener(onState);
    await requestRuntimeStatus(onState);
    return stopListener;
  } catch {
    stopListener();
    onState({ phase: "unavailable", reason: "transport" });
    return STOP_NOOP;
  }
}

function registerRuntimeStatusListener(onState: RuntimeStateListener) {
  return listenToRuntimeStatus(
    (event) => onState({ phase: "ready", version: event.version }),
    (error) => onState({ phase: "unavailable", reason: error.type }),
  );
}

async function requestRuntimeStatus(onState: RuntimeStateListener) {
  const result = await getRuntimeStatus();
  if (result.status === "error") {
    onState({ phase: "unavailable", reason: result.error.type });
  }
}
