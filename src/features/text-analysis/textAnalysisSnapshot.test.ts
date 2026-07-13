import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { afterEach, describe, expect, it } from "vitest";

import { createTextAnalysisSnapshot } from "./textAnalysisSnapshot";

let editor: Editor | null = null;

afterEach(() => {
  editor?.destroy();
  editor = null;
});

describe("text analysis snapshot", () => {
  it("maps UTF-8 byte ranges back to editor positions", () => {
    editor = new Editor({
      extensions: [StarterKit],
      content: "<p>Café café.</p><p>Next line.</p>",
    });
    const snapshot = createTextAnalysisSnapshot(editor);
    const startByte = new TextEncoder().encode("Café ").length;
    const endByte = new TextEncoder().encode("Café café").length;
    const range = snapshot.locate(startByte, endByte);

    expect(snapshot.text).toBe("Café café.\nNext line.");
    expect(snapshot.excerpt(startByte, endByte)).toBe("café");
    expect(range).not.toBeNull();
    expect(editor.state.doc.textBetween(range!.from, range!.to)).toBe("café");
  });

  it("returns no location for a range outside document text", () => {
    editor = new Editor({ extensions: [StarterKit], content: "<p>Draft</p>" });
    const snapshot = createTextAnalysisSnapshot(editor);

    expect(snapshot.locate(50, 60)).toBeNull();
  });
});
