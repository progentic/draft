import { Wifi, WifiOff } from "lucide-react";

import type { ConnectivityMode } from "../../ipc/connectivityMode";
import type { ConnectivityModeState } from "./useConnectivityMode";

interface ConnectivityModeControlProps {
  state: ConnectivityModeState;
  onRefresh: () => void;
  onSetMode: (mode: ConnectivityMode) => void;
}

export function ConnectivityModeControl(props: ConnectivityModeControlProps) {
  const view = connectivityView(props.state);
  const Icon = view.mode === "online" ? Wifi : WifiOff;

  return (
    <div className="connectivity-control">
      <button
        className="connectivity-toggle"
        type="button"
        aria-label={view.actionLabel}
        aria-pressed={view.mode === undefined ? undefined : view.mode === "offline"}
        disabled={props.state.phase === "checking" || props.state.phase === "changing"}
        title={view.actionLabel}
        onClick={() => runConnectivityAction(props, view.mode)}
      >
        <Icon aria-hidden="true" size={15} strokeWidth={1.9} />
        <span className="connectivity-toggle__label">{view.visibleLabel}</span>
      </button>
      {props.state.phase === "failed" ? (
        <span className="connectivity-feedback" role="alert">
          {view.failureLabel}
        </span>
      ) : null}
    </div>
  );
}

function runConnectivityAction(
  props: ConnectivityModeControlProps,
  mode: ConnectivityMode | undefined,
) {
  if (mode === undefined) {
    props.onRefresh();
    return;
  }
  props.onSetMode(mode === "online" ? "offline" : "online");
}

function connectivityView(state: ConnectivityModeState) {
  const mode = "mode" in state ? state.mode : undefined;
  if (state.phase === "checking") {
    return connectivityViewModel(undefined, "Loading mode", "Loading connectivity mode");
  }
  if (mode === undefined) {
    return connectivityViewModel(undefined, "Mode unavailable", "Retry connectivity status");
  }
  const modeLabel = mode === "online" ? "Online" : "Offline";
  return connectivityViewModel(
    mode,
    state.phase === "failed" ? `${modeLabel} - change failed` : modeLabel,
    mode === "online" ? "Work offline" : "Go online",
  );
}

function connectivityViewModel(
  mode: ConnectivityMode | undefined,
  visibleLabel: string,
  actionLabel: string,
) {
  return {
    mode,
    visibleLabel,
    actionLabel,
    failureLabel: mode === undefined
      ? "Connectivity mode unavailable."
      : `Could not change mode. DRAFT remains ${mode}.`,
  };
}
