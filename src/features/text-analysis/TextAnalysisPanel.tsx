import type { Editor } from "@tiptap/react";
import { LocateFixed, ScanText, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import {
  runTextAnalysis,
  type TextAnalysisClientError,
  type TextAnalysisFinding,
} from "../../ipc/textAnalysis";
import {
  createTextAnalysisSnapshot,
  type TextAnalysisSnapshot,
} from "./textAnalysisSnapshot";

interface TextAnalysisPanelProps {
  editor: Editor | null;
  isOpen: boolean;
  onClose: () => void;
}

type ReviewState =
  | { phase: "idle"; message: string }
  | { phase: "pending"; message: string }
  | { phase: "ready"; findings: TextAnalysisFinding[]; snapshot: TextAnalysisSnapshot }
  | { phase: "failed"; message: string };

export function TextAnalysisPanel(props: TextAnalysisPanelProps) {
  const [state, setState] = useState<ReviewState>({
    phase: "idle",
    message: "Run text checks to review five fixed writing patterns.",
  });
  const generation = useRef(0);

  useEffect(() => subscribeToEdits(props.editor, generation, setState), [props.editor]);

  const run = async () => {
    if (!props.editor) {
      return;
    }
    const snapshot = createTextAnalysisSnapshot(props.editor);
    if (!snapshot.text.trim()) {
      setState({ phase: "failed", message: "Write some text before running text checks." });
      return;
    }
    const currentGeneration = ++generation.current;
    setState({ phase: "pending", message: "Checking the current document." });
    const result = await runTextAnalysis(snapshot.text);
    if (generation.current !== currentGeneration) {
      return;
    }
    setState(
      result.status === "ready"
        ? { phase: "ready", findings: result.findings, snapshot }
        : { phase: "failed", message: analysisFailureMessage(result.error) },
    );
  };

  return (
    <section
      id="text-analysis-panel"
      className="workflow-panel text-analysis-panel"
      aria-labelledby="text-analysis-title"
      hidden={!props.isOpen}
    >
      <header className="workflow-panel__header">
        <div className="workflow-panel__title">
          <ScanText aria-hidden="true" size={16} />
          <h2 id="text-analysis-title">Text checks</h2>
        </div>
        <p>Fixed checks flag patterns for review. Findings are suggestions, not conclusions.</p>
        <button className="command-button command-button--primary" type="button" disabled={state.phase === "pending"} onClick={() => void run()}>
          <ScanText aria-hidden="true" size={14} /> Check document
        </button>
        <button className="icon-button workflow-panel__close" type="button" aria-label="Close text checks" onClick={props.onClose}>
          <X aria-hidden="true" size={16} />
        </button>
      </header>
      <ReviewResults editor={props.editor} state={state} />
    </section>
  );
}

function ReviewResults(props: { editor: Editor | null; state: ReviewState }) {
  if (props.state.phase !== "ready") {
    return <p className={`workflow-panel__message workflow-panel__message--${props.state.phase}`} role="status" aria-live="polite" aria-atomic="true">{props.state.message}</p>;
  }
  if (props.state.findings.length === 0) {
    return <p className="workflow-panel__message" role="status">No patterns were flagged by the five text checks.</p>;
  }
  const readyState = props.state;
  return (
    <div className="text-findings" role="status" aria-live="polite" aria-label={`${readyState.findings.length} text check findings`}>
      <p>{readyState.findings.length} suggestions to review.</p>
      <ol>
        {readyState.findings.map((finding, index) => (
          <FindingRow key={`${finding.code}-${finding.startByte}-${index}`} editor={props.editor} finding={finding} snapshot={readyState.snapshot} />
        ))}
      </ol>
    </div>
  );
}

function FindingRow(props: {
  editor: Editor | null;
  finding: TextAnalysisFinding;
  snapshot: TextAnalysisSnapshot;
}) {
  const passage = props.snapshot.excerpt(props.finding.startByte, props.finding.endByte);
  return (
    <li>
      <div className="text-finding__copy">
        <span className={`text-finding__severity text-finding__severity--${props.finding.severity}`}>{props.finding.severity === "warning" ? "Review" : "Suggestion"}</span>
        <strong>{props.finding.title}</strong>
        <p>{props.finding.explanation}</p>
        <blockquote aria-label="Flagged passage">{passage}</blockquote>
      </div>
      <button className="command-button" type="button" onClick={() => showPassage(props.editor, props.snapshot, props.finding)}>
        <LocateFixed aria-hidden="true" size={14} /> Show in document
      </button>
    </li>
  );
}

function showPassage(
  editor: Editor | null,
  snapshot: TextAnalysisSnapshot,
  finding: TextAnalysisFinding,
) {
  const range = snapshot.locate(finding.startByte, finding.endByte);
  if (editor && range) {
    editor.chain().focus().setTextSelection(range).scrollIntoView().run();
    editor.view.dom.focus();
  }
}

function subscribeToEdits(
  editor: Editor | null,
  generation: React.MutableRefObject<number>,
  setState: (state: ReviewState) => void,
) {
  if (!editor) {
    return;
  }
  const invalidate = () => {
    generation.current += 1;
    setState({ phase: "idle", message: "The document changed. Run text checks again." });
  };
  editor.on("update", invalidate);
  return () => {
    editor.off("update", invalidate);
  };
}

function analysisFailureMessage(error: TextAnalysisClientError) {
  if (error.type !== "command") {
    return "DRAFT could not verify the text-check results. Try again.";
  }
  if (error.code === "empty_text") {
    return "Write some text before running text checks.";
  }
  if (error.code === "text_too_long") {
    return "This document is too large for text checks. Shorten it before trying again.";
  }
  if (error.code === "runtime_unavailable" || error.code === "worker_unavailable") {
    return "Text checks are unavailable in this installation.";
  }
  if (error.code === "cancelled") {
    return "Text checks stopped before completion.";
  }
  return "Text checks did not finish. Try again.";
}
