import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { waitFor } from "@testing-library/dom";
import { describe, expect, it, vi } from "vitest";

import {
  CitationNode,
  createCitationPresentation,
  type CitationResolver,
} from "./CitationNode";

describe("CitationNode", () => {
  it("preserves exact attrs in Tiptap JSON", () => {
    const editor = createEditor(resolvedCitation);

    expect(editor.getJSON()).toEqual(documentWith(validAttrs()));
    editor.destroy();
  });

  it("uses an explicit unresolved static HTML state", () => {
    const editor = createEditor(resolvedCitation);

    expect(editor.getHTML()).toContain('data-citation-state="requires-resolution"');
    expect(editor.getHTML()).toContain("Citation requires resolution");
    expect(editor.getHTML()).not.toContain("[@smith2025]");
    editor.destroy();
  });

  it("shows invalid attrs without calling Rust", () => {
    const resolver = vi.fn<CitationResolver>();
    const presentation = createCitationPresentation({ citekey: "smith2025" }, resolver);

    expect(presentation.dom.dataset.citationState).toBe("invalid");
    expect(presentation.dom.textContent).toBe("Invalid citation");
    expect(resolver).not.toHaveBeenCalled();
  });

  it("shows the marker only after successful resolution", async () => {
    const presentation = createCitationPresentation(validAttrs(), resolvedCitation);

    expect(presentation.dom.dataset.citationState).toBe("resolving");
    await waitFor(() => expect(presentation.dom.dataset.citationState).toBe("resolved"));
    expect(presentation.dom.textContent).toBe("[@smith2025]");
  });

  it("distinguishes missing references from other failures", async () => {
    const missing = createCitationPresentation(validAttrs(), async () => ({
      status: "error",
      error: { type: "command", error: { code: "reference_not_found" } },
    }));
    const failed = createCitationPresentation(validAttrs(), async () => ({
      status: "error",
      error: { type: "transport" },
    }));

    await waitFor(() => expect(missing.dom.dataset.citationState).toBe("unavailable"));
    await waitFor(() => expect(failed.dom.dataset.citationState).toBe("failed"));
    expect(missing.dom.textContent).toBe("Citation unavailable");
    expect(failed.dom.textContent).toBe("Citation unavailable");
  });

  it("ignores stale resolution after attrs change", async () => {
    const first = deferredResolution();
    const resolver = vi.fn<CitationResolver>((attrs) =>
      attrs.citekey === "smith2025" ? first.promise : Promise.resolve(resolved("jones2026")),
    );
    const presentation = createCitationPresentation(validAttrs(), resolver);

    presentation.update(validAttrs("jones2026"));
    await waitFor(() => expect(presentation.dom.textContent).toBe("[@jones2026]"));
    first.resolve(resolved("smith2025"));
    await Promise.resolve();

    expect(presentation.dom.textContent).toBe("[@jones2026]");
  });
});

const resolvedCitation: CitationResolver = async (attrs) => resolved(attrs.citekey);

function createEditor(resolver: CitationResolver) {
  return new Editor({
    extensions: [StarterKit, CitationNode.configure({ resolveCitation: resolver })],
    content: documentWith(validAttrs()),
  });
}

function resolved(citekey: string) {
  return {
    status: "resolved" as const,
    citation: {
      schemaVersion: 1 as const,
      citekey,
      renderStyle: "apa7" as const,
      displayMarker: `[@${citekey}]`,
    },
  };
}

function validAttrs(citekey = "smith2025") {
  return { schema_version: 1 as const, citekey, render_style: "apa7" as const };
}

function documentWith(attrs: ReturnType<typeof validAttrs>) {
  return {
    type: "doc",
    content: [{
      type: "paragraph",
      content: [{ type: "citation", attrs }],
    }],
  };
}

function deferredResolution() {
  let resolve!: (value: ReturnType<typeof resolved>) => void;
  const promise = new Promise<ReturnType<typeof resolved>>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}
