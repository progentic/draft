import { EditorContent, useEditor } from "@tiptap/react";
import type { Editor, JSONContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";

import { CitationNode } from "./CitationNode";
import { FontFamilyMark, FontSizeMark } from "./TextFormattingMarks";
import { ParagraphFormatting } from "./ParagraphFormatting";
import { PageBreakNode } from "./PageBreakNode";

interface DraftEditorProps {
  editor: Editor | null;
}

export const INITIAL_DOCUMENT: JSONContent = {
  type: "doc",
  content: [{ type: "paragraph" }],
};

export function DraftEditor(props: DraftEditorProps) {
  return (
    <div className="editor-scroll">
      <div className="document-page">
        <EditorContent editor={props.editor} />
      </div>
    </div>
  );
}

export function useDraftEditor() {
  return useEditor({
    content: INITIAL_DOCUMENT,
    editorProps: {
      attributes: {
        "aria-label": "Document editor",
        class: "draft-editor",
        role: "textbox",
        spellcheck: "true",
      },
    },
    extensions: [
      StarterKit.configure({
        heading: {
          levels: [1, 2, 3, 4, 5, 6],
        },
      }),
      FontFamilyMark,
      FontSizeMark,
      ParagraphFormatting,
      PageBreakNode,
      CitationNode,
    ],
  });
}
