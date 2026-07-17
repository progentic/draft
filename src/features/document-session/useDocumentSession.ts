import type { Editor, JSONContent } from "@tiptap/react";
import { omitUnsetParagraphStyles } from "../../documents/paragraphFormatting";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { closeDocument } from "../../ipc/documentClose";
import { createUnsavedDocument } from "../../ipc/documentCreate";
import {
  dismissApplicationOpenRequest,
  listenToApplicationOpenRequests,
  takeApplicationOpenRequest,
  type ApplicationOpenClientError,
} from "../../ipc/applicationOpen";
import type { DocumentEnvelopeSnapshot } from "../../ipc/documentEnvelope";
import { openDocument } from "../../ipc/documentOpen";
import { saveDocument } from "../../ipc/documentSave";
import type { SaveDocumentMode, SaveDocumentOrigin } from "../../ipc/documentSave";
import type { ExternalDocumentSummary } from "../../ipc/externalDocument";
import {
  useExternalSourceSave,
  type ExternalSourceSaveConfirmation,
} from "../external-source-save/useExternalSourceSave";

export type PendingDocumentAction = "close" | "new" | "open" | "open_requested";
export type DocumentOperation =
  | "checking_source"
  | "closing"
  | "confirming_source_save"
  | "creating"
  | "opening"
  | "ready"
  | "saving"
  | "saving_source";

type DocumentOrigin = SaveDocumentOrigin;

interface DocumentIdentity {
  displayName: string;
  documentId: string;
  external: ExternalDocumentSummary | null;
  origin: DocumentOrigin;
  persisted: boolean;
  registered: boolean;
  title: string;
}

export interface DocumentSession {
  canClose: boolean;
  canExport: boolean;
  canSave: boolean;
  canSaveAs: boolean;
  canSaveBack: boolean;
  documentId: string | null;
  feedback: string;
  operation: DocumentOperation;
  pendingAction: PendingDocumentAction | null;
  requestClose: () => void;
  requestNew: () => void;
  requestOpen: () => void;
  requestSaveBack: () => void;
  resolvePendingAction: (decision: "cancel" | "discard" | "save") => void;
  resolveSaveBack: (decision: "cancel" | "confirm") => void;
  save: () => Promise<boolean>;
  saveAs: () => Promise<boolean>;
  saveBackConfirmation: ExternalSourceSaveConfirmation | null;
  saveBackUnavailableReason: string;
  saveBackVisible: boolean;
  snapshot: () => DocumentEnvelopeSnapshot | null;
  statusLabel: string;
  title: string;
  unsaved: boolean;
}

const EMPTY_DOCUMENT: JSONContent = { type: "doc", content: [] };

export function useDocumentSession(editor: Editor | null): DocumentSession {
  const [identity, setIdentity] = useState<DocumentIdentity | null>(null);
  const identityRef = useRef(identity);
  const initializationStarted = useRef(false);
  const [dirty, setDirty] = useState(false);
  const [revision, setRevision] = useState(0);
  const [operation, setOperation] = useState<DocumentOperation>("creating");
  const [feedback, setFeedback] = useState("");
  const [pendingAction, setPendingAction] = useState<PendingDocumentAction | null>(null);
  const [applicationOpenPending, setApplicationOpenPending] = useState(false);

  useEffect(
    () => subscribeToDocumentUpdates(editor, setDirty, setIdentity, setRevision),
    [editor],
  );
  useEffect(() => {
    if (editor && !initializationStarted.current) {
      initializationStarted.current = true;
      void initializeDocumentSession(
        editor,
        setIdentity,
        setDirty,
        setOperation,
        setFeedback,
      );
    }
  }, [editor]);
  useEffect(() => {
    identityRef.current = identity;
  }, [identity]);
  useEffect(
    () => attachApplicationOpenListener(setApplicationOpenPending, setFeedback),
    [],
  );

  const snapshot = useCallback(
    () => currentSnapshot(editor, identity),
    [editor, identity],
  );

  const persist = useCallback(async (mode: SaveDocumentMode) => {
    const current = snapshot();
    const currentIdentity = identityRef.current;
    if (!current || !currentIdentity) {
      setFeedback("Create or open a document before saving.");
      return false;
    }
    setOperation("saving");
    const result = await saveDocument(
      current,
      mode,
      currentIdentity.displayName,
      currentIdentity.origin,
    );
    setOperation("ready");
    if (result.status !== "saved") {
      setFeedback(saveFailureMessage(result));
      return false;
    }
    const savedIdentity = {
      displayName: result.displayName,
      documentId: result.documentId,
      external: null,
      origin: "opened_draft" as const,
      persisted: true,
      registered: true,
      title: current.title,
    };
    identityRef.current = savedIdentity;
    setIdentity(savedIdentity);
    setDirty(false);
    setFeedback(`Saved as ${result.displayName}.`);
    return true;
  }, [snapshot]);
  const save = useCallback(() => persist("save"), [persist]);
  const saveAs = useCallback(() => persist("save_as"), [persist]);
  const markExternalSaved = useCallback((documentId: string, displayName: string) => {
    const current = identityRef.current;
    if (!current?.external || current.documentId !== documentId) return;
    const savedIdentity = {
      ...current,
      displayName,
    };
    identityRef.current = savedIdentity;
    setIdentity(savedIdentity);
    setDirty(false);
  }, []);
  const sourceSave = useExternalSourceSave({
    external: identity?.external ?? null,
    modified: identity?.origin === "imported_external" && dirty,
    operation,
    revision,
    snapshot,
    onFeedback: setFeedback,
    onOperation: setOperation,
    onSaved: markExternalSaved,
  });

  const executeAction = useCallback(
    async (action: PendingDocumentAction) => {
      if (action === "new") {
        await createIntoSession(
          editor,
          identityRef.current,
          setIdentity,
          setDirty,
          setOperation,
          setFeedback,
        );
        return;
      }
      if (action === "open") {
        await openIntoSession(
          editor,
          identityRef.current,
          setIdentity,
          setDirty,
          setOperation,
          setFeedback,
        );
        return;
      }
      if (action === "open_requested") {
        await openRequestedIntoSession(
          editor,
          identityRef.current,
          setIdentity,
          setDirty,
          setOperation,
          setFeedback,
        );
        return;
      }
      const closed = await closeCurrent(identityRef.current, setOperation, setFeedback);
      if (!closed) {
        return;
      }
      clearDocument(editor, setIdentity, setDirty, setFeedback);
    },
    [editor],
  );

  const request = useCallback(
    (action: PendingDocumentAction) => {
      if (dirty) {
        setPendingAction(action);
        return;
      }
      void executeAction(action);
    },
    [dirty, executeAction],
  );

  const resolvePendingAction = useCallback(
    (decision: "cancel" | "discard" | "save") => {
      const action = pendingAction;
      setPendingAction(null);
      if (!action) {
        return;
      }
      if (decision === "cancel") {
        if (action === "open_requested") {
          void dismissApplicationOpenRequest().then((dismissed) => {
            setFeedback(
              dismissed
                ? "Requested document open cancelled."
                : "DRAFT could not cancel the requested document. Finish editing, then use Open.",
            );
          });
        }
        return;
      }
      void continuePendingAction(action, decision, save, executeAction).then((completed) => {
        if (!completed && action === "open_requested") {
          setApplicationOpenPending(true);
        }
      });
    },
    [executeAction, pendingAction, save],
  );

  useEffect(() => {
    if (!applicationOpenPending || operation !== "ready" || pendingAction) {
      return;
    }
    setApplicationOpenPending(false);
    request("open_requested");
  }, [applicationOpenPending, operation, pendingAction, request]);

  return useMemo(
    () => ({
      canClose: identity !== null && operation === "ready",
      canExport: identity !== null && operation === "ready",
      canSave: identity !== null && operation === "ready",
      canSaveAs: identity !== null && operation === "ready",
      canSaveBack: sourceSave.enabled,
      documentId: identity?.documentId ?? null,
      feedback,
      operation,
      pendingAction,
      requestClose: () => request("close"),
      requestNew: () => request("new"),
      requestOpen: () => request("open"),
      requestSaveBack: sourceSave.request,
      resolvePendingAction,
      resolveSaveBack: sourceSave.resolve,
      save,
      saveAs,
      saveBackConfirmation: sourceSave.confirmation,
      saveBackUnavailableReason: sourceSave.unavailableReason,
      saveBackVisible: sourceSave.visible,
      snapshot,
      statusLabel: documentStatus(identity, dirty, operation),
      title: identity?.displayName ?? "No document open",
      unsaved: identity !== null && (!identity.persisted || dirty),
    }),
    [dirty, feedback, identity, operation, pendingAction, request, resolvePendingAction, save, saveAs, snapshot, sourceSave],
  );
}

async function continuePendingAction(
  action: PendingDocumentAction,
  decision: "discard" | "save",
  save: () => Promise<boolean>,
  execute: (action: PendingDocumentAction) => Promise<void>,
) {
  if (decision === "save" && !(await save())) {
    return false;
  }
  await execute(action);
  return true;
}

function subscribeToDocumentUpdates(
  editor: Editor | null,
  setDirty: (dirty: boolean) => void,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setRevision: React.Dispatch<React.SetStateAction<number>>,
) {
  if (!editor) {
    return;
  }
  const handleUpdate = () => {
    setDirty(true);
    setRevision((revision) => revision + 1);
    setIdentity((identity) =>
      identity
        ? { ...identity, title: documentTitle(editor.getJSON(), identity.title) }
        : identity,
    );
  };
  editor.on("update", handleUpdate);
  return () => {
    editor.off("update", handleUpdate);
  };
}

async function openIntoSession(
  editor: Editor | null,
  current: DocumentIdentity | null,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  if (!editor) {
    return;
  }
  setOperation("opening");
  setFeedback("Opening document…");
  const result = await openDocument();
  await applyOpenResult(
    result,
    editor,
    current,
    setIdentity,
    setDirty,
    setOperation,
    setFeedback,
  );
}

async function openRequestedIntoSession(
  editor: Editor | null,
  current: DocumentIdentity | null,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  if (!editor) {
    return;
  }
  setOperation("opening");
  setFeedback("Opening requested DRAFT document…");
  const request = await takeApplicationOpenRequest();
  if (request.status === "none") {
    setOperation("ready");
    setFeedback("");
    return;
  }
  if (request.status === "error") {
    setOperation("ready");
    setFeedback(applicationOpenFailureMessage(request.error));
    return;
  }
  await applyOpenResult(
    request.result,
    editor,
    current,
    setIdentity,
    setDirty,
    setOperation,
    setFeedback,
  );
}

async function applyOpenResult(
  result: Awaited<ReturnType<typeof openDocument>>,
  editor: Editor,
  current: DocumentIdentity | null,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  if (!isSuccessfulOpen(result)) {
    setOperation("ready");
    setFeedback(openFailureMessage(result));
    return;
  }
  const persisted = result.status === "opened_draft";
  const registered = result.status !== "imported_text";
  if (
    current?.registered &&
    !(await releaseReplacedDocument(current, result.envelope, registered))
  ) {
    setOperation("ready");
    setFeedback("DRAFT could not replace the open document. Your current document remains open.");
    return;
  }
  loadEnvelope(editor, result.envelope);
  setIdentity({
    displayName:
      result.status === "imported_external"
        ? result.external.displayName
        : result.envelope.title,
    documentId: result.envelope.document_id,
    external: result.status === "imported_external" ? result.external : null,
    origin: result.status,
    persisted,
    registered,
    title: result.envelope.title,
  });
  setDirty(result.status === "imported_text");
  setOperation("ready");
  setFeedback(openSuccessMessage(result));
}

function attachApplicationOpenListener(
  setPending: (pending: boolean) => void,
  setFeedback: (feedback: string) => void,
) {
  let disposed = false;
  let stop: (() => void) | undefined;
  void listenToApplicationOpenRequests(
    () => setPending(true),
    () => setFeedback("DRAFT could not receive the requested document. Use Open instead."),
  ).then((listener) => {
    if (disposed) {
      listener();
      return;
    }
    stop = listener;
    setPending(true);
  }).catch(() => {
    if (!disposed) {
      setFeedback("DRAFT could not receive documents from macOS. Use Open instead.");
    }
  });
  return () => {
    disposed = true;
    stop?.();
  };
}

function applicationOpenFailureMessage(error: ApplicationOpenClientError) {
  if (error.type === "open") {
    return openCommandFailureMessage(error.error);
  }
  if (error.type === "command") {
    if (error.code === "multiple_files_unsupported") {
      return "Open one DRAFT document at a time.";
    }
    if (error.code === "unsupported_file_location") {
      return "DRAFT could not use the requested file location. Use Open instead.";
    }
  }
  return "DRAFT could not open the requested document. Use Open to try again.";
}

function isSuccessfulOpen(
  result: Awaited<ReturnType<typeof openDocument>>,
): result is Extract<
  Awaited<ReturnType<typeof openDocument>>,
  { status: "imported_external" | "imported_text" | "opened_draft" }
> {
  return (
    result.status === "opened_draft" ||
    result.status === "imported_text" ||
    result.status === "imported_external"
  );
}

function openSuccessMessage(result: Extract<
  Awaited<ReturnType<typeof openDocument>>,
  { status: "imported_external" | "imported_text" | "opened_draft" }
>) {
  if (result.status === "opened_draft") {
    return "DRAFT document opened.";
  }
  if (result.status === "imported_text") {
    return result.format === "markdown"
      ? "Markdown imported as literal editable text. DRAFT does not parse or preview Markdown. Save as a DRAFT document to keep your work."
      : "Text imported. Save as a DRAFT document to keep your work.";
  }
  return result.external.fidelity.classification === "unsupported_preservable"
    ? "DOCX imported. Supported text and paragraph formatting was retained. Some source features remain only in the original. Save as a DRAFT document to edit a copy."
    : "DOCX opened. Save creates a DRAFT document; Save Back to Source replaces the DOCX only after confirmation.";
}

async function initializeDocumentSession(
  editor: Editor,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  const result = await createUnsavedDocument();
  if (result.status !== "created") {
    clearDocument(editor, setIdentity, setDirty, setFeedback);
    setOperation("ready");
    setFeedback("DRAFT could not create a document. Choose New to try again.");
    return;
  }
  loadCreatedDocument(editor, result.envelope, setIdentity, setDirty);
  setOperation("ready");
}

async function createIntoSession(
  editor: Editor | null,
  current: DocumentIdentity | null,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  if (!editor) {
    return;
  }
  setOperation("creating");
  const result = await createUnsavedDocument();
  if (result.status !== "created") {
    setOperation("ready");
    setFeedback("DRAFT could not create a document. Try again.");
    return;
  }
  if (current?.registered && !(await releaseCurrentDocument(current))) {
    setOperation("ready");
    setFeedback("DRAFT could not replace the open document. Your current document remains open.");
    return;
  }
  loadCreatedDocument(editor, result.envelope, setIdentity, setDirty);
  setOperation("ready");
  setFeedback("New document ready.");
}

async function releaseCurrentDocument(current: DocumentIdentity) {
  const result = await closeDocument(current.documentId);
  return result.status === "closed";
}

async function releaseReplacedDocument(
  current: DocumentIdentity,
  replacement: DocumentEnvelopeSnapshot,
  replacementRegistered: boolean,
) {
  const closed = await closeDocument(current.documentId);
  if (closed.status === "closed") {
    return true;
  }
  if (replacementRegistered) {
    await closeDocument(replacement.document_id);
  }
  return false;
}

async function closeCurrent(
  identity: DocumentIdentity | null,
  setOperation: (operation: DocumentOperation) => void,
  setFeedback: (feedback: string) => void,
) {
  if (!identity?.registered) {
    return true;
  }
  setOperation("closing");
  const result = await closeDocument(identity.documentId);
  setOperation("ready");
  if (result.status === "closed") {
    return true;
  }
  setFeedback("DRAFT could not close the document. Try again before leaving it.");
  return false;
}

function loadCreatedDocument(
  editor: Editor,
  envelope: DocumentEnvelopeSnapshot,
  setIdentity: React.Dispatch<React.SetStateAction<DocumentIdentity | null>>,
  setDirty: (dirty: boolean) => void,
) {
  loadEnvelope(editor, envelope);
  focusEditorStart(editor);
  setIdentity({
    displayName: envelope.title,
    documentId: envelope.document_id,
    external: null,
    origin: "new",
    persisted: false,
    registered: false,
    title: envelope.title,
  });
  setDirty(false);
}

function clearDocument(
  editor: Editor | null,
  setIdentity: (identity: DocumentIdentity | null) => void,
  setDirty: (dirty: boolean) => void,
  setFeedback: (feedback: string) => void,
) {
  replaceEditorDocument(editor, EMPTY_DOCUMENT);
  editor?.setEditable(false);
  setIdentity(null);
  setDirty(false);
  setFeedback("Document closed.");
}

function loadEnvelope(editor: Editor, envelope: DocumentEnvelopeSnapshot) {
  editor.setEditable(true);
  replaceEditorDocument(editor, envelope.document as JSONContent);
}

function replaceEditorDocument(editor: Editor | null, document: JSONContent) {
  editor
    ?.chain()
    .command(({ tr }) => {
      tr.setMeta("addToHistory", false);
      return true;
    })
    .setContent(document, { emitUpdate: false })
    .run();
}

function currentSnapshot(
  editor: Editor | null,
  identity: DocumentIdentity | null,
): DocumentEnvelopeSnapshot | null {
  if (!editor || !identity) {
    return null;
  }
  const document = omitUnsetParagraphStyles(editor.getJSON());
  return {
    schema_version: 2,
    document_id: identity.documentId,
    title: documentTitle(document, identity.title),
    document: document as DocumentEnvelopeSnapshot["document"],
  };
}

function documentTitle(document: JSONContent, fallback: string) {
  const heading = document.content?.find((node) => node.type === "heading" && node.attrs?.level === 1);
  const text = heading?.content?.map((node) => node.text ?? "").join("").trim();
  return text || fallback;
}

function focusEditorStart(editor: Editor) {
  queueMicrotask(() => editor.commands.focus("start", { scrollIntoView: false }));
}

function documentStatus(
  identity: DocumentIdentity | null,
  dirty: boolean,
  operation: DocumentOperation,
) {
  if (operation !== "ready") {
    if (operation === "checking_source") {
      return "Checking source";
    }
    if (operation === "confirming_source_save") {
      return "Waiting for confirmation";
    }
    if (operation === "saving_source") {
      return "Saving to source";
    }
    if (operation === "saving") {
      return "Saving";
    }
    if (operation === "opening") {
      return "Opening";
    }
    return operation === "creating" ? "Creating" : "Closing";
  }
  if (!identity) {
    return "No document open";
  }
  if (identity.origin === "imported_external") {
    return dirty ? "Source modified" : "Source unchanged";
  }
  if (identity.origin === "imported_text" && !identity.persisted) {
    return "Imported, unsaved";
  }
  if (dirty) {
    return "Unsaved changes";
  }
  return identity.persisted ? "Saved" : "Not saved";
}

function openFailureMessage(result: Awaited<ReturnType<typeof openDocument>>) {
  if (result.status === "cancelled") {
    return "Open cancelled.";
  }
  if (result.status === "error" && result.error.type === "command") {
    return openCommandFailureMessage(result.error.error);
  }
  return "DRAFT could not open the document. Try again.";
}

function openCommandFailureMessage(
  error: import("../../ipc/documentErrors").OpenDocumentCommandError,
) {
  switch (error.code) {
    case "file_not_found":
      return "That file is no longer available. Choose another document.";
    case "unsupported_file_type":
      return "Choose a DRAFT, DOCX, plain-text, or Markdown file.";
    case "invalid_text_encoding":
      return "Text and Markdown files must use UTF-8 encoding.";
    case "text_too_large":
      return "That text file is too large to import. Choose a file no larger than 8 MiB.";
    case "malformed_json":
    case "invalid_envelope":
      return "That DRAFT document is invalid or uses an unsupported format.";
    case "read_failed":
      return "DRAFT could not read that file. Check access and try again.";
    case "registry":
      return "That DRAFT document is already open or unavailable.";
    case "unsupported_file_location":
      return "DRAFT could not use that file location. Choose another file.";
    case "external_import":
      return externalImportFailureMessage(error.cause);
  }
}

function externalImportFailureMessage(
  error: import("../../ipc/documentErrors").ExternalDocumentImportError,
) {
  switch (error.code) {
    case "file_not_found":
      return "That DOCX file is no longer available. Choose another document.";
    case "read_failed":
      return "DRAFT could not read that DOCX file. Check access and try again.";
    case "package_too_large":
      return "That DOCX file is too large to import. Choose a file no larger than 16 MiB.";
    case "invalid_canonical_document":
      return "DRAFT could not convert that DOCX file safely. The original was not changed.";
    case "docx":
      return docxImportFailureMessage(error.cause.code);
  }
}

function docxImportFailureMessage(
  code: import("../../ipc/documentErrors").DocxImportError["code"],
) {
  switch (code) {
    case "malformed_package":
      return "That DOCX file is malformed. The original was not changed.";
    case "unsafe_package":
      return "That DOCX exceeds DRAFT’s supported package, XML, or document-size limits. The original was not changed. Try a smaller document or remove large embedded content.";
    case "unsupported_external_feature":
      return "That DOCX file contains structure DRAFT cannot import safely. The original was not changed.";
    case "lossy_import_denied":
      return "Importing that DOCX file would change unsupported formatting. The original was not changed.";
  }
}

function saveFailureMessage(result: Awaited<ReturnType<typeof saveDocument>>) {
  if (result.status === "cancelled") {
    return "Save cancelled. Your document remains unsaved.";
  }
  if (
    result.status === "error" &&
    result.error.type === "command" &&
    result.error.error.code === "invalid_target"
  ) {
    return "Choose a .draft file name. Your document remains unsaved.";
  }
  return "DRAFT could not save the document. Your open document has not been replaced.";
}
