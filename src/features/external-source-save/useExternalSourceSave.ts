import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { DocumentEnvelopeSnapshot } from "../../ipc/documentEnvelope";
import type { ExternalDocumentSummary } from "../../ipc/externalDocument";
import {
  saveExternalDocument,
  type SaveExternalDocumentResult,
} from "../../ipc/externalDocumentSave";

type WritableDisposition =
  | "allowed_exact"
  | "allowed_after_accepted_normalization";

export type ExternalSourceSaveOperation =
  | "checking_source"
  | "confirming_source_save"
  | "saving_source";

export interface ExternalSourceSaveConfirmation {
  displayName: string;
  disposition: WritableDisposition;
}

interface PendingConfirmation extends ExternalSourceSaveConfirmation {
  revision: number;
  snapshot: DocumentEnvelopeSnapshot;
}

type WorkflowState =
  | { phase: "idle" }
  | { phase: "blocked"; reason: string }
  | { phase: "confirmation"; pending: PendingConfirmation };

interface ExternalSourceSaveOptions {
  external: ExternalDocumentSummary | null;
  modified: boolean;
  operation: string;
  revision: number;
  snapshot: () => DocumentEnvelopeSnapshot | null;
  onFeedback: (feedback: string) => void;
  onOperation: (operation: "ready" | ExternalSourceSaveOperation) => void;
  onSaved: (documentId: string, displayName: string) => void;
}

export interface ExternalSourceSaveWorkflow {
  confirmation: ExternalSourceSaveConfirmation | null;
  enabled: boolean;
  request: () => void;
  resolve: (decision: "cancel" | "confirm") => void;
  unavailableReason: string;
  visible: boolean;
}

const IDLE_STATE: WorkflowState = { phase: "idle" };

export function useExternalSourceSave(
  options: ExternalSourceSaveOptions,
): ExternalSourceSaveWorkflow {
  const [state, setState] = useState<WorkflowState>(IDLE_STATE);
  const activeRun = useRef(0);
  const currentRevision = useRef(options.revision);
  currentRevision.current = options.revision;
  const targetKey = externalTargetKey(options.external);

  useEffect(() => {
    activeRun.current += 1;
    setState(IDLE_STATE);
  }, [options.revision, targetKey]);

  const enabled = canRequestSave(options, state);
  const request = useCallback(() => {
    if (!canRequestSave(options, state)) return;
    const snapshot = options.snapshot();
    if (!snapshot) return;
    void inspectSource(options, snapshot, activeRun, currentRevision, setState);
  }, [options, state]);
  const resolve = useCallback(
    (decision: "cancel" | "confirm") => {
      if (state.phase !== "confirmation") return;
      void resolveConfirmation(options, state.pending, decision, activeRun, setState);
    },
    [options, state],
  );

  return useMemo(
    () => ({
      confirmation:
        state.phase === "confirmation"
          ? publicConfirmation(state.pending)
          : null,
      enabled,
      request,
      resolve,
      unavailableReason: unavailableReason(options, state),
      visible: options.external !== null,
    }),
    [enabled, options, request, resolve, state],
  );
}

async function inspectSource(
  options: ExternalSourceSaveOptions,
  snapshot: DocumentEnvelopeSnapshot,
  activeRun: React.MutableRefObject<number>,
  currentRevision: React.MutableRefObject<number>,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  const run = ++activeRun.current;
  const revision = options.revision;
  options.onOperation("checking_source");
  options.onFeedback("Checking whether DRAFT can safely replace the source.");
  const result = await saveExternalDocument(snapshot, "inspect");
  if (!isCurrentRun(activeRun, currentRevision, run, options, revision)) return;
  presentEligibility(options, snapshot, revision, result, setState);
}

function presentEligibility(
  options: ExternalSourceSaveOptions,
  snapshot: DocumentEnvelopeSnapshot,
  revision: number,
  result: SaveExternalDocumentResult,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  if (result.status !== "eligibility" || result.documentId !== snapshot.document_id) {
    presentUnexpectedResult(options, result, setState);
    return;
  }
  if (isWritableDisposition(result.disposition)) {
    setState({
      phase: "confirmation",
      pending: {
        displayName: result.displayName,
        disposition: result.disposition,
        revision,
        snapshot,
      },
    });
    options.onOperation("confirming_source_save");
    options.onFeedback("Review the source replacement warning.");
    return;
  }
  if (result.disposition === "no_changes") {
    options.onSaved(result.documentId, result.displayName);
    setState(IDLE_STATE);
    options.onOperation("ready");
    options.onFeedback("The source already matches the current document.");
    return;
  }
  blockSave(options, denialMessage(result.disposition), setState);
}

async function resolveConfirmation(
  options: ExternalSourceSaveOptions,
  pending: PendingConfirmation,
  decision: "cancel" | "confirm",
  activeRun: React.MutableRefObject<number>,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  if (decision === "cancel") {
    await cancelReplacement(options, pending, activeRun, setState);
    return;
  }
  if (options.revision !== pending.revision) {
    setState(IDLE_STATE);
    options.onOperation("ready");
    options.onFeedback("The document changed. Choose Save Back to Source again.");
    return;
  }
  await replaceSource(options, pending, activeRun, setState);
}

async function cancelReplacement(
  options: ExternalSourceSaveOptions,
  pending: PendingConfirmation,
  activeRun: React.MutableRefObject<number>,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  const run = ++activeRun.current;
  setState(IDLE_STATE);
  options.onOperation("checking_source");
  const result = await saveExternalDocument(pending.snapshot, "cancel");
  if (activeRun.current !== run) return;
  if (result.status !== "cancelled") {
    presentUnexpectedResult(options, result, setState);
    return;
  }
  options.onOperation("ready");
  options.onFeedback("Save Back cancelled. The source was not changed.");
}

async function replaceSource(
  options: ExternalSourceSaveOptions,
  pending: PendingConfirmation,
  activeRun: React.MutableRefObject<number>,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  const run = ++activeRun.current;
  options.onOperation("saving_source");
  options.onFeedback("Replacing the source safely.");
  const decision = pending.disposition === "allowed_exact"
    ? "save_exact"
    : "accept_normalization";
  const result = await saveExternalDocument(pending.snapshot, decision);
  if (activeRun.current !== run) return;
  presentSaveResult(options, pending, result, setState);
}

function presentSaveResult(
  options: ExternalSourceSaveOptions,
  pending: PendingConfirmation,
  result: SaveExternalDocumentResult,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  if (
    (result.status === "saved" || result.status === "unchanged") &&
    result.documentId === pending.snapshot.document_id
  ) {
    options.onSaved(result.documentId, result.displayName);
    setState(IDLE_STATE);
    options.onOperation("ready");
    options.onFeedback(`Saved back to ${result.displayName}.`);
    return;
  }
  if (result.status === "cancelled") {
    setState(IDLE_STATE);
    options.onOperation("ready");
    options.onFeedback("Save Back cancelled. The source was not changed.");
    return;
  }
  presentUnexpectedResult(options, result, setState);
}

function presentUnexpectedResult(
  options: ExternalSourceSaveOptions,
  result: SaveExternalDocumentResult,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  if (result.status === "denied") {
    blockSave(options, denialMessage(result.disposition), setState);
    return;
  }
  if (result.status === "error") {
    const message = externalSaveFailureMessage(result);
    if (result.recovery === "retry") {
      setState(IDLE_STATE);
      options.onOperation("ready");
      options.onFeedback(message);
    } else {
      blockSave(options, message, setState);
    }
    return;
  }
  blockSave(
    options,
    "DRAFT could not confirm source safety. Save as a DRAFT document instead.",
    setState,
  );
}

function blockSave(
  options: ExternalSourceSaveOptions,
  reason: string,
  setState: React.Dispatch<React.SetStateAction<WorkflowState>>,
) {
  setState({ phase: "blocked", reason });
  options.onOperation("ready");
  options.onFeedback(reason);
}

function canRequestSave(
  options: ExternalSourceSaveOptions,
  state: WorkflowState,
) {
  return (
    options.external !== null &&
    options.modified &&
    options.operation === "ready" &&
    sourceMayBeWritable(options.external) &&
    state.phase === "idle"
  );
}

function sourceMayBeWritable(external: ExternalDocumentSummary) {
  return (
    external.fidelity.classification === "exact" ||
    external.fidelity.classification === "canonically_normalized"
  );
}

function unavailableReason(
  options: ExternalSourceSaveOptions,
  state: WorkflowState,
) {
  if (!options.external) return "";
  if (state.phase === "blocked") return state.reason;
  if (!sourceMayBeWritable(options.external)) {
    return "This source contains content DRAFT cannot replace safely. Save as a DRAFT document instead.";
  }
  if (!options.modified) return "The source has no changes to save back.";
  if (options.operation !== "ready") return "Finish the current document action first.";
  return "";
}

function denialMessage(
  disposition: Exclude<
    import("../../ipc/externalDocument").SameFormatSaveDisposition,
    "allowed_after_accepted_normalization" | "allowed_exact" | "no_changes"
  >,
) {
  switch (disposition) {
    case "denied_source_missing":
      return "The source is no longer available. Reopen it before saving back.";
    case "denied_source_changed":
      return "The source changed outside DRAFT. Reopen it before saving back.";
    case "denied_unsupported_source_behavior":
    case "denied_read_only":
    case "denied_missing_provenance":
    case "denied_fidelity_unknown":
    case "denied_writer_unavailable":
      return "DRAFT cannot safely replace this source. Save as a DRAFT document or export a new DOCX.";
    default:
      return assertNever(disposition);
  }
}

function externalSaveFailureMessage(
  result: Extract<SaveExternalDocumentResult, { status: "error" }>,
) {
  if (result.error.type !== "command") {
    return "DRAFT could not confirm the source’s final state. Reopen it before continuing.";
  }
  if (result.recovery === "reopen_source") {
    return "DRAFT could not confirm the source’s final state. Reopen it before continuing.";
  }
  if (result.recovery === "save_as_draft" || result.recovery === "none") {
    return "This document cannot be saved back safely. Save as a DRAFT document or export a new DOCX.";
  }
  return "DRAFT could not replace the source. The original was preserved. Try again.";
}

function externalTargetKey(external: ExternalDocumentSummary | null) {
  return external ? JSON.stringify(external) : "none";
}

function isCurrentRun(
  activeRun: React.MutableRefObject<number>,
  currentRevision: React.MutableRefObject<number>,
  run: number,
  options: ExternalSourceSaveOptions,
  revision: number,
) {
  if (activeRun.current === run && currentRevision.current === revision) return true;
  options.onOperation("ready");
  options.onFeedback("The document changed. Choose Save Back to Source again.");
  return false;
}

function isWritableDisposition(value: string): value is WritableDisposition {
  return (
    value === "allowed_exact" ||
    value === "allowed_after_accepted_normalization"
  );
}

function publicConfirmation(
  pending: PendingConfirmation,
): ExternalSourceSaveConfirmation {
  return {
    displayName: pending.displayName,
    disposition: pending.disposition,
  };
}

function assertNever(value: never): never {
  throw new Error(`unreachable source-save variant: ${String(value)}`);
}
