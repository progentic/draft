import { Editor, type JSONContent } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { afterEach, describe, expect, it } from "vitest";

import { ParagraphFormatting } from "./ParagraphFormatting";

let editor: Editor | null = null;

afterEach(() => {
  editor?.destroy();
  editor = null;
});

describe("paragraph-formatting editor model", () => {
  it("preserves canonical paragraph data through editor JSON and HTML", () => {
    editor = createEditor(documentWith(style()));

    expect(editor.getJSON().content?.[0]?.attrs?.paragraphStyle).toEqual(style());
    expect(editor.getHTML()).toContain("data-draft-paragraph-style");
    expect(editor.getHTML()).toContain("text-align: justify");
  });

  it("ignores arbitrary pasted CSS and malformed DRAFT attributes", () => {
    editor = createEditor("<p style=\"text-align: center\">CSS</p>");
    expect(editor.getJSON().content?.[0]?.attrs?.paragraphStyle).toBeNull();

    editor.commands.setContent("<p data-draft-paragraph-style=\"not-json\">Bad</p>");
    expect(editor.getJSON().content?.[0]?.attrs?.paragraphStyle).toBeNull();
  });

  it("accepts only a complete canonical DRAFT HTML attribute", () => {
    const encoded = JSON.stringify(style()).replaceAll('"', "&quot;");
    editor = createEditor(`<p data-draft-paragraph-style="${encoded}">Canonical</p>`);

    expect(editor.getJSON().content?.[0]?.attrs?.paragraphStyle).toEqual(style());
  });
});

function createEditor(content: JSONContent | string): Editor {
  return new Editor({
    content,
    extensions: [StarterKit, ParagraphFormatting],
  });
}

function documentWith(paragraphStyle: ReturnType<typeof style>) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      attrs: { paragraphStyle },
      content: [{ type: "text", text: "Styled" }],
    }],
  };
}

function style() {
  return {
    schemaVersion: 1,
    alignment: "justify",
    lineSpacingHundredths: 150,
    spaceBeforeTwips: 120,
    spaceAfterTwips: 240,
    leftIndentTwips: 360,
    rightIndentTwips: 180,
    specialIndent: { kind: "hanging", twips: 720 },
  };
}
