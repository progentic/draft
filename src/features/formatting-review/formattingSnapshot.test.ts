import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { describe, expect, it } from "vitest";

import { CitationNode } from "../../editor/CitationNode";
import {
  applyFormattingHeadingLevel,
  collectFormattingSnapshot,
  formattingTarget,
  inspectFormattingTarget,
} from "./formattingSnapshot";

describe("formatting snapshot", () => {
  it("collects ordered headings and citations without editor content in targets", () => {
    const editor = createEditor(documentWithCitation());
    const collection = collectFormattingSnapshot(editor, "mla9");

    expect(collection).toEqual({
      status: "ready",
      snapshot: expect.objectContaining({
        request: {
          style: "mla9",
          headings: [{ level: 2, title: "Introduction" }],
          citations: [{ citekey: "smith2025", renderStyle: "apa7" }],
        },
      }),
    });
    editor.destroy();
  });

  it("inspects and applies only to the current captured heading", () => {
    const editor = createEditor(documentWithHeading());
    const snapshot = readySnapshot(editor);
    const target = formattingTarget(snapshot, { type: "heading", index: 0 });

    expect(target).toBeDefined();
    expect(inspectFormattingTarget(editor, target!)).toBe(true);
    expect(editor.state.selection.from).toBe(target!.position + 1);
    expect(applyFormattingHeadingLevel(editor, target!, 1)).toBe(true);
    expect(editor.state.doc.nodeAt(target!.position)?.attrs.level).toBe(1);
    editor.destroy();
  });

  it("rejects a target whose node was removed", () => {
    const editor = createEditor(documentWithHeading());
    const target = firstHeadingTarget(editor);
    const node = editor.state.doc.nodeAt(target.position)!;

    editor.view.dispatch(editor.state.tr.delete(target.position, target.position + node.nodeSize));

    expect(inspectFormattingTarget(editor, target)).toBe(false);
    expect(applyFormattingHeadingLevel(editor, target, 1)).toBe(false);
    editor.destroy();
  });

  it("rejects a target whose captured position now addresses another node", () => {
    const editor = createEditor(documentWithHeading());
    const target = firstHeadingTarget(editor);

    editor.commands.insertContentAt(0, { type: "paragraph", content: [{ type: "text", text: "New" }] });

    expect(inspectFormattingTarget(editor, target)).toBe(false);
    expect(applyFormattingHeadingLevel(editor, target, 1)).toBe(false);
    editor.destroy();
  });
});

function createEditor(content: Record<string, unknown>) {
  return new Editor({
    extensions: [
      StarterKit.configure({ heading: { levels: [1, 2, 3] } }),
      CitationNode.configure({
        resolveCitation: async (attrs) => ({
          status: "resolved",
          citation: {
            schemaVersion: 1,
            citekey: attrs.citekey,
            renderStyle: "apa7",
            displayMarker: `[@${attrs.citekey}]`,
          },
        }),
      }),
    ],
    content,
  });
}

function readySnapshot(editor: Editor) {
  const collection = collectFormattingSnapshot(editor, "apa7");
  if (collection.status !== "ready") {
    throw new Error("expected a valid formatting snapshot");
  }
  return collection.snapshot;
}

function firstHeadingTarget(editor: Editor) {
  return readySnapshot(editor).headings[0]!;
}

function documentWithHeading() {
  return {
    type: "doc",
    content: [{ type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Introduction" }] }],
  };
}

function documentWithCitation() {
  return {
    type: "doc",
    content: [
      ...documentWithHeading().content,
      {
        type: "paragraph",
        content: [{
          type: "citation",
          attrs: { schema_version: 1, citekey: "smith2025", render_style: "apa7" },
        }],
      },
    ],
  };
}
