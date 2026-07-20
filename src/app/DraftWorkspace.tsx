import type { Editor } from "@tiptap/react";
import { useCallback, useState } from "react";

import { DocumentInspector } from "../components/DocumentInspector";
import { DocumentOutline } from "../components/DocumentOutline";
import { WorkspaceHeader } from "../components/WorkspaceHeader";
import { WorkspaceOperationNotice } from "../components/WorkspaceOperationNotice";
import { WorkspaceCommandBar } from "../components/WorkspaceCommandBar";
import { WorkspaceStatusBar } from "../components/WorkspaceStatusBar";
import { DraftEditor, useDraftEditor } from "../editor/DraftEditor";
import { EditorToolbar } from "../editor/EditorToolbar";
import { useConnectivityMode } from "../features/connectivity/useConnectivityMode";
import { UnsavedChangesDialog } from "../features/document-session/UnsavedChangesDialog";
import { SaveAsDialog } from "../features/document-session/SaveAsDialog";
import { useDocumentSession } from "../features/document-session/useDocumentSession";
import { SaveBackToSourceDialog } from "../features/external-source-save/SaveBackToSourceDialog";
import { FormattingReviewPanel } from "../features/formatting-review/FormattingReviewPanel";
import { ReferenceLibraryPanel } from "../features/references/ReferenceLibraryPanel";
import { useRuntimeStatus } from "../features/runtime-status/useRuntimeStatus";
import { TextAnalysisPanel } from "../features/text-analysis/TextAnalysisPanel";
import { useWorkspaceActions } from "../features/workspace-actions/useWorkspaceActions";
import { useWindowTitle } from "../features/window-title/useWindowTitle";

type WorkspacePanel = "formatting" | "references" | "text-review" | null;

export function DraftWorkspace() {
  const editor = useDraftEditor();
  const connectivity = useConnectivityMode();
  const runtimeStatus = useRuntimeStatus();
  const documentSession = useDocumentSession(editor);
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);
  const [activePanel, setActivePanel] = useState<WorkspacePanel>(null);
  const windowTitleFeedback = useWindowTitle(
    documentSession.documentId ? documentSession.title : null,
    documentSession.unsaved,
  );

  const togglePanel = useCallback((panel: Exclude<WorkspacePanel, null>) => {
    setActivePanel((active) => (active === panel ? null : panel));
  }, []);
  const workspaceActions = useWorkspaceActions(documentSession, togglePanel);
  const operationMessage = workspaceActions.feedback ||
    documentSession.feedback ||
    windowTitleFeedback;
  const operationPending = documentSession.operation !== "ready";

  return (
    <main className="workspace-shell" aria-label="DRAFT workspace">
      <WorkspaceHeader
        documentTitle={documentSession.title}
        isOutlineOpen={isOutlineOpen}
        unsaved={documentSession.unsaved}
        onToggleOutline={() => setIsOutlineOpen((isOpen) => !isOpen)}
      />
      <WorkspaceCommandBar
        actions={workspaceActions}
        activePanel={activePanel === "references" || activePanel === "text-review" ? activePanel : null}
      />
      <WorkspaceOperationNotice
        message={operationMessage}
        pending={operationPending}
      />
      <WorkspaceBody
        activePanel={activePanel}
        editor={editor}
        isOutlineOpen={isOutlineOpen}
        onClosePanel={() => setActivePanel(null)}
        onToggleFormattingReview={() => togglePanel("formatting")}
      />
      <WorkspaceStatusBar
        connectivityState={connectivity.state}
        documentStatus={documentSession.statusLabel}
        operation={documentSession.operation}
        runtimeStatus={runtimeStatus}
        onRefreshConnectivity={() => void connectivity.refresh()}
        onSetConnectivityMode={(mode) => void connectivity.setMode(mode)}
      />
      <UnsavedChangesDialog
        action={documentSession.pendingAction}
        onResolve={documentSession.resolvePendingAction}
      />
      <SaveAsDialog open={documentSession.saveAsOpen} onResolve={documentSession.resolveSaveAs} />
      <SaveBackToSourceDialog
        confirmation={documentSession.saveBackConfirmation}
        onResolve={documentSession.resolveSaveBack}
      />
    </main>
  );
}

function WorkspaceBody(props: {
  activePanel: WorkspacePanel;
  editor: Editor | null;
  isOutlineOpen: boolean;
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
      <DocumentInspector editor={props.editor} />
    </div>
  );
}

function workspaceBodyClassName(isOutlineOpen: boolean) {
  return isOutlineOpen
    ? "workspace-body"
    : "workspace-body workspace-body--outline-closed";
}
