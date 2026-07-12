import type { Editor } from "@tiptap/react";
import { useState } from "react";

import { DocumentInspector } from "../components/DocumentInspector";
import { DocumentOutline } from "../components/DocumentOutline";
import { WorkspaceHeader } from "../components/WorkspaceHeader";
import { WorkspaceCommandBar } from "../components/WorkspaceCommandBar";
import { DraftEditor, useDraftEditor } from "../editor/DraftEditor";
import { EditorToolbar } from "../editor/EditorToolbar";
import { useConnectivityMode } from "../features/connectivity/useConnectivityMode";
import { UnsavedChangesDialog } from "../features/document-session/UnsavedChangesDialog";
import { useDocumentSession } from "../features/document-session/useDocumentSession";
import { useDocxExport } from "../features/docx-export/useDocxExport";
import { FormattingReviewPanel } from "../features/formatting-review/FormattingReviewPanel";
import { ReferenceLibraryPanel } from "../features/references/ReferenceLibraryPanel";
import { useRuntimeStatus } from "../features/runtime-status/useRuntimeStatus";
import type { RuntimeConnectionState } from "../features/runtime-status/useRuntimeStatus";
import { TextAnalysisPanel } from "../features/text-analysis/TextAnalysisPanel";

type WorkspacePanel = "formatting" | "references" | "text-review" | null;

export function DraftWorkspace() {
  const editor = useDraftEditor();
  const connectivity = useConnectivityMode();
  const runtimeStatus = useRuntimeStatus();
  const documentSession = useDocumentSession(editor);
  const docxExport = useDocxExport(documentSession);
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);
  const [activePanel, setActivePanel] = useState<WorkspacePanel>(null);

  const togglePanel = (panel: Exclude<WorkspacePanel, null>) => {
    setActivePanel((active) => (active === panel ? null : panel));
  };

  return (
    <main className="workspace-shell" aria-label="DRAFT workspace">
      <WorkspaceHeader
        connectivityState={connectivity.state}
        documentStatus={documentSession.statusLabel}
        documentTitle={documentSession.title}
        isOutlineOpen={isOutlineOpen}
        onRefreshConnectivity={() => void connectivity.refresh()}
        onSetConnectivityMode={(mode) => void connectivity.setMode(mode)}
        onToggleOutline={() => setIsOutlineOpen((isOpen) => !isOpen)}
      />
      <WorkspaceCommandBar
        activePanel={activePanel === "references" || activePanel === "text-review" ? activePanel : null}
        documentSession={documentSession}
        exportDisabled={docxExport.disabled}
        exportLabel={docxExport.label}
        feedback={docxExport.feedback || documentSession.feedback}
        onExport={docxExport.run}
        onTogglePanel={togglePanel}
      />
      <WorkspaceBody
        activePanel={activePanel}
        editor={editor}
        isOutlineOpen={isOutlineOpen}
        runtimeStatus={runtimeStatus}
        onClosePanel={() => setActivePanel(null)}
        onToggleFormattingReview={() => togglePanel("formatting")}
      />
      <UnsavedChangesDialog
        action={documentSession.pendingAction}
        onResolve={documentSession.resolvePendingAction}
      />
    </main>
  );
}

function WorkspaceBody(props: {
  activePanel: WorkspacePanel;
  editor: Editor | null;
  isOutlineOpen: boolean;
  runtimeStatus: RuntimeConnectionState;
  onClosePanel: () => void;
  onToggleFormattingReview: () => void;
}) {
  return (
    <div className={workspaceBodyClassName(props.isOutlineOpen)} data-testid="workspace-body">
      <DocumentOutline editor={props.editor} isOpen={props.isOutlineOpen} />
      <section className="editor-workspace" aria-label="Document workspace">
        <EditorToolbar
          editor={props.editor}
          formattingReviewOpen={props.activePanel === "formatting"}
          onToggleFormattingReview={props.onToggleFormattingReview}
        />
        <FormattingReviewPanel
          editor={props.editor}
          isOpen={props.activePanel === "formatting"}
          onClose={props.onClosePanel}
        />
        <ReferenceLibraryPanel
          editor={props.editor}
          isOpen={props.activePanel === "references"}
          onClose={props.onClosePanel}
        />
        <TextAnalysisPanel
          editor={props.editor}
          isOpen={props.activePanel === "text-review"}
          onClose={props.onClosePanel}
        />
        <DraftEditor editor={props.editor} />
      </section>
      <DocumentInspector editor={props.editor} runtimeStatus={props.runtimeStatus} />
    </div>
  );
}

function workspaceBodyClassName(isOutlineOpen: boolean) {
  return isOutlineOpen
    ? "workspace-body"
    : "workspace-body workspace-body--outline-closed";
}
