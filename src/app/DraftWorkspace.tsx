import type { Editor } from "@tiptap/react";
import { useState } from "react";

import { DocumentInspector } from "../components/DocumentInspector";
import { DocumentOutline } from "../components/DocumentOutline";
import { WorkspaceHeader } from "../components/WorkspaceHeader";
import { DraftEditor, useDraftEditor } from "../editor/DraftEditor";
import { EditorToolbar } from "../editor/EditorToolbar";
import { useRuntimeStatus } from "../features/runtime-status/useRuntimeStatus";
import type { RuntimeConnectionState } from "../features/runtime-status/useRuntimeStatus";

export function DraftWorkspace() {
  const editor = useDraftEditor();
  const runtimeStatus = useRuntimeStatus();
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);

  return (
    <main className="workspace-shell" aria-label="DRAFT workspace">
      <WorkspaceHeader
        isOutlineOpen={isOutlineOpen}
        onToggleOutline={() => setIsOutlineOpen((isOpen) => !isOpen)}
      />
      <WorkspaceBody
        editor={editor}
        isOutlineOpen={isOutlineOpen}
        runtimeStatus={runtimeStatus}
      />
    </main>
  );
}

function WorkspaceBody(props: {
  editor: Editor | null;
  isOutlineOpen: boolean;
  runtimeStatus: RuntimeConnectionState;
}) {
  return (
    <div className={workspaceBodyClassName(props.isOutlineOpen)} data-testid="workspace-body">
      <DocumentOutline editor={props.editor} isOpen={props.isOutlineOpen} />
      <section className="editor-workspace" aria-label="Document workspace">
        <EditorToolbar editor={props.editor} />
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
