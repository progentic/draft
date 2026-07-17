import { Activity, FileCheck2 } from "lucide-react";

import { ConnectivityModeControl } from "../features/connectivity/ConnectivityModeControl";
import type { ConnectivityModeState } from "../features/connectivity/useConnectivityMode";
import type { DocumentOperation } from "../features/document-session/useDocumentSession";
import { runtimeFailureMessage } from "../features/error-ux/errorPresentation";
import type { RuntimeConnectionState } from "../features/runtime-status/useRuntimeStatus";
import type { ConnectivityMode } from "../ipc/connectivityMode";

interface WorkspaceStatusBarProps {
  connectivityState: ConnectivityModeState;
  documentStatus: string;
  exportPending: boolean;
  operation: DocumentOperation;
  runtimeStatus: RuntimeConnectionState;
  onRefreshConnectivity: () => void;
  onSetConnectivityMode: (mode: ConnectivityMode) => void;
}

export function WorkspaceStatusBar(props: WorkspaceStatusBarProps) {
  const operation = operationLabel(props.operation, props.exportPending);
  const showOperation = operation !== props.documentStatus;
  return (
    <footer className="workspace-status-bar" aria-label="Workspace status">
      <StatusItem icon={FileCheck2} label={props.documentStatus} name="Document state" />
      {showOperation ? <OperationStatus label={operation} /> : null}
      <span className="workspace-status-bar__spacer" aria-hidden="true" />
      <ConnectivityModeControl
        state={props.connectivityState}
        onRefresh={props.onRefreshConnectivity}
        onSetMode={props.onSetConnectivityMode}
      />
      <RuntimeIdentity status={props.runtimeStatus} />
    </footer>
  );
}

function RuntimeIdentity(props: { status: RuntimeConnectionState }) {
  const label = runtimeIdentityLabel(props.status);
  return (
    <div
      className="workspace-status-bar__build"
      role="status"
      aria-label="Application build"
      aria-live="polite"
      aria-atomic="true"
      title={label}
    >
      {label}
    </div>
  );
}

function runtimeIdentityLabel(status: RuntimeConnectionState) {
  if (status.phase === "checking") return "Checking build";
  if (status.phase === "unavailable") return runtimeFailureMessage(status.reason);
  const commit = status.buildCommit === "development"
    ? status.buildCommit
    : status.buildCommit.slice(0, 8);
  return `v${status.version} · ${commit}`;
}

function OperationStatus(props: { label: string }) {
  return (
    <>
      <span className="workspace-status-bar__separator" aria-hidden="true" />
      <StatusItem icon={Activity} label={props.label} name="Background operation" />
    </>
  );
}

function StatusItem(props: {
  icon: typeof Activity;
  label: string;
  name: string;
}) {
  const Icon = props.icon;
  return (
    <div className="workspace-status-bar__item" aria-label={props.name}>
      <Icon aria-hidden="true" size={13} strokeWidth={1.9} />
      <span>{props.label}</span>
    </div>
  );
}

function operationLabel(operation: DocumentOperation, exportPending: boolean) {
  if (exportPending) return "Exporting";
  if (operation === "ready") return "Ready";
  if (operation === "creating") return "Creating";
  if (operation === "opening") return "Opening";
  if (operation === "saving") return "Saving";
  if (operation === "checking_source") return "Checking source";
  if (operation === "confirming_source_save") return "Waiting for confirmation";
  if (operation === "saving_source") return "Saving to source";
  return "Closing";
}
