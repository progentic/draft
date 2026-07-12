import { Mark, mergeAttributes } from "@tiptap/core";

import {
  fontFamilyCss,
  isFontFamilyId,
  isFontSizePoints,
} from "../documents/textFormatting";

export const FontFamilyMark = Mark.create({
  name: "fontFamily",

  addAttributes() {
    return {
      family: {
        default: null,
        parseHTML: (element) => familyFromElement(element),
        renderHTML: (attributes) => familyStyle(attributes.family),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-draft-font-family]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes), 0];
  },
});

export const FontSizeMark = Mark.create({
  name: "fontSize",

  addAttributes() {
    return {
      points: {
        default: null,
        parseHTML: (element) => sizeFromElement(element),
        renderHTML: (attributes) => sizeStyle(attributes.points),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-draft-font-size]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes), 0];
  },
});

function familyFromElement(element: HTMLElement) {
  const family = element.dataset.draftFontFamily;
  return isFontFamilyId(family) ? family : null;
}

function familyStyle(value: unknown) {
  const css = fontFamilyCss(value);
  return css && isFontFamilyId(value)
    ? { "data-draft-font-family": value, style: `font-family: ${css}` }
    : {};
}

function sizeFromElement(element: HTMLElement) {
  const points = Number(element.dataset.draftFontSize);
  return isFontSizePoints(points) ? points : null;
}

function sizeStyle(value: unknown) {
  return isFontSizePoints(value)
    ? { "data-draft-font-size": String(value), style: `font-size: ${value}pt` }
    : {};
}
