import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { describe, expect, it } from "vitest";

import { FontFamilyMark, FontSizeMark } from "./TextFormattingMarks";

describe("text formatting marks", () => {
  it("preserves existing marks and does not format unrelated text", () => {
    const editor = createEditor("Alpha Beta");
    editor.commands.setTextSelection({ from: 1, to: 6 });
    editor.chain().setBold().setMark("fontFamily", { family: "georgia" }).run();
    editor.commands.setMark("fontSize", { points: 18 });

    const content = editor.getJSON().content?.[0]?.content;
    expect(content?.[0]).toMatchObject({
      text: "Alpha",
      marks: expect.arrayContaining([
        { type: "bold" },
        { type: "fontFamily", attrs: { family: "georgia" } },
        { type: "fontSize", attrs: { points: 18 } },
      ]),
    });
    expect(content?.[1]).toMatchObject({ text: " Beta" });
    editor.destroy();
  });

  it("round-trips family and size through editor JSON", () => {
    const editor = createEditor("Round trip");
    editor.commands.selectAll();
    editor.chain()
      .setMark("fontFamily", { family: "times_new_roman" })
      .setMark("fontSize", { points: 13 })
      .run();
    const snapshot = editor.getJSON();
    const restored = createEditor(snapshot);

    expect(restored.getJSON()).toEqual(snapshot);
    expect(restored.getHTML()).toContain('data-draft-font-family="times_new_roman"');
    expect(restored.getHTML()).toContain("font-size: 13pt");
    editor.destroy();
    restored.destroy();
  });

  it("applies at a collapsed selection and removes marks back to defaults", () => {
    const editor = createEditor("Text");
    editor.commands.setTextSelection(5);
    editor.chain()
      .setMark("fontFamily", { family: "arial" })
      .setMark("fontSize", { points: 12 })
      .insertContent(" added")
      .run();
    editor.commands.setTextSelection({ from: 5, to: 11 });
    editor.chain().unsetMark("fontFamily").unsetMark("fontSize").run();

    const content = editor.getJSON().content?.[0]?.content as Array<{
      marks?: unknown[];
      text?: string;
    }> | undefined;
    const added = content?.find((node) => node.text === " added");
    expect(added?.marks).toBeUndefined();
    editor.destroy();
  });

  it("ignores pasted font CSS and accepts only canonical DRAFT attributes", () => {
    const pasted = createEditor(
      '<p><span style="font-family: Comic Sans MS; font-size: 99pt">Pasted</span></p>',
    );
    const canonical = createEditor(
      '<p><span data-draft-font-family="arial">Family</span>' +
        '<span data-draft-font-size="12">Size</span></p>',
    );

    expect(pasted.getJSON().content?.[0]?.content?.[0]?.marks).toBeUndefined();
    expect(canonical.getJSON().content?.[0]?.content?.[0]?.marks).toEqual([
      { type: "fontFamily", attrs: { family: "arial" } },
    ]);
    expect(canonical.getJSON().content?.[0]?.content?.[1]?.marks).toEqual([
      { type: "fontSize", attrs: { points: 12 } },
    ]);
    pasted.destroy();
    canonical.destroy();
  });
});

function createEditor(content: string | Record<string, unknown>) {
  return new Editor({
    content,
    extensions: [StarterKit, FontFamilyMark, FontSizeMark],
  });
}
