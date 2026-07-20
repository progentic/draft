import { Node } from "@tiptap/core";

export const PageBreakNode = Node.create({
  name: "pageBreak",
  group: "block",
  atom: true,
  selectable: true,

  parseHTML() {
    return [{ tag: "div[data-draft-page-break]" }];
  },

  renderHTML() {
    return [
      "div",
      {
        "aria-label": "Page break",
        "data-draft-page-break": "",
        role: "separator",
      },
    ];
  },
});
