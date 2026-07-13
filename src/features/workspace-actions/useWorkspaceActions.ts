import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { DocumentSession } from "../document-session/useDocumentSession";
import type { DocxExportState } from "../docx-export/useDocxExport";
import {
  listenToNativeMenuActions,
  setNativeMenuState,
  type NativeMenuAction,
  type NativeMenuState,
} from "../../ipc/nativeMenu";

export type WorkspaceAction = NativeMenuAction | "open_references" | "run_text_checks";

export interface WorkspaceActions {
  dispatch: (action: WorkspaceAction) => void;
  enabled: Record<WorkspaceAction, boolean>;
  feedback: string;
  sourceSave: {
    unavailableReason: string;
    visible: boolean;
  };
}

export function useWorkspaceActions(
  session: DocumentSession,
  docxExport: DocxExportState,
  onTogglePanel: (panel: "references" | "text-review") => void,
): WorkspaceActions {
  const [feedback, setFeedback] = useState("");
  const enabled = useActionAvailability(session, docxExport);
  const dispatch = useActionDispatcher(session, docxExport, onTogglePanel, enabled);
  const dispatchRef = useRef(dispatch);
  dispatchRef.current = dispatch;

  useNativeMenuListener(dispatchRef, setFeedback);
  useNativeMenuState(enabled, setFeedback);

  return {
    dispatch,
    enabled,
    feedback,
    sourceSave: {
      unavailableReason: session.saveBackUnavailableReason,
      visible: session.saveBackVisible,
    },
  };
}

function useActionAvailability(
  session: DocumentSession,
  docxExport: DocxExportState,
): Record<WorkspaceAction, boolean> {
  return useMemo(() => {
    const ready = session.operation === "ready" && !docxExport.disabled;
    return {
      new_document: ready,
      open_document: ready,
      close_document: ready && session.canClose,
      save_document: ready && session.canSave,
      save_document_as: ready && session.canSaveAs,
      save_back_to_source: ready && session.canSaveBack,
      export_docx: ready && session.canExport,
      open_references: ready,
      run_text_checks: ready,
    };
  }, [docxExport.disabled, session]);
}

function useActionDispatcher(
  session: DocumentSession,
  docxExport: DocxExportState,
  onTogglePanel: (panel: "references" | "text-review") => void,
  enabled: Record<WorkspaceAction, boolean>,
) {
  return useCallback((action: WorkspaceAction) => {
    if (!enabled[action]) {
      return;
    }
    dispatchEnabledAction(action, session, docxExport, onTogglePanel);
  }, [docxExport, enabled, onTogglePanel, session]);
}

function dispatchEnabledAction(
  action: WorkspaceAction,
  session: DocumentSession,
  docxExport: DocxExportState,
  onTogglePanel: (panel: "references" | "text-review") => void,
) {
  const documentActions: Partial<Record<WorkspaceAction, () => void>> = {
    new_document: session.requestNew,
    open_document: session.requestOpen,
    close_document: session.requestClose,
    save_document: () => void session.save(),
    save_document_as: () => void session.saveAs(),
    save_back_to_source: session.requestSaveBack,
    export_docx: docxExport.run,
    open_references: () => onTogglePanel("references"),
    run_text_checks: () => onTogglePanel("text-review"),
  };
  documentActions[action]?.();
}

function useNativeMenuListener(
  dispatchRef: React.RefObject<(action: WorkspaceAction) => void>,
  setFeedback: (feedback: string) => void,
) {
  useEffect(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    void listenToNativeMenuActions(
      (action) => dispatchRef.current?.(action),
      () => setFeedback("DRAFT could not read a native menu action. Use the toolbar."),
    )
      .then((listener) => {
        if (disposed) {
          listener();
        } else {
          stop = listener;
        }
      })
      .catch(() => {
        if (!disposed) {
          setFeedback("DRAFT could not read native menu actions. Use the toolbar.");
        }
      });
    return () => {
      disposed = true;
      stop?.();
    };
  }, [dispatchRef, setFeedback]);
}

function useNativeMenuState(
  enabled: Record<WorkspaceAction, boolean>,
  setFeedback: (feedback: string) => void,
) {
  const state = useMemo<NativeMenuState>(() => ({
    canNew: enabled.new_document,
    canOpen: enabled.open_document,
    canClose: enabled.close_document,
    canSave: enabled.save_document,
    canSaveAs: enabled.save_document_as,
    canSaveBack: enabled.save_back_to_source,
    canExport: enabled.export_docx,
  }), [enabled]);

  useEffect(() => {
    let current = true;
    void setNativeMenuState(state).then((result) => {
      if (current && result.status === "error") {
        setFeedback("DRAFT could not update the native menu. Use the toolbar.");
      }
    });
    return () => {
      current = false;
    };
  }, [setFeedback, state]);
}
