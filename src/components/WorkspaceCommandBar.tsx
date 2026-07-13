import {
  BookOpen,
  Download,
  FilePlus2,
  FolderOpen,
  Save,
  SaveAll,
  ScanText,
  X,
} from "lucide-react";

import type { WorkspaceActions } from "../features/workspace-actions/useWorkspaceActions";

interface WorkspaceCommandBarProps {
  activePanel: "references" | "text-review" | null;
  actions: WorkspaceActions;
  feedback: string;
  exportLabel: string;
}

export function WorkspaceCommandBar(props: WorkspaceCommandBarProps) {
  const actions = props.actions;

  return (
    <nav className="workspace-command-bar" aria-label="Document actions">
      <CommandButton icon={FilePlus2} label="New Document" disabled={!actions.enabled.new_document} onClick={() => actions.dispatch("new_document")} />
      <CommandButton icon={FolderOpen} label="Open…" disabled={!actions.enabled.open_document} onClick={() => actions.dispatch("open_document")} />
      <CommandButton icon={X} label="Close" disabled={!actions.enabled.close_document} onClick={() => actions.dispatch("close_document")} />
      <span className="workspace-command-bar__separator" aria-hidden="true" />
      <CommandButton icon={Save} label="Save" disabled={!actions.enabled.save_document} onClick={() => actions.dispatch("save_document")} />
      <CommandButton icon={SaveAll} label="Save As…" disabled={!actions.enabled.save_document_as} onClick={() => actions.dispatch("save_document_as")} />
      <span className="workspace-command-bar__separator" aria-hidden="true" />
      <CommandButton
        icon={Download}
        label={props.exportLabel}
        disabled={!actions.enabled.export_docx}
        onClick={() => actions.dispatch("export_docx")}
      />
      <span className="workspace-command-bar__separator" aria-hidden="true" />
      <CommandButton
        icon={BookOpen}
        label="References"
        disabled={!actions.enabled.open_references}
        controls="reference-library-panel"
        expanded={props.activePanel === "references"}
        onClick={() => actions.dispatch("open_references")}
      />
      <CommandButton
        icon={ScanText}
        label="Text checks"
        disabled={!actions.enabled.run_text_checks}
        controls="text-analysis-panel"
        expanded={props.activePanel === "text-review"}
        onClick={() => actions.dispatch("run_text_checks")}
      />
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
