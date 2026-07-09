import type { Editor } from "@tiptap/react";
import { ListTree } from "lucide-react";
import { useEditorState } from "@tiptap/react";

interface DocumentOutlineProps {
  editor: Editor | null;
  isOpen: boolean;
}

interface OutlineEntry {
  level: number;
  position: number;
  text: string;
}

export function DocumentOutline(props: DocumentOutlineProps) {
  const entries = useDocumentOutline(props.editor);

  return (
    <aside
      id="document-outline"
      className="outline-panel"
      aria-label="Document outline"
      aria-hidden={!props.isOpen}
      inert={!props.isOpen}
    >
      <div className="panel-heading">
        <ListTree aria-hidden="true" size={16} strokeWidth={1.8} />
        <h2>Outline</h2>
      </div>
      <OutlineNavigation editor={props.editor} entries={entries} />
    </aside>
  );
}

function OutlineNavigation(props: { editor: Editor | null; entries: OutlineEntry[] }) {
  return (
    <nav className="outline-navigation" aria-label="Document headings">
      {props.entries.length === 0 ? (
        <p className="panel-empty-state">No headings</p>
      ) : (
        props.entries.map((entry) => (
          <OutlineButton editor={props.editor} entry={entry} key={outlineKey(entry)} />
        ))
      )}
    </nav>
  );
}

function OutlineButton(props: { editor: Editor | null; entry: OutlineEntry }) {
  return (
    <button
      className="outline-entry"
      style={{ paddingInlineStart: outlineIndent(props.entry.level) }}
      type="button"
      onClick={() => focusHeading(props.editor, props.entry.position)}
    >
      <span className="outline-entry__level">H{props.entry.level}</span>
      <span className="outline-entry__text">{props.entry.text}</span>
    </button>
  );
}

function outlineKey(entry: OutlineEntry) {
  return `${entry.position}-${entry.text}`;
}

function useDocumentOutline(editor: Editor | null) {
  return (
    useEditorState({
      editor,
      selector: ({ editor: currentEditor }) =>
        currentEditor ? collectHeadings(currentEditor) : [],
    }) ?? []
  );
}

function collectHeadings(editor: Editor) {
  const headings: OutlineEntry[] = [];

  editor.state.doc.descendants((node, position) => {
    if (node.type.name !== "heading") {
      return;
    }

    headings.push({
      level: Number(node.attrs.level),
      position,
      text: node.textContent || "Untitled heading",
    });
  });

  return headings;
}

function focusHeading(editor: Editor | null, position: number) {
  editor?.chain().focus().setTextSelection(position + 1).run();
}

function outlineIndent(level: number) {
  return `${12 + Math.max(0, level - 1) * 14}px`;
}
