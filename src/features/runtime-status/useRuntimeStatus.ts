import { useEffect, useState } from "react";

import {
  getRuntimeStatus,
  type RuntimeStatusClientError,
  type RuntimeStatusResult,
} from "../../ipc/runtimeStatus";

export type RuntimeConnectionState =
  | { phase: "checking" }
  | { phase: "ready"; version: string }
  | { phase: "unavailable"; reason: RuntimeStatusClientError["type"] };

const CHECKING_STATE: RuntimeConnectionState = { phase: "checking" };

export function useRuntimeStatus() {
  const [state, setState] = useState<RuntimeConnectionState>(CHECKING_STATE);

  useEffect(() => {
    let isActive = true;
    void getRuntimeStatus().then((result) => {
      if (isActive) {
        setState(connectionStateFrom(result));
      }
    });

    return () => {
      isActive = false;
    };
  }, []);

  return state;
}

function connectionStateFrom(result: RuntimeStatusResult): RuntimeConnectionState {
  if (result.status === "ready") {
    return { phase: "ready", version: result.version };
  }

  return { phase: "unavailable", reason: result.error.type };
}
