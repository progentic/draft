import { Extension } from "@tiptap/core";

import {
  parseParagraphStyle,
  type ParagraphStyle,
} from "../documents/paragraphFormatting";

const PARAGRAPH_STYLE_ATTRIBUTE = "data-draft-paragraph-style";

export const ParagraphFormatting = Extension.create({
  name: "paragraphFormatting",

  addGlobalAttributes() {
    return [{
      types: ["paragraph", "heading"],
      attributes: {
        paragraphStyle: {
          default: null,
          parseHTML: parseStyleAttribute,
          renderHTML: renderStyleAttribute,
        },
      },
    }];
  },
});

function parseStyleAttribute(element: HTMLElement): ParagraphStyle | null {
  const serialized = element.getAttribute(PARAGRAPH_STYLE_ATTRIBUTE);
  if (!serialized) {
    return null;
  }
  try {
    return parseParagraphStyle(JSON.parse(serialized));
  } catch {
    return null;
  }
}

function renderStyleAttribute(attributes: Record<string, unknown>) {
  const style = parseParagraphStyle(attributes.paragraphStyle);
  if (!style) {
    return {};
  }
  return {
    [PARAGRAPH_STYLE_ATTRIBUTE]: JSON.stringify(style),
    style: paragraphCss(style),
  };
}

function paragraphCss(style: ParagraphStyle): string {
  return [
    `text-align: ${style.alignment}`,
    `line-height: ${style.lineSpacingHundredths / 100}`,
    `margin-top: ${twipsToPoints(style.spaceBeforeTwips)}pt`,
    `margin-bottom: ${twipsToPoints(style.spaceAfterTwips)}pt`,
    `margin-left: ${twipsToPoints(style.leftIndentTwips)}pt`,
    `margin-right: ${twipsToPoints(style.rightIndentTwips)}pt`,
    `text-indent: ${specialIndentPoints(style)}pt`,
  ].join("; ");
}

function specialIndentPoints(style: ParagraphStyle): number {
  const points = twipsToPoints(style.specialIndent.twips);
  return style.specialIndent.kind === "hanging" ? -points : points;
}

function twipsToPoints(twips: number): number {
  return twips / 20;
}
