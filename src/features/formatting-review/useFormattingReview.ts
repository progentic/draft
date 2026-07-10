import type { Editor } from "@tiptap/react";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  runFormattingReview,
  type FormattingReviewClientError,
  type FormattingReviewFinding,
  type FormattingReviewResponse,
  type FormattingStyle,
} from "../../ipc/formattingReview";
import {
  applyFormattingHeadingLevel,
  collectFormattingSnapshot,
  formattingTarget,
  inspectFormattingTarget,
  type FormattingSnapshotContext,
} from "./formattingSnapshot";

export type FormattingReviewFailure =
  | FormattingReviewClientError
  | { type: "invalid-citation" };

export type FormattingReviewState =
  | { phase: "idle" }
  | { phase: "running" }
  | { phase: "stale" }
  | { phase: "failed"; error: FormattingReviewFailure }
  | {
      phase: "ready";
      review: FormattingReviewResponse;
      snapshot: FormattingSnapshotContext;
      generation: number;
    };

const IDLE_STATE: FormattingReviewState = { phase: "idle" };
const STALE_STATE: FormattingReviewState = { phase: "stale" };

export function useFormattingReview(editor: Editor | null) {
  const [state, setState] = useState<FormattingReviewState>(IDLE_STATE);
  const generationRef = useRef(0);
  const activeRunRef = useRef<string | undefined>(undefined);

  useEffect(() => editorUpdateSubscription(editor, generationRef, activeRunRef, setState), [editor]);

  const run = useCallback(
    async (style: FormattingStyle) => {
      if (!editor) {
        return;
      }
      const collection = collectFormattingSnapshot(editor, style);
      if (collection.status === "invalid-citation") {
        setState({ phase: "failed", error: { type: "invalid-citation" } });
        return;
      }
      const runId = globalThis.crypto.randomUUID();
      const generation = generationRef.current;
      activeRunRef.current = runId;
      setState({ phase: "running" });
      const result = await runFormattingReview(collection.snapshot.request);
      if (activeRunRef.current !== runId) {
        return;
      }
      if (generationRef.current !== generation) {
        setState(STALE_STATE);
        return;
      }
      setState(
        result.status === "ready"
          ? {
              phase: "ready",
              review: result.review,
              snapshot: collection.snapshot,
              generation,
            }
          : { phase: "failed", error: result.error },
      );
    },
    [editor],
  );

  const invalidate = useCallback(() => {
    activeRunRef.current = undefined;
    setState((current) => (current.phase === "idle" ? current : STALE_STATE));
  }, []);

  const inspect = useCallback(
    (finding: FormattingReviewFinding) =>
      actOnFinding(editor, state, generationRef.current, finding, inspectFormattingTarget, setState),
    [editor, state],
  );

  const apply = useCallback(
    (finding: FormattingReviewFinding, level: number) =>
      actOnFinding(
        editor,
        state,
        generationRef.current,
        finding,
        (currentEditor, target) => applyFormattingHeadingLevel(currentEditor, target, level),
        setState,
      ),
    [editor, state],
  );

  const dismiss = useCallback((finding: FormattingReviewFinding) => {
    setState((current) => dismissFinding(current, finding));
  }, []);

  return { state, run, invalidate, inspect, apply, dismiss };
}

function editorUpdateSubscription(
  editor: Editor | null,
  generationRef: React.MutableRefObject<number>,
  activeRunRef: React.MutableRefObject<string | undefined>,
  setState: React.Dispatch<React.SetStateAction<FormattingReviewState>>,
) {
  generationRef.current += 1;
  activeRunRef.current = undefined;
  setState(IDLE_STATE);
  if (!editor) {
    return;
  }
  const markStale = () => {
    generationRef.current += 1;
    activeRunRef.current = undefined;
    setState((current) => (current.phase === "idle" ? current : STALE_STATE));
  };
  editor.on("update", markStale);
  return () => {
    editor.off("update", markStale);
  };
}

function actOnFinding(
  editor: Editor | null,
  state: FormattingReviewState,
  currentGeneration: number,
  finding: FormattingReviewFinding,
  action: (
    editor: Editor,
    target: NonNullable<ReturnType<typeof formattingTarget>>,
  ) => boolean,
  setState: React.Dispatch<React.SetStateAction<FormattingReviewState>>,
) {
  if (!editor || state.phase !== "ready" || state.generation !== currentGeneration) {
    setState(STALE_STATE);
    return false;
  }
  const target = formattingTarget(state.snapshot, finding.target);
  if (!target || !action(editor, target)) {
    setState(STALE_STATE);
    return false;
  }
  return true;
}

function dismissFinding(
  state: FormattingReviewState,
  finding: FormattingReviewFinding,
): FormattingReviewState {
  if (state.phase !== "ready") {
    return state;
  }
  return {
    ...state,
    review: {
      ...state.review,
      findings: state.review.findings.filter((candidate) => candidate !== finding),
    },
  };
}
