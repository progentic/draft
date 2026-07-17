import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { describe, expect, it } from "vitest";

import { PageBreakNode } from "./PageBreakNode";

describe("PageBreakNode", () => {
  it("preserves the canonical block in JSON and HTML", () => {
    const editor = new Editor({
      extensions: [StarterKit, PageBreakNode],
      content: {
        type: "doc",
        content: [
          { type: "paragraph", content: [{ type: "text", text: "Before" }] },
          { type: "pageBreak" },
          { type: "paragraph", content: [{ type: "text", text: "After" }] },
        ],
      },
    });

    expect(editor.getJSON().content?.[1]).toEqual({ type: "pageBreak" });
    expect(editor.getHTML()).toContain('data-draft-page-break=""');
    expect(editor.getHTML()).toContain('aria-label="Page break"');
    editor.destroy();
  });

  it("accepts only the canonical DRAFT page-break attribute", () => {
    const editor = new Editor({
      extensions: [StarterKit, PageBreakNode],
      content: '<div style="break-before: page"></div><div data-draft-page-break></div>',
    });

    expect(editor.getJSON().content).toEqual([{ type: "pageBreak" }]);
    editor.destroy();
  });
});
