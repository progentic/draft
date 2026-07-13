import type { Editor } from "@tiptap/react";
import { BookOpen, Plus, Quote, X } from "lucide-react";
import { useEffect, useState } from "react";

import {
  type AddReferenceInput,
  type ReferenceLibraryClientError,
  type ReferenceSummary,
} from "../../ipc/referenceLibrary";
import { addReference } from "../../ipc/referenceLibraryAdd";
import { listReferences } from "../../ipc/referenceLibraryList";

interface ReferenceLibraryPanelProps {
  editor: Editor | null;
  isOpen: boolean;
  onClose: () => void;
}

interface FormState {
  author: string;
  citekey: string;
  title: string;
  year: string;
}

const EMPTY_FORM: FormState = { author: "", citekey: "", title: "", year: "" };

export function ReferenceLibraryPanel(props: ReferenceLibraryPanelProps) {
  const [references, setReferences] = useState<ReferenceSummary[]>([]);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [status, setStatus] = useState("Open this panel to load your references.");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (props.isOpen) {
      void loadReferences(setReferences, setStatus, setBusy);
    }
  }, [props.isOpen]);

  const submit = async () => {
    const input = referenceInput(form);
    if (!input) {
      setStatus("Enter a citekey, title, author, and four-digit year.");
      return;
    }
    setBusy(true);
    setStatus("Adding reference.");
    const result = await addReference(input);
    setBusy(false);
    if (result.status === "error") {
      setStatus(referenceFailureMessage(result.error));
      return;
    }
    setReferences((current) => sortedReferences([...current, result.value]));
    setForm(EMPTY_FORM);
    setStatus(`Reference ${result.value.citekey} added.`);
  };

  return (
    <section
      id="reference-library-panel"
      className="workflow-panel"
      aria-labelledby="reference-library-title"
      hidden={!props.isOpen}
    >
      <header className="workflow-panel__header">
        <div className="workflow-panel__title">
          <BookOpen aria-hidden="true" size={16} />
          <h2 id="reference-library-title">References</h2>
        </div>
        <p>Save a source, then insert its citation at the cursor.</p>
        <button className="icon-button workflow-panel__close" type="button" aria-label="Close references" onClick={props.onClose}>
          <X aria-hidden="true" size={16} />
        </button>
      </header>
      <div className="reference-workspace">
        <ReferenceForm form={form} busy={busy} onChange={setForm} onSubmit={() => void submit()} />
        <ReferenceList editor={props.editor} references={references} busy={busy} setStatus={setStatus} />
      </div>
      <p className="workflow-panel__status" role="status" aria-live="polite" aria-atomic="true">{status}</p>
    </section>
  );
}

function ReferenceForm(props: {
  busy: boolean;
  form: FormState;
  onChange: (form: FormState) => void;
  onSubmit: () => void;
}) {
  return (
    <form className="reference-form" onSubmit={(event) => { event.preventDefault(); props.onSubmit(); }}>
      <h3>Add a reference</h3>
      <label>Citekey<input name="citekey" value={props.form.citekey} maxLength={80} onChange={(event) => props.onChange({ ...props.form, citekey: event.target.value })} /></label>
      <label>Title<input name="title" value={props.form.title} maxLength={500} onChange={(event) => props.onChange({ ...props.form, title: event.target.value })} /></label>
      <label>Author<input name="author" value={props.form.author} maxLength={200} onChange={(event) => props.onChange({ ...props.form, author: event.target.value })} /></label>
      <label>Year<input name="year" type="number" min="1000" max="9999" value={props.form.year} onChange={(event) => props.onChange({ ...props.form, year: event.target.value })} /></label>
      <button className="command-button command-button--primary" type="submit" disabled={props.busy}>
        <Plus aria-hidden="true" size={14} /> Add reference
      </button>
    </form>
  );
}

function ReferenceList(props: {
  busy: boolean;
  editor: Editor | null;
  references: ReferenceSummary[];
  setStatus: (status: string) => void;
}) {
  return (
    <div className="reference-list-section">
      <h3>Saved references</h3>
      {props.references.length === 0 ? (
        <p className="workflow-empty">No references saved yet.</p>
      ) : (
        <ul className="reference-list">
          {props.references.map((reference) => (
            <li key={reference.citekey}>
              <div><strong>{reference.title}</strong><span>{reference.citekey}</span></div>
              <button
                className="command-button"
                type="button"
                disabled={props.busy || !props.editor}
                aria-label={`Insert citation for ${reference.title}`}
                onClick={() => insertCitation(props.editor, reference, props.setStatus)}
              >
                <Quote aria-hidden="true" size={14} /> Insert citation
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

async function loadReferences(
  setReferences: (references: ReferenceSummary[]) => void,
  setStatus: (status: string) => void,
  setBusy: (busy: boolean) => void,
) {
  setBusy(true);
  setStatus("Loading references.");
  const result = await listReferences();
  setBusy(false);
  if (result.status === "error") {
    setStatus(referenceFailureMessage(result.error));
    return;
  }
  setReferences(sortedReferences(result.value));
  setStatus(result.value.length === 0 ? "No references saved yet." : `${result.value.length} references ready.`);
}

function insertCitation(
  editor: Editor | null,
  reference: ReferenceSummary,
  setStatus: (status: string) => void,
) {
  const inserted = editor?.chain().focus().insertContent({
    type: "citation",
    attrs: { schema_version: 1, citekey: reference.citekey, render_style: "apa7" },
  }).run();
  setStatus(inserted ? `Citation ${reference.citekey} inserted.` : "Place the cursor in the document and try again.");
}

function referenceInput(form: FormState): AddReferenceInput | null {
  const year = Number(form.year);
  if (![form.citekey, form.title, form.author].every((value) => value.trim()) || !Number.isInteger(year) || year < 1000 || year > 9999) {
    return null;
  }
  return { citekey: form.citekey.trim(), title: form.title.trim(), author: form.author.trim(), year };
}

function sortedReferences(references: ReferenceSummary[]) {
  return [...references].sort((left, right) => left.citekey.localeCompare(right.citekey));
}

function referenceFailureMessage(error: ReferenceLibraryClientError) {
  if (error.type === "command" && error.code === "duplicate_citekey") {
    return "That citekey is already in use. Choose a different citekey.";
  }
  if (error.type === "command" && error.code === "invalid_reference") {
    return "The reference details are not valid. Review each field and try again.";
  }
  return "DRAFT could not update the reference library. Try again.";
}
