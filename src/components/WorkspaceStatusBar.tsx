import { Activity, FileCheck2 } from "lucide-react";

import { ConnectivityModeControl } from "../features/connectivity/ConnectivityModeControl";
import type { ConnectivityModeState } from "../features/connectivity/useConnectivityMode";
import type { DocumentOperation } from "../features/document-session/useDocumentSession";
import type { ConnectivityMode } from "../ipc/connectivityMode";

interface WorkspaceStatusBarProps {
  connectivityState: ConnectivityModeState;
  documentStatus: string;
  exportPending: boolean;
  feedback: string;
  operation: DocumentOperation;
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
      <div
        className="workspace-status-bar__feedback"
        role="status"
        aria-live="polite"
        aria-atomic="true"
      >
        {props.feedback}
      </div>
      <ConnectivityModeControl
        state={props.connectivityState}
        onRefresh={props.onRefreshConnectivity}
        onSetMode={props.onSetConnectivityMode}
      />
    </footer>
  );
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
  return "Closing";
}
