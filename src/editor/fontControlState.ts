import type { Editor } from "@tiptap/react";

import {
  isFontFamilyId,
  isFontSizePoints,
  type FontFamilyId,
} from "../documents/textFormatting";

type SelectionMark = NonNullable<Editor["state"]["storedMarks"]>[number];
type SelectionParent = Editor["state"]["selection"]["$from"]["parent"];
type ValueResolver = (marks: readonly SelectionMark[], parent: SelectionParent) => string;

export const DOCUMENT_FONT_FAMILY: FontFamilyId = "georgia";
export const DOCUMENT_FONT_SIZE_POINTS = 13;
export const MIXED_FONT_VALUE = "__mixed__";
export const RESET_FONT_VALUE = "__document_default__";

export function effectiveFontControlState(editor: Editor) {
  return {
    fontFamily: selectionValue(editor, effectiveFamily),
    fontSize: selectionValue(editor, effectiveSize),
  };
}

function selectionValue(editor: Editor, resolve: ValueResolver) {
  const selection = editor.state.selection;
  if (selection.empty) {
    return resolve(activeMarks(editor), selection.$from.parent);
  }
  const values = selectedTextValues(editor, resolve);
  return collapseValues(values) ?? resolve(activeMarks(editor), selection.$from.parent);
}

function selectedTextValues(editor: Editor, resolve: ValueResolver) {
  const values = new Set<string>();
  const { from, to, $from } = editor.state.selection;
  editor.state.doc.nodesBetween(from, to, (node, _position, parent) => {
    if (node.isText) {
      values.add(resolve(node.marks, parent ?? $from.parent));
    }
  });
  return values;
}

function collapseValues(values: Set<string>) {
  if (values.size > 1) {
    return MIXED_FONT_VALUE;
  }
  return values.values().next().value as string | undefined;
}

function activeMarks(editor: Editor) {
  return editor.state.storedMarks ?? editor.state.selection.$from.marks();
}

function effectiveFamily(marks: readonly SelectionMark[]) {
  const family = markAttribute(marks, "fontFamily", "family");
  return isFontFamilyId(family) ? family : DOCUMENT_FONT_FAMILY;
}

function effectiveSize(marks: readonly SelectionMark[], parent: SelectionParent) {
  const points = markAttribute(marks, "fontSize", "points");
  return String(isFontSizePoints(points) ? points : defaultSizeFor(parent));
}

function markAttribute(
  marks: readonly SelectionMark[],
  markName: string,
  attributeName: string,
) {
  return marks.find((mark) => mark.type.name === markName)?.attrs[attributeName];
}

function defaultSizeFor(parent: SelectionParent) {
  if (parent.type.name !== "heading") {
    return DOCUMENT_FONT_SIZE_POINTS;
  }
  return headingSize(parent.attrs.level);
}

function headingSize(level: unknown) {
  if (level === 1) {
    return 24;
  }
  return level === 2 ? 18 : 14;
}
