import { FileText, PanelLeftClose, PanelLeftOpen } from "lucide-react";

import { ConnectivityModeControl } from "../features/connectivity/ConnectivityModeControl";
import type { ConnectivityModeState } from "../features/connectivity/useConnectivityMode";
import type { ConnectivityMode } from "../ipc/connectivityMode";

interface WorkspaceHeaderProps {
  isOutlineOpen: boolean;
  connectivityState: ConnectivityModeState;
  onRefreshConnectivity: () => void;
  onSetConnectivityMode: (mode: ConnectivityMode) => void;
  onToggleOutline: () => void;
}

export function WorkspaceHeader(props: WorkspaceHeaderProps) {
  return (
    <header className="workspace-header">
      <HeaderBrand {...props} />
      <DocumentIdentity />
      <HeaderSession {...props} />
    </header>
  );
}

function HeaderBrand(props: WorkspaceHeaderProps) {
  const OutlineIcon = props.isOutlineOpen ? PanelLeftClose : PanelLeftOpen;
  const outlineLabel = props.isOutlineOpen ? "Close outline" : "Open outline";

  return (
    <div className="workspace-header__brand-group">
      <button
        className="icon-button icon-button--header outline-toggle"
        type="button"
        aria-controls="document-outline"
        aria-label={outlineLabel}
        aria-pressed={props.isOutlineOpen}
        title={outlineLabel}
        onClick={props.onToggleOutline}
      >
        <OutlineIcon aria-hidden="true" size={18} strokeWidth={1.8} />
      </button>
      <h1 className="wordmark">
        <span className="wordmark__mark" aria-hidden="true">D</span>
        <span className="wordmark__name">DRAFT</span>
      </h1>
    </div>
  );
}

function DocumentIdentity() {
  return (
    <div className="workspace-header__document">
      <FileText aria-hidden="true" size={16} strokeWidth={1.8} />
      <span>Untitled document</span>
    </div>
  );
}

function HeaderSession(props: WorkspaceHeaderProps) {
  return (
    <div className="workspace-header__session">
      <div className="session-status" role="status">
        <span className="session-status__dot" aria-hidden="true" />
        <span>Not saved</span>
      </div>
      <ConnectivityModeControl
        state={props.connectivityState}
        onRefresh={props.onRefreshConnectivity}
        onSetMode={props.onSetConnectivityMode}
      />
    </div>
  );
}
