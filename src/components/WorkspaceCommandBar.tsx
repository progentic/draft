import {
  BookOpen,
  Download,
  FilePlus2,
  FolderOpen,
  Save,
  ScanText,
  X,
} from "lucide-react";

import type { DocumentSession } from "../features/document-session/useDocumentSession";

interface WorkspaceCommandBarProps {
  activePanel: "references" | "text-review" | null;
  documentSession: DocumentSession;
  exportDisabled: boolean;
  feedback: string;
  exportLabel: string;
  onExport: () => void;
  onTogglePanel: (panel: "references" | "text-review") => void;
}

export function WorkspaceCommandBar(props: WorkspaceCommandBarProps) {
  const session = props.documentSession;
  const busy = session.operation !== "ready" || props.exportDisabled;

  return (
    <nav className="workspace-command-bar" aria-label="Document actions">
      <CommandButton icon={FilePlus2} label="New" disabled={busy} onClick={session.requestNew} />
      <CommandButton icon={FolderOpen} label="Open" disabled={busy} onClick={session.requestOpen} />
      <CommandButton icon={Save} label="Save" disabled={busy || !session.canSave} onClick={() => void session.save()} />
      <span className="workspace-command-bar__separator" aria-hidden="true" />
      <CommandButton
        icon={BookOpen}
        label="References"
        disabled={busy}
        controls="reference-library-panel"
        expanded={props.activePanel === "references"}
        onClick={() => props.onTogglePanel("references")}
      />
      <CommandButton
        icon={ScanText}
        label="Text checks"
        disabled={busy}
        controls="text-analysis-panel"
        expanded={props.activePanel === "text-review"}
        onClick={() => props.onTogglePanel("text-review")}
      />
      <span className="workspace-command-bar__separator" aria-hidden="true" />
      <CommandButton
        icon={Download}
        label={props.exportLabel}
        disabled={props.exportDisabled || !session.canExport}
        onClick={props.onExport}
      />
      <CommandButton icon={X} label="Close" disabled={busy || !session.canClose} onClick={session.requestClose} />
      <span className="workspace-command-bar__feedback" role="status" aria-live="polite" aria-atomic="true">
        {props.feedback}
      </span>
    </nav>
  );
}

function CommandButton(props: {
  controls?: string;
  disabled?: boolean;
  expanded?: boolean;
  icon: typeof Save;
  label: string;
  onClick: () => void;
}) {
  const Icon = props.icon;
  return (
    <button
      className="workspace-command"
      type="button"
      aria-controls={props.controls}
      aria-expanded={props.expanded}
      disabled={props.disabled}
      title={props.label}
      onClick={props.onClick}
    >
      <Icon aria-hidden="true" size={15} strokeWidth={1.9} />
      <span>{props.label}</span>
    </button>
  );
}
