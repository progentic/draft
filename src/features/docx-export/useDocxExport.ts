import { useCallback, useState } from "react";

import type { DocumentSession } from "../document-session/useDocumentSession";
import {
  exportDocument,
  type DocxExportErrorCode,
  type ExportDocumentClientError,
} from "../../ipc/docxExport";

export interface DocxExportState {
  disabled: boolean;
  feedback: string;
  label: string;
  run: () => void;
}

export function useDocxExport(session: DocumentSession): DocxExportState {
  const [exporting, setExporting] = useState(false);
  const [feedback, setFeedback] = useState("");

  const run = useCallback(() => {
    const snapshot = session.snapshot();
    if (!snapshot) {
      setFeedback("Create or open a document before exporting.");
      return;
    }
    void performExport(snapshot, setExporting, setFeedback);
  }, [session]);

  return {
    disabled: exporting,
    feedback,
    label: exporting ? "Exporting DOCX" : "Export DOCX",
    run,
  };
}

async function performExport(
  snapshot: NonNullable<ReturnType<DocumentSession["snapshot"]>>,
  setExporting: (exporting: boolean) => void,
  setFeedback: (feedback: string) => void,
) {
  setExporting(true);
  setFeedback("Preparing DOCX export.");
  const result = await exportDocument(snapshot);
  setExporting(false);
  if (result.status === "exported") {
    setFeedback("DOCX export complete. Your DRAFT document was not changed.");
    return;
  }
  if (result.status === "cancelled") {
    setFeedback("DOCX export cancelled.");
    return;
  }
  setFeedback(exportFailureMessage(result.error));
}

function exportFailureMessage(error: ExportDocumentClientError) {
  if (error.type !== "command") {
    return "DRAFT could not export this document. Try again.";
  }
  if (error.code === "unsupported_file_location") {
    return "Choose a writable location for the DOCX file.";
  }
  if (error.code === "invalid_envelope") {
    return "This document cannot be exported until its content is valid.";
  }
  return docxFailureMessage(error.cause);
}

function docxFailureMessage(cause: DocxExportErrorCode | undefined) {
  if (cause === "unsupported_citation") {
    return "DOCX export does not currently include citations. Remove citations before exporting; your DRAFT document remains unchanged.";
  }
  if (cause === "unsupported_document_content") {
    return "Some document content is not supported in DOCX export. Remove that content and try again.";
  }
  if (cause === "source_too_large" || cause === "artifact_too_large" || cause === "too_many_nodes" || cause === "nesting_too_deep") {
    return "This document exceeds the DOCX export limits. Reduce its size or nesting and try again.";
  }
  if (cause === "invalid_target") {
    return "Choose a valid DOCX file location and try again.";
  }
  return "DRAFT could not finish the DOCX export. Your DRAFT document was not changed.";
}
