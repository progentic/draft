import type { Editor } from "@tiptap/react";

import { validateCitationNodeAttributes } from "../../citations/citationNode";
import type {
  FormattingCitationInput,
  FormattingReviewRequest,
  FormattingStyle,
  FormattingTarget,
} from "../../ipc/formattingReview";

export type EditorFormattingTarget =
  | {
      type: "heading";
      position: number;
      level: number;
      text: string;
    }
  | {
      type: "citation";
      position: number;
      citation: FormattingCitationInput;
    };

export interface FormattingSnapshotContext {
  request: FormattingReviewRequest;
  headings: Extract<EditorFormattingTarget, { type: "heading" }>[];
  citations: Extract<EditorFormattingTarget, { type: "citation" }>[];
}

export type FormattingSnapshotCollection =
  | { status: "ready"; snapshot: FormattingSnapshotContext }
  | { status: "invalid-citation" };

const EMPTY_HEADING_TITLE = "Untitled heading";
const SUPPORTED_EDITOR_HEADING_LEVELS = new Set([1, 2, 3]);

export function collectFormattingSnapshot(
  editor: Editor,
  style: FormattingStyle,
): FormattingSnapshotCollection {
  const headings: FormattingSnapshotContext["headings"] = [];
  const citations: FormattingSnapshotContext["citations"] = [];
  let invalidCitation = false;

  editor.state.doc.descendants((node, position) => {
    if (node.type.name === "heading") {
      headings.push({
        type: "heading",
        position,
        level: Number(node.attrs.level),
        text: node.textContent,
      });
    }
    if (node.type.name === "citation") {
      const validation = validateCitationNodeAttributes(node.attrs);
      if (!validation.valid) {
        invalidCitation = true;
        return;
      }
      citations.push({
        type: "citation",
        position,
        citation: {
          citekey: validation.attrs.citekey,
          renderStyle: validation.attrs.render_style,
        },
      });
    }
  });

  return invalidCitation
    ? { status: "invalid-citation" }
    : { status: "ready", snapshot: snapshotContext(style, headings, citations) };
}

export function formattingTarget(
  snapshot: FormattingSnapshotContext,
  target: FormattingTarget,
): EditorFormattingTarget | undefined {
  return target.type === "heading"
    ? snapshot.headings[target.index]
    : snapshot.citations[target.index];
}

export function inspectFormattingTarget(editor: Editor, target: EditorFormattingTarget) {
  if (!isCurrentFormattingTarget(editor, target)) {
    return false;
  }
  return target.type === "heading"
    ? editor.chain().focus().setTextSelection(target.position + 1).run()
    : editor.chain().focus().setNodeSelection(target.position).run();
}

export function applyFormattingHeadingLevel(
  editor: Editor,
  target: EditorFormattingTarget,
  level: number,
) {
  if (
    target.type !== "heading" ||
    !SUPPORTED_EDITOR_HEADING_LEVELS.has(level) ||
    !isCurrentFormattingTarget(editor, target)
  ) {
    return false;
  }

  return editor
    .chain()
    .focus()
    .command(({ tr }) => {
      const node = tr.doc.nodeAt(target.position);
      if (!node || node.type.name !== "heading") {
        return false;
      }
      tr.setNodeMarkup(target.position, undefined, { ...node.attrs, level });
      return true;
    })
    .run();
}

function snapshotContext(
  style: FormattingStyle,
  headings: FormattingSnapshotContext["headings"],
  citations: FormattingSnapshotContext["citations"],
): FormattingSnapshotContext {
  return {
    request: {
      style,
      headings: headings.map(({ level, text }) => ({
        level,
        title: text || EMPTY_HEADING_TITLE,
      })),
      citations: citations.map(({ citation }) => citation),
    },
    headings,
    citations,
  };
}

function isCurrentFormattingTarget(editor: Editor, target: EditorFormattingTarget) {
  const node = editor.state.doc.nodeAt(target.position);
  if (!node || node.type.name !== target.type) {
    return false;
  }
  if (target.type === "heading") {
    return Number(node.attrs.level) === target.level && node.textContent === target.text;
  }
  const validation = validateCitationNodeAttributes(node.attrs);
  return (
    validation.valid &&
    validation.attrs.citekey === target.citation.citekey &&
    validation.attrs.render_style === target.citation.renderStyle
  );
}
