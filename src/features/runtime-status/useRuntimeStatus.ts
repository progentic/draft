import { useEffect, useState } from "react";

import {
  startRuntimeStatusSession,
  type RuntimeConnectionState,
} from "./runtimeStatusSession";

export type { RuntimeConnectionState } from "./runtimeStatusSession";

const CHECKING_STATE: RuntimeConnectionState = { phase: "checking" };

export function useRuntimeStatus() {
  const [state, setState] = useState<RuntimeConnectionState>(CHECKING_STATE);

  useEffect(() => attachRuntimeStatusSession(setState), []);

  return state;
}

function attachRuntimeStatusSession(onState: (state: RuntimeConnectionState) => void) {
  let isActive = true;
  let stopSession: (() => void) | undefined;

  void startRuntimeStatusSession((state) => {
    if (isActive) {
      onState(state);
    }
  }).then((stop) => {
    if (isActive) {
      stopSession = stop;
    } else {
      stop();
    }
  });

  return () => {
    isActive = false;
    stopSession?.();
  };
}
