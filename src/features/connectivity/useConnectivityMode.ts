import { useCallback, useEffect, useRef, useState } from "react";

import {
  getConnectivityMode,
  type ConnectivityMode,
  type ConnectivityModeClientError,
} from "../../ipc/connectivityMode";
import { setConnectivityMode as requestConnectivityMode } from "../../ipc/connectivityModeSet";

export type ConnectivityModeState =
  | { phase: "checking" }
  | { phase: "ready"; mode: ConnectivityMode }
  | { phase: "changing"; mode: ConnectivityMode }
  | { phase: "failed"; mode?: ConnectivityMode; error: ConnectivityModeClientError };

export function useConnectivityMode() {
  const [state, setState] = useState<ConnectivityModeState>({ phase: "checking" });
  const requestRef = useRef(0);

  const refresh = useCallback(async () => {
    const request = ++requestRef.current;
    setState((current) => checkingState(current));
    const result = await getConnectivityMode();
    if (requestRef.current !== request) {
      return;
    }
    setState(
      result.status === "ready"
        ? { phase: "ready", mode: result.mode }
        : (current) => failedState(current, result.error),
    );
  }, []);

  useEffect(() => {
    void refresh();
    return () => { requestRef.current += 1; };
  }, [refresh]);

  const setMode = useCallback(async (mode: ConnectivityMode) => {
    const currentMode = modeFromState(state);
    if (currentMode === undefined || state.phase === "changing") {
      return;
    }
    const request = ++requestRef.current;
    setState({ phase: "changing", mode: currentMode });
    const result = await requestConnectivityMode(mode);
    if (requestRef.current !== request) {
      return;
    }
    setState(
      result.status === "ready"
        ? { phase: "ready", mode: result.mode }
        : { phase: "failed", mode: currentMode, error: result.error },
    );
  }, [state]);

  return { state, refresh, setMode };
}

function checkingState(state: ConnectivityModeState): ConnectivityModeState {
  const mode = modeFromState(state);
  return mode === undefined ? { phase: "checking" } : { phase: "changing", mode };
}

function failedState(
  state: ConnectivityModeState,
  error: ConnectivityModeClientError,
): ConnectivityModeState {
  const mode = modeFromState(state);
  return mode === undefined ? { phase: "failed", error } : { phase: "failed", mode, error };
}

function modeFromState(state: ConnectivityModeState) {
  return "mode" in state ? state.mode : undefined;
}
