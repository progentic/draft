// @vitest-environment jsdom

import { Editor, type JSONContent } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { bench, describe } from "vitest";

import { CitationNode } from "./CitationNode";

const PARAGRAPH_COUNT = 1_000;
const LARGE_DOCUMENT = freezeDocument(largeDocument(PARAGRAPH_COUNT));
const BENCHMARK_OPTIONS = {
  iterations: 10,
  time: 500,
  warmupIterations: 2,
  warmupTime: 100,
};

describe("Draft editor startup measurement", () => {
  bench(
    "starts and destroys a 1,000-paragraph document",
    () => {
      const editor = createEditor(LARGE_DOCUMENT);
      editor.destroy();
    },
    BENCHMARK_OPTIONS,
  );
});

function createEditor(content: JSONContent) {
  return new Editor({
    content,
    extensions: [
      StarterKit.configure({
        heading: { levels: [1, 2, 3] },
      }),
      CitationNode,
    ],
  });
}

function largeDocument(paragraphCount: number): JSONContent {
  return {
    type: "doc",
    content: Array.from({ length: paragraphCount }, (_, index) => ({
      type: "paragraph",
      content: [
        {
          type: "text",
          text: `Paragraph ${index + 1}: measured DRAFT editor content for repeatable startup comparison.`,
        },
      ],
    })),
  };
}

function freezeDocument(node: JSONContent): JSONContent {
  node.content?.forEach(freezeDocument);
  if (node.content !== undefined) {
    Object.freeze(node.content);
  }
  return Object.freeze(node);
}
