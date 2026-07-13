import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { afterEach, describe, expect, it } from "vitest";

import { FontFamilyMark, FontSizeMark } from "./TextFormattingMarks";
import {
  MIXED_FONT_VALUE,
  effectiveFontControlState,
} from "./fontControlState";

const editors: Editor[] = [];

describe("effective font control state", () => {
  afterEach(() => editors.splice(0).forEach((editor) => editor.destroy()));

  it("reports document and heading defaults instead of empty placeholders", () => {
    const paragraph = createEditor({
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: "Body" }] }],
    });
    const heading = createEditor({
      type: "doc",
      content: [{ type: "heading", attrs: { level: 1 }, content: [{ type: "text", text: "Title" }] }],
    });

    expect(effectiveFontControlState(paragraph)).toEqual({
      fontFamily: "georgia",
      fontSize: "13",
    });
    expect(effectiveFontControlState(heading)).toEqual({
      fontFamily: "georgia",
      fontSize: "24",
    });
  });

  it("follows explicit formatting at the caret after a JSON round trip", () => {
    const original = formattedEditor();
    const restored = createEditor(original.getJSON());
    restored.commands.setTextSelection(9);

    expect(effectiveFontControlState(restored)).toEqual({
      fontFamily: "avenir_next",
      fontSize: "19",
    });
  });

  it("reports mixed family and size for a heterogeneous range", () => {
    const editor = formattedEditor();
    editor.commands.setTextSelection({ from: 2, to: 12 });

    expect(effectiveFontControlState(editor)).toEqual({
      fontFamily: MIXED_FONT_VALUE,
      fontSize: MIXED_FONT_VALUE,
    });
  });
});

function formattedEditor() {
  return createEditor({
    type: "doc",
    content: [
      {
        type: "paragraph",
        content: [
          { type: "text", text: "Plain " },
          {
            type: "text",
            text: "Styled",
            marks: [
              { type: "fontFamily", attrs: { family: "avenir_next" } },
              { type: "fontSize", attrs: { points: 19 } },
            ],
          },
        ],
      },
    ],
  });
}

function createEditor(content: Record<string, unknown>) {
  const editor = new Editor({
    content,
    extensions: [StarterKit, FontFamilyMark, FontSizeMark],
  });
  editors.push(editor);
  return editor;
}
