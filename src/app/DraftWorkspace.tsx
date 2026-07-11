import type { Editor } from "@tiptap/react";
import { useState } from "react";

import { DocumentInspector } from "../components/DocumentInspector";
import { DocumentOutline } from "../components/DocumentOutline";
import { WorkspaceHeader } from "../components/WorkspaceHeader";
import { DraftEditor, useDraftEditor } from "../editor/DraftEditor";
import { EditorToolbar } from "../editor/EditorToolbar";
import { useConnectivityMode } from "../features/connectivity/useConnectivityMode";
import { FormattingReviewPanel } from "../features/formatting-review/FormattingReviewPanel";
import { useRuntimeStatus } from "../features/runtime-status/useRuntimeStatus";
import type { RuntimeConnectionState } from "../features/runtime-status/useRuntimeStatus";

export function DraftWorkspace() {
  const editor = useDraftEditor();
  const connectivity = useConnectivityMode();
  const runtimeStatus = useRuntimeStatus();
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);
  const [isFormattingReviewOpen, setIsFormattingReviewOpen] = useState(false);

  return (
    <main className="workspace-shell" aria-label="DRAFT workspace">
      <WorkspaceHeader
        connectivityState={connectivity.state}
        isOutlineOpen={isOutlineOpen}
        onRefreshConnectivity={() => void connectivity.refresh()}
        onSetConnectivityMode={(mode) => void connectivity.setMode(mode)}
        onToggleOutline={() => setIsOutlineOpen((isOpen) => !isOpen)}
      />
      <WorkspaceBody
        editor={editor}
        isFormattingReviewOpen={isFormattingReviewOpen}
        isOutlineOpen={isOutlineOpen}
        runtimeStatus={runtimeStatus}
        onCloseFormattingReview={() => setIsFormattingReviewOpen(false)}
        onToggleFormattingReview={() => setIsFormattingReviewOpen((isOpen) => !isOpen)}
      />
    </main>
  );
}

function WorkspaceBody(props: {
  editor: Editor | null;
  isFormattingReviewOpen: boolean;
  isOutlineOpen: boolean;
  runtimeStatus: RuntimeConnectionState;
  onCloseFormattingReview: () => void;
  onToggleFormattingReview: () => void;
}) {
  return (
    <div className={workspaceBodyClassName(props.isOutlineOpen)} data-testid="workspace-body">
      <DocumentOutline editor={props.editor} isOpen={props.isOutlineOpen} />
      <section className="editor-workspace" aria-label="Document workspace">
        <EditorToolbar
          editor={props.editor}
          formattingReviewOpen={props.isFormattingReviewOpen}
          onToggleFormattingReview={props.onToggleFormattingReview}
        />
        <FormattingReviewPanel
          editor={props.editor}
          isOpen={props.isFormattingReviewOpen}
          onClose={props.onCloseFormattingReview}
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
