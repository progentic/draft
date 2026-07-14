import { useEffect, useRef } from "react";

import type { PendingDocumentAction } from "./useDocumentSession";

interface UnsavedChangesDialogProps {
  action: PendingDocumentAction | null;
  onResolve: (decision: "cancel" | "discard" | "save") => void;
}

export function UnsavedChangesDialog(props: UnsavedChangesDialogProps) {
  const dialog = useRef<HTMLElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const saveButton = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (props.action) {
      previousFocus.current = document.activeElement as HTMLElement | null;
      saveButton.current?.focus();
      return;
    }
    previousFocus.current?.focus();
  }, [props.action]);

  if (!props.action) {
    return null;
  }

  return (
    <div className="dialog-backdrop">
      <section
        ref={dialog}
        className="confirmation-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="unsaved-dialog-title"
        aria-describedby="unsaved-dialog-message"
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            props.onResolve("cancel");
          }
          if (event.key === "Tab") {
            containFocus(event, dialog.current);
          }
        }}
      >
        <h2 id="unsaved-dialog-title">Save your changes?</h2>
        <p id="unsaved-dialog-message">
          {dialogMessage(props.action)} Unsaved changes will be lost if you continue without saving.
        </p>
        <div className="confirmation-dialog__actions">
          <button
            ref={saveButton}
            className="command-button command-button--primary"
            type="button"
            onClick={() => props.onResolve("save")}
          >
            Save and continue
          </button>
          <button className="command-button" type="button" onClick={() => props.onResolve("discard")}>Discard changes</button>
          <button className="command-button" type="button" onClick={() => props.onResolve("cancel")}>Keep editing</button>
        </div>
      </section>
    </div>
  );
}

function containFocus(event: React.KeyboardEvent, dialog: HTMLElement | null) {
  if (!dialog) {
    return;
  }
  const controls = Array.from(dialog.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"));
  const first = controls[0];
  const last = controls.at(-1);
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first?.focus();
  }
}

function dialogMessage(action: PendingDocumentAction) {
  if (action === "open" || action === "open_requested") {
    return "You are about to open another document.";
  }
  if (action === "new") {
    return "You are about to start a new document.";
  }
  return "You are about to close this document.";
}
