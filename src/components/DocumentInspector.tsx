import type { Editor } from "@tiptap/react";
import { useEditorState } from "@tiptap/react";
import { FileCheck2 } from "lucide-react";

import { runtimeFailureMessage } from "../features/error-ux/errorPresentation";
import type { RuntimeConnectionState } from "../features/runtime-status/useRuntimeStatus";

interface DocumentInspectorProps {
  editor: Editor | null;
  runtimeStatus: RuntimeConnectionState;
}

interface DocumentMetrics {
  characters: number;
  headings: number;
  words: number;
}

const EMPTY_METRICS: DocumentMetrics = {
  characters: 0,
  headings: 0,
  words: 0,
};

export function DocumentInspector(props: DocumentInspectorProps) {
  const metrics = useDocumentMetrics(props.editor);

  return (
    <aside className="document-inspector" aria-label="Document details">
      <div className="panel-heading">
        <FileCheck2 aria-hidden="true" size={16} strokeWidth={1.8} />
        <h2>Document</h2>
      </div>
      <SessionSection runtimeStatus={props.runtimeStatus} />
      <StatisticsSection metrics={metrics} />
    </aside>
  );
}

function SessionSection(props: { runtimeStatus: RuntimeConnectionState }) {
  return (
    <section className="inspector-section" aria-labelledby="session-heading">
      <h3 id="session-heading">Session</h3>
      <div className="inspector-status-list">
        <div className="inspector-status inspector-status--unsaved">
          <span className="inspector-status__dot" aria-hidden="true" />
          <span>Unsaved</span>
        </div>
        <RuntimeStatusRow runtimeStatus={props.runtimeStatus} />
      </div>
    </section>
  );
}

function RuntimeStatusRow(props: { runtimeStatus: RuntimeConnectionState }) {
  const view = runtimeStatusView(props.runtimeStatus);

  return (
    <div
      className={`inspector-status inspector-status--${view.modifier}`}
      role="status"
      aria-live="polite"
      aria-atomic="true"
    >
      <span className="inspector-status__dot" aria-hidden="true" />
      <span>{view.label}</span>
    </div>
  );
}

function runtimeStatusView(status: RuntimeConnectionState) {
  if (status.phase === "checking") {
    return { label: "Connecting to core", modifier: "checking" };
  }

  if (status.phase === "ready") {
    return { label: `Core v${status.version}`, modifier: "ready" };
  }

  return { label: runtimeFailureMessage(status.reason), modifier: "unavailable" };
}

function StatisticsSection(props: { metrics: DocumentMetrics }) {
  return (
    <section className="inspector-section" aria-labelledby="statistics-heading">
      <h3 id="statistics-heading">Statistics</h3>
      <dl className="metric-list">
        <Metric label="Words" value={props.metrics.words} />
        <Metric label="Characters" value={props.metrics.characters} />
        <Metric label="Headings" value={props.metrics.headings} />
      </dl>
    </section>
  );
}

function Metric(props: { label: string; value: number }) {
  return (
    <div className="metric-list__row">
      <dt>{props.label}</dt>
      <dd>{props.value.toLocaleString()}</dd>
    </div>
  );
}

function useDocumentMetrics(editor: Editor | null) {
  return (
    useEditorState({
      editor,
      selector: ({ editor: currentEditor }) =>
        currentEditor ? calculateMetrics(currentEditor) : EMPTY_METRICS,
    }) ?? EMPTY_METRICS
  );
}

function calculateMetrics(editor: Editor): DocumentMetrics {
  return {
    characters: editor.getText().length,
    headings: countHeadings(editor),
    words: countWords(editor.getText({ blockSeparator: " " })),
  };
}

function countHeadings(editor: Editor) {
  let headingCount = 0;

  editor.state.doc.descendants((node) => {
    if (node.type.name === "heading") {
      headingCount += 1;
    }
  });

  return headingCount;
}

function countWords(text: string) {
  const normalizedText = text.trim();
  return normalizedText ? normalizedText.split(/\s+/u).length : 0;
}
