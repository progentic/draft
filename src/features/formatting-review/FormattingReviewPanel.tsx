import type { Editor } from "@tiptap/react";
import { Eye, ListChecks, SearchCheck, X } from "lucide-react";
import { useState } from "react";

import {
  DEFAULT_FORMATTING_STYLE,
  type FormattingAction,
  type FormattingReviewClientError,
  type FormattingReviewCommandErrorCode,
  type FormattingReviewFinding,
  type FormattingStyle,
} from "../../ipc/formattingReview";
import { useFormattingReview } from "./useFormattingReview";
import type { FormattingReviewFailure, FormattingReviewState } from "./useFormattingReview";

interface FormattingReviewPanelProps {
  editor: Editor | null;
  isOpen: boolean;
  onClose: () => void;
}

const FORMATTING_STYLE_LABELS = {
  apa7: "APA 7",
  mla9: "MLA 9",
  chicago17_author_date: "Chicago 17 author-date",
} satisfies Record<FormattingStyle, string>;

const FORMATTING_COMMAND_FAILURE_LABELS = {
  too_many_headings: "This document has too many headings for one formatting check.",
  too_many_citations: "This document has too many citations for one formatting check.",
  invalid_heading_level: "DRAFT could not validate a heading level in this document.",
  empty_heading_title: "DRAFT could not validate an empty heading in this document.",
  heading_title_too_long: "DRAFT could not check a heading because its title is too long.",
  invalid_citekey: "DRAFT could not validate a citation in this document.",
} satisfies Record<FormattingReviewCommandErrorCode, string>;

export function FormattingReviewPanel(props: FormattingReviewPanelProps) {
  const [style, setStyle] = useState<FormattingStyle>(DEFAULT_FORMATTING_STYLE);
  const review = useFormattingReview(props.editor);

  return (
    <section
      id="formatting-review-panel"
      className="formatting-review-panel"
      aria-labelledby="formatting-review-heading"
      hidden={!props.isOpen}
      inert={!props.isOpen}
    >
      <FormattingReviewHeader
        canRun={props.editor !== null}
        isRunning={review.state.phase === "running"}
        style={style}
        onClose={props.onClose}
        onRun={() => void review.run(style)}
        onStyleChange={(nextStyle) => {
          setStyle(nextStyle);
          review.invalidate();
        }}
      />
      <FormattingReviewContent
        state={review.state}
        onApply={review.apply}
        onDismiss={review.dismiss}
        onInspect={review.inspect}
      />
    </section>
  );
}

function FormattingReviewHeader(props: {
  canRun: boolean;
  isRunning: boolean;
  style: FormattingStyle;
  onClose: () => void;
  onRun: () => void;
  onStyleChange: (style: FormattingStyle) => void;
}) {
  return (
    <div className="formatting-review-header">
      <div className="formatting-review-title">
        <ListChecks aria-hidden="true" size={17} strokeWidth={1.8} />
        <h2 id="formatting-review-heading">Formatting review</h2>
      </div>
      <FormattingStyleSelector style={props.style} onChange={props.onStyleChange} />
      <button
        className="command-button command-button--primary"
        type="button"
        disabled={!props.canRun}
        onClick={props.onRun}
      >
        <SearchCheck aria-hidden="true" size={16} strokeWidth={1.9} />
        <span>{props.isRunning ? "Check again" : "Check formatting"}</span>
      </button>
      <button
        className="icon-button formatting-review-close"
        type="button"
        aria-label="Close formatting review"
        title="Close formatting review"
        onClick={props.onClose}
      >
        <X aria-hidden="true" size={17} strokeWidth={1.9} />
      </button>
    </div>
  );
}

function FormattingStyleSelector(props: {
  style: FormattingStyle;
  onChange: (style: FormattingStyle) => void;
}) {
  return (
    <fieldset className="formatting-style-selector">
      <legend>Document style</legend>
      <div className="segmented-control">
        {(Object.keys(FORMATTING_STYLE_LABELS) as FormattingStyle[]).map((style) => (
          <label key={style}>
            <input
              type="radio"
              name="formatting-style"
              value={style}
              checked={props.style === style}
              onChange={() => props.onChange(style)}
            />
            <span>{FORMATTING_STYLE_LABELS[style]}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}

function FormattingReviewContent(props: {
  state: FormattingReviewState;
  onApply: (finding: FormattingReviewFinding, level: number) => boolean;
  onDismiss: (finding: FormattingReviewFinding) => void;
  onInspect: (finding: FormattingReviewFinding) => boolean;
}) {
  if (props.state.phase !== "ready") {
    return <FormattingReviewStatus state={props.state} />;
  }
  const structure = props.state.review.findings.filter(
    (finding) => finding.target.type === "heading",
  );
  const citations = props.state.review.findings.filter(
    (finding) => finding.target.type === "citation",
  );

  return (
    <div className="formatting-review-groups" aria-live="polite">
      <FormattingFindingGroup
        findings={structure}
        label="Structure"
        {...props}
      />
      <FormattingFindingGroup
        findings={citations}
        label="Citations"
        {...props}
      />
    </div>
  );
}

function FormattingReviewStatus(props: { state: Exclude<FormattingReviewState, { phase: "ready" }> }) {
  if (props.state.phase === "failed") {
    return (
      <p className="formatting-review-status formatting-review-status--error" role="alert">
        {formattingFailureLabel(props.state.error)}
      </p>
    );
  }
  const labels = {
    idle: "Formatting has not been checked.",
    running: "Checking formatting.",
    stale: "The document changed. Run the formatting check again.",
  } as const;
  return (
    <p className="formatting-review-status" role="status" aria-live="polite">
      {labels[props.state.phase]}
    </p>
  );
}

function FormattingFindingGroup(props: {
  findings: FormattingReviewFinding[];
  label: string;
  onApply: (finding: FormattingReviewFinding, level: number) => boolean;
  onDismiss: (finding: FormattingReviewFinding) => void;
  onInspect: (finding: FormattingReviewFinding) => boolean;
  state: FormattingReviewState;
}) {
  return (
    <section className="formatting-finding-group" aria-label={`${props.label} findings`}>
      <h3>{props.label}</h3>
      {props.findings.length === 0 ? (
        <p className="formatting-finding-empty">No active findings</p>
      ) : (
        <ul className="formatting-finding-list">
          {props.findings.map((finding) => (
            <FormattingFindingRow
              finding={finding}
              key={findingKey(finding)}
              onApply={props.onApply}
              onDismiss={props.onDismiss}
              onInspect={props.onInspect}
            />
          ))}
        </ul>
      )}
    </section>
  );
}

function FormattingFindingRow(props: {
  finding: FormattingReviewFinding;
  onApply: (finding: FormattingReviewFinding, level: number) => boolean;
  onDismiss: (finding: FormattingReviewFinding) => void;
  onInspect: (finding: FormattingReviewFinding) => boolean;
}) {
  const applyAction = props.finding.actions.find(isApplyAction);
  return (
    <li className="formatting-finding">
      <div className="formatting-finding__copy">
        <span className={`formatting-severity formatting-severity--${props.finding.severity}`}>
          {props.finding.severity}
        </span>
        <span className="formatting-target-label">{formattingTargetLabel(props.finding)}</span>
        <strong>{props.finding.title}</strong>
        <p>{props.finding.explanation}</p>
      </div>
      <div className="formatting-finding__actions">
        <button
          className="command-button"
          type="button"
          onClick={() => props.onInspect(props.finding)}
        >
          <Eye aria-hidden="true" size={15} strokeWidth={1.9} />
          <span>Inspect</span>
        </button>
        {applyAction ? (
          <button
            className="command-button"
            type="button"
            onClick={() => props.onApply(props.finding, applyAction.level)}
          >
            Apply H{applyAction.level}
          </button>
        ) : null}
        <button
          className="icon-button formatting-dismiss"
          type="button"
          aria-label={`Dismiss ${props.finding.title}`}
          title="Dismiss finding"
          onClick={() => props.onDismiss(props.finding)}
        >
          <X aria-hidden="true" size={15} strokeWidth={1.9} />
        </button>
      </div>
    </li>
  );
}

function formattingFailureLabel(error: FormattingReviewFailure) {
  if (error.type === "command") {
    return FORMATTING_COMMAND_FAILURE_LABELS[error.code];
  }
  if (error.type === "invalid-citation") {
    return "DRAFT found a citation that cannot be checked.";
  }
  if (error.type === "invalid-response") {
    return "DRAFT received an invalid formatting response.";
  }
  return "Formatting review could not reach the DRAFT core.";
}

function formattingTargetLabel(finding: FormattingReviewFinding) {
  const label = finding.target.type === "heading" ? "Heading" : "Citation";
  return `${label} ${finding.target.index + 1}`;
}

function findingKey(finding: FormattingReviewFinding) {
  return `${finding.code}-${finding.target.type}-${finding.target.index}`;
}

function isApplyAction(action: FormattingAction): action is Extract<
  FormattingAction,
  { type: "apply_heading_level" }
> {
  return action.type === "apply_heading_level";
}

export function formattingCommandFailureLabel(code: FormattingReviewCommandErrorCode) {
  return FORMATTING_COMMAND_FAILURE_LABELS[code];
}

export function formattingClientFailureLabel(error: FormattingReviewClientError) {
  return formattingFailureLabel(error);
}
