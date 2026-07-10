import { Node } from "@tiptap/core";

import {
  validateCitationNodeAttributes,
  type CitationNodeAttributes,
} from "../citations/citationNode";
import {
  resolveCitation,
  type CitationResolutionResult,
} from "../ipc/citationResolution";

export type CitationResolver = (
  attrs: CitationNodeAttributes,
) => Promise<CitationResolutionResult>;

interface CitationNodeOptions {
  resolveCitation: CitationResolver;
}

export interface CitationPresentation {
  dom: HTMLSpanElement;
  update: (attrs: unknown) => void;
  destroy: () => void;
}

interface MutableCitationPresentation {
  dom: HTMLSpanElement;
  revision: number;
  destroyed: boolean;
}

export const CitationNode = Node.create<CitationNodeOptions>({
  name: "citation",
  inline: true,
  group: "inline",
  atom: true,
  marks: "",
  selectable: true,

  addOptions() {
    return { resolveCitation };
  },

  addAttributes() {
    return {
      schema_version: { default: null },
      citekey: { default: null },
      render_style: { default: null },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-draft-citation]", getAttrs: readCitationElement }];
  },

  renderHTML({ node }) {
    return citationHtml(node.attrs);
  },

  addNodeView() {
    return ({ node }) => {
      const presentation = createCitationPresentation(node.attrs, this.options.resolveCitation);
      return {
        dom: presentation.dom,
        update: (nextNode) => {
          if (nextNode.type.name !== this.name) {
            return false;
          }
          presentation.update(nextNode.attrs);
          return true;
        },
        destroy: presentation.destroy,
      };
    };
  },
});

export function createCitationPresentation(
  attrs: unknown,
  resolver: CitationResolver,
): CitationPresentation {
  const presentation = createMutablePresentation();
  updatePresentation(presentation, attrs, resolver);
  return {
    dom: presentation.dom,
    update: (nextAttrs) => updatePresentation(presentation, nextAttrs, resolver),
    destroy: () => destroyPresentation(presentation),
  };
}

function createMutablePresentation(): MutableCitationPresentation {
  const dom = document.createElement("span");
  dom.dataset.draftCitation = "";
  dom.contentEditable = "false";
  dom.setAttribute("role", "note");
  return { dom, revision: 0, destroyed: false };
}

function updatePresentation(
  presentation: MutableCitationPresentation,
  attrs: unknown,
  resolver: CitationResolver,
) {
  const revision = ++presentation.revision;
  const validation = validateCitationNodeAttributes(attrs);
  if (!validation.valid) {
    renderCitationState(presentation.dom, "invalid", "Invalid citation");
    return;
  }
  renderCitationState(presentation.dom, "resolving", "Resolving citation", validation.attrs);
  void resolvePresentation(presentation, revision, validation.attrs, resolver);
}

async function resolvePresentation(
  presentation: MutableCitationPresentation,
  revision: number,
  attrs: CitationNodeAttributes,
  resolver: CitationResolver,
) {
  const result = await resolver(attrs);
  if (presentation.destroyed || presentation.revision !== revision) {
    return;
  }
  renderResolutionResult(presentation.dom, attrs, result);
}

function renderResolutionResult(
  dom: HTMLSpanElement,
  attrs: CitationNodeAttributes,
  result: CitationResolutionResult,
) {
  if (result.status === "resolved") {
    renderCitationState(dom, "resolved", result.citation.displayMarker, attrs);
    return;
  }
  const state = isMissingReference(result) ? "unavailable" : "failed";
  renderCitationState(dom, state, "Citation unavailable", attrs);
}

function isMissingReference(result: CitationResolutionResult) {
  return (
    result.status === "error" &&
    result.error.type === "command" &&
    result.error.error.code === "reference_not_found"
  );
}

function renderCitationState(
  dom: HTMLSpanElement,
  state: string,
  label: string,
  attrs?: CitationNodeAttributes,
) {
  dom.dataset.citationState = state;
  dom.textContent = label;
  dom.setAttribute("aria-label", label);
  updateCitationData(dom, attrs);
}

function updateCitationData(dom: HTMLSpanElement, attrs?: CitationNodeAttributes) {
  if (attrs === undefined) {
    delete dom.dataset.citekey;
    delete dom.dataset.renderStyle;
    delete dom.dataset.schemaVersion;
    return;
  }
  dom.dataset.citekey = attrs.citekey;
  dom.dataset.renderStyle = attrs.render_style;
  dom.dataset.schemaVersion = String(attrs.schema_version);
}

function destroyPresentation(presentation: MutableCitationPresentation) {
  presentation.destroyed = true;
  presentation.revision += 1;
}

function citationHtml(attrs: unknown): [string, Record<string, string>, string] {
  const validation = validateCitationNodeAttributes(attrs);
  if (!validation.valid) {
    return ["span", citationHtmlAttributes("invalid"), "Invalid citation"];
  }
  return [
    "span",
    citationHtmlAttributes("requires-resolution", validation.attrs),
    "Citation requires resolution",
  ];
}

function citationHtmlAttributes(state: string, attrs?: CitationNodeAttributes) {
  const attributes: Record<string, string> = {
    "data-draft-citation": "",
    "data-citation-state": state,
  };
  if (attrs !== undefined) {
    attributes["data-schema-version"] = String(attrs.schema_version);
    attributes["data-citekey"] = attrs.citekey;
    attributes["data-render-style"] = attrs.render_style;
  }
  return attributes;
}

function readCitationElement(element: HTMLElement) {
  return {
    schema_version: readSchemaVersion(element.dataset.schemaVersion),
    citekey: element.dataset.citekey ?? null,
    render_style: element.dataset.renderStyle ?? null,
  };
}

function readSchemaVersion(value: string | undefined) {
  return value === undefined ? null : Number(value);
}
