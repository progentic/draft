import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { DocumentSession } from "../document-session/useDocumentSession";
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
  onTogglePanel: (panel: "references" | "text-review") => void,
): WorkspaceActions {
  const [feedback, setFeedback] = useState("");
  const enabled = useActionAvailability(session);
  const dispatch = useActionDispatcher(
    session,
    onTogglePanel,
    enabled,
    setFeedback,
  );
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

function useActionAvailability(session: DocumentSession): Record<WorkspaceAction, boolean> {
  return useMemo(() => {
    const ready = session.operation === "ready";
    return {
      new_document: ready,
      open_document: ready,
      close_document: ready && session.canClose,
      save_document: ready && session.canSave,
      save_document_as: ready && session.canSaveAs,
      save_back_to_source: ready && session.canSaveBack,
      open_references: ready,
      run_text_checks: ready,
    };
  }, [session]);
}

function useActionDispatcher(
  session: DocumentSession,
  onTogglePanel: (panel: "references" | "text-review") => void,
  enabled: Record<WorkspaceAction, boolean>,
  setFeedback: (feedback: string) => void,
) {
  return useCallback((action: WorkspaceAction) => {
    if (!enabled[action]) {
      setFeedback(unavailableActionMessage(action, session));
      return;
    }
    setFeedback("");
    dispatchEnabledAction(action, session, onTogglePanel);
  }, [enabled, onTogglePanel, session, setFeedback]);
}

function unavailableActionMessage(
  action: WorkspaceAction,
  session: DocumentSession,
) {
  if (action === "save_back_to_source" && session.saveBackUnavailableReason) {
    return session.saveBackUnavailableReason;
  }
  if (session.operation !== "ready") {
    return "Finish the current document action first.";
  }
  return "This document action is not available in the current state.";
}

function dispatchEnabledAction(
  action: WorkspaceAction,
  session: DocumentSession,
  onTogglePanel: (panel: "references" | "text-review") => void,
) {
  const documentActions: Partial<Record<WorkspaceAction, () => void>> = {
    new_document: session.requestNew,
    open_document: session.requestOpen,
    close_document: session.requestClose,
    save_document: () => void session.save(),
    save_document_as: session.requestSaveAs,
    save_back_to_source: session.requestSaveBack,
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
