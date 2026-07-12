use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use serde_json::{Value, json};
use uuid::Uuid;

use super::envelope::{DOCUMENT_ENVELOPE_SCHEMA_VERSION, DocumentEnvelope, DocumentEnvelopeError};

pub(crate) const MAX_TEXT_IMPORT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TextImportError {
    FileNotFound,
    ReadFailed,
    TooLarge,
    InvalidUtf8,
    InvalidEnvelope(DocumentEnvelopeError),
}

pub(crate) fn import_text_document(path: &Path) -> Result<DocumentEnvelope, TextImportError> {
    let bytes = read_bounded_text(path)?;
    let text = String::from_utf8(bytes).map_err(|_| TextImportError::InvalidUtf8)?;
    create_imported_envelope(import_title(path), &text)
}

fn read_bounded_text(path: &Path) -> Result<Vec<u8>, TextImportError> {
    let file = File::open(path).map_err(map_read_error)?;
    let mut bytes = Vec::new();
    file.take((MAX_TEXT_IMPORT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| TextImportError::ReadFailed)?;
    if bytes.len() > MAX_TEXT_IMPORT_BYTES {
        return Err(TextImportError::TooLarge);
    }
    Ok(bytes)
}

fn map_read_error(error: io::Error) -> TextImportError {
    match error.kind() {
        io::ErrorKind::NotFound => TextImportError::FileNotFound,
        _ => TextImportError::ReadFailed,
    }
}

fn import_title(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Imported document")
        .to_owned()
}

fn create_imported_envelope(
    title: String,
    text: &str,
) -> Result<DocumentEnvelope, TextImportError> {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": Uuid::new_v4().to_string(),
        "title": title,
        "document": text_document(text),
    }))
    .map_err(TextImportError::InvalidEnvelope)
}

fn text_document(text: &str) -> Value {
    let content = normalized_lines(text)
        .map(paragraph_for_line)
        .collect::<Vec<_>>();
    json!({ "type": "doc", "content": content })
}

fn normalized_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
}

fn paragraph_for_line(line: &str) -> Value {
    if line.is_empty() {
        json!({ "type": "paragraph" })
    } else {
        json!({
            "type": "paragraph",
            "content": [{ "type": "text", "text": line }]
        })
    }
}
