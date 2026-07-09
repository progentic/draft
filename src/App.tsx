import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";

const INITIAL_DOCUMENT = `
  <h1>Untitled document</h1>
  <p>Begin writing here.</p>
`;

export function App() {
  const editor = useDraftEditor();

  return (
    <main className="workspace">
      <header className="workspace-header">
        <strong>DRAFT</strong>
        <span>Local document</span>
      </header>
      <EditorContent className="document-surface" editor={editor} />
    </main>
  );
}

function useDraftEditor() {
  return useEditor({
    content: INITIAL_DOCUMENT,
    extensions: [StarterKit],
  });
}
