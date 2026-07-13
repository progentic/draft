import { useEffect, useRef } from "react";

import type { ExternalNormalizationFeature } from "../../ipc/externalDocument";
import type { ExternalSourceSaveConfirmation } from "./useExternalSourceSave";

interface SaveBackToSourceDialogProps {
  confirmation: ExternalSourceSaveConfirmation | null;
  onResolve: (decision: "cancel" | "confirm") => void;
}

export function SaveBackToSourceDialog(
  props: SaveBackToSourceDialogProps,
) {
  const dialog = useRef<HTMLElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const confirmButton = useRef<HTMLButtonElement>(null);

  useEffect(() => manageDialogFocus(props.confirmation, previousFocus, confirmButton), [
    props.confirmation,
  ]);

  if (!props.confirmation) return null;
  const normalized =
    props.confirmation.disposition ===
    "allowed_after_accepted_normalization";
  return (
    <div className="dialog-backdrop">
      <section
        ref={dialog}
        className="confirmation-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="source-save-title"
        aria-describedby="source-save-message"
        onKeyDown={(event) => handleDialogKey(event, dialog.current, props.onResolve)}
      >
        <h2 id="source-save-title">Replace the source DOCX?</h2>
        <p id="source-save-message">
          {sourceReplacementMessage(props.confirmation.displayName, normalized)}
        </p>
        {normalized ? (
          <NormalizationDetails features={props.confirmation.normalizations} />
        ) : null}
        <div className="confirmation-dialog__actions">
          <button
            ref={confirmButton}
            className="command-button command-button--primary"
            type="button"
            onClick={() => props.onResolve("confirm")}
          >
            Replace
          </button>
          <button
            className="command-button"
            type="button"
            onClick={() => props.onResolve("cancel")}
          >
            Cancel
          </button>
        </div>
      </section>
    </div>
  );
}

function manageDialogFocus(
  confirmation: ExternalSourceSaveConfirmation | null,
  previousFocus: React.MutableRefObject<HTMLElement | null>,
  confirmButton: React.RefObject<HTMLButtonElement | null>,
) {
  if (confirmation) {
    previousFocus.current = document.activeElement as HTMLElement | null;
    confirmButton.current?.focus();
    return;
  }
  previousFocus.current?.focus();
}

function handleDialogKey(
  event: React.KeyboardEvent,
  dialog: HTMLElement | null,
  resolve: (decision: "cancel" | "confirm") => void,
) {
  if (event.key === "Escape") resolve("cancel");
  if (event.key === "Tab") containFocus(event, dialog);
}

function containFocus(event: React.KeyboardEvent, dialog: HTMLElement | null) {
  if (!dialog) return;
  const controls = Array.from(
    dialog.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"),
  );
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

function sourceReplacementMessage(displayName: string, normalized: boolean) {
  if (normalized) {
    return `DRAFT will replace ${displayName} and apply the listed normalization. No unsupported source content is eligible for replacement.`;
  }
  return `DRAFT will replace ${displayName} with the current supported content. This overwrite cannot be undone in DRAFT.`;
}

function NormalizationDetails(props: { features: ExternalNormalizationFeature[] }) {
  return (
    <div className="confirmation-dialog__details">
      <strong>What will change</strong>
      <ul>
        {props.features.map((feature) => (
          <li key={feature}>{normalizationMessage(feature)}</li>
        ))}
      </ul>
    </div>
  );
}

function normalizationMessage(feature: ExternalNormalizationFeature) {
  switch (feature) {
    case "alternate_heading_style_name":
      return "Alternate heading style names will use DRAFT’s standard heading names.";
    default:
      return assertNever(feature);
  }
}

function assertNever(value: never): never {
  throw new Error(`unreachable normalization feature: ${String(value)}`);
}
