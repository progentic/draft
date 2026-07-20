import { FileText, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import draftIconUrl from "../../src-tauri/icons/32x32.png";

interface WorkspaceHeaderProps {
  isOutlineOpen: boolean;
  documentTitle: string;
  unsaved: boolean;
  onToggleOutline: () => void;
}

export function WorkspaceHeader(props: WorkspaceHeaderProps) {
  const OutlineIcon = props.isOutlineOpen ? PanelLeftClose : PanelLeftOpen;
  const outlineLabel = props.isOutlineOpen ? "Close outline" : "Open outline";

  return (
    <header className="workspace-header">
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
          <img className="wordmark__mark" src={draftIconUrl} alt="" aria-hidden="true" />
          <span className="wordmark__name">DRAFT</span>
        </h1>
      </div>
      <div className="workspace-header__document">
        <FileText aria-hidden="true" size={16} strokeWidth={1.8} />
        <span>{props.documentTitle}</span>
        {props.unsaved ? <span className="workspace-header__state">Unsaved</span> : null}
      </div>
    </header>
  );
}
