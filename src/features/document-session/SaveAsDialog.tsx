import { useEffect, useRef, useState } from "react";

import { SAVE_AS_FORMATS, type SaveAsFormat } from "../../ipc/documentSave";

interface SaveAsDialogProps {
  open: boolean;
  onResolve: (format: SaveAsFormat | "cancel") => void;
}

export function SaveAsDialog(props: SaveAsDialogProps) {
  const dialog = useRef<HTMLElement>(null);
  const formatControl = useRef<HTMLSelectElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const [format, setFormat] = useState<SaveAsFormat>("draft");

  useEffect(() => manageDialogFocus(props.open, previousFocus, formatControl), [props.open]);
  useEffect(() => {
    if (props.open) setFormat("draft");
  }, [props.open]);
  if (!props.open) return null;

  return (
    <div className="dialog-backdrop">
      <section
        ref={dialog}
        className="confirmation-dialog save-as-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="save-as-dialog-title"
        aria-describedby="save-as-dialog-message"
        onKeyDown={(event) => handleDialogKey(event, dialog.current, props.onResolve)}
      >
        <h2 id="save-as-dialog-title">Save As</h2>
        <p id="save-as-dialog-message">
          Choose an editable DRAFT document or create a converted copy.
        </p>
        <label className="save-as-dialog__field">
          <span>File format</span>
          <select
            ref={formatControl}
            value={format}
            onChange={(event) => setFormat(event.target.value as SaveAsFormat)}
          >
            {SAVE_AS_FORMATS.map((value) => (
              <option key={value} value={value}>{formatLabel(value)}</option>
            ))}
          </select>
        </label>
        <p className="save-as-dialog__note" aria-live="polite">{formatNote(format)}</p>
        <div className="confirmation-dialog__actions">
          <button
            className="command-button command-button--primary"
            type="button"
            onClick={() => props.onResolve(format)}
          >
            Continue
          </button>
          <button className="command-button" type="button" onClick={() => props.onResolve("cancel")}>
            Cancel
          </button>
        </div>
      </section>
    </div>
  );
}

function manageDialogFocus(
  open: boolean,
  previousFocus: React.MutableRefObject<HTMLElement | null>,
  formatControl: React.RefObject<HTMLSelectElement | null>,
) {
  if (open) {
    previousFocus.current = document.activeElement as HTMLElement | null;
    queueMicrotask(() => formatControl.current?.focus());
    return;
  }
  previousFocus.current?.focus();
}

function handleDialogKey(
  event: React.KeyboardEvent,
  dialog: HTMLElement | null,
  resolve: (format: SaveAsFormat | "cancel") => void,
) {
  if (event.key === "Escape") {
    event.preventDefault();
    resolve("cancel");
    return;
  }
  if (event.key === "Tab") containFocus(event, dialog);
}

function containFocus(event: React.KeyboardEvent, dialog: HTMLElement | null) {
  if (!dialog) return;
  const controls = Array.from(
    dialog.querySelectorAll<HTMLElement>("button:not(:disabled), select:not(:disabled)"),
  ).sort(compareDocumentOrder);
  const first = controls[0];
  const last = controls.at(-1);
  const active = event.target;
  if (event.shiftKey && active === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && active === last) {
    event.preventDefault();
    first?.focus();
  }
}

function compareDocumentOrder(left: HTMLElement, right: HTMLElement) {
  return left.compareDocumentPosition(right) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
}

function formatLabel(format: SaveAsFormat) {
  if (format === "draft") return "DRAFT Document (.draft)";
  if (format === "docx") return "Word Document (.docx)";
  return "Plain Text (.txt)";
}

function formatNote(format: SaveAsFormat) {
  if (format === "draft") {
    return "This becomes the editable DRAFT document and updates the active file name.";
  }
  if (format === "docx") {
    return "Creates a Word copy. The active DRAFT document and Unsaved state do not change.";
  }
  return "Creates a UTF-8 text copy without formatting. The active DRAFT document and Unsaved state do not change.";
}
