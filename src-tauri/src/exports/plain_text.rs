use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    citations::node::CitationNodeAttributes,
    documents::{
        atomic_write::{AtomicDocumentWriteError, write_document_atomically},
        envelope::DocumentEnvelope,
    },
};

const MAX_TEXT_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_TEXT_NODES: usize = 100_000;
const MAX_TEXT_DEPTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlainTextArtifact {
    bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlainTextWriteStage {
    OpenTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    ReplaceTarget,
    CleanupTemporaryFile,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum PlainTextExportError {
    InvalidTarget,
    InvalidDocumentStructure,
    TooManyNodes,
    NestingTooDeep,
    OutputTooLarge,
    WriteFailed { stage: PlainTextWriteStage },
    DurabilityUncertain,
}

pub(crate) fn compile_plain_text(
    document: &DocumentEnvelope,
) -> Result<PlainTextArtifact, PlainTextExportError> {
    let mut output = TextOutput::default();
    let mut budget = TraversalBudget::default();
    flatten_root(document.document(), &mut output, &mut budget)?;
    output.finish()
}

pub(crate) fn write_plain_text_artifact(
    artifact: &PlainTextArtifact,
    target: &Path,
) -> Result<usize, PlainTextExportError> {
    require_txt_target(target)?;
    write_document_atomically(target, &artifact.bytes).map_err(map_write_error)?;
    Ok(artifact.bytes.len())
}

impl PlainTextArtifact {
    #[cfg(test)]
    fn as_text(&self) -> &str {
        std::str::from_utf8(&self.bytes).expect("plain-text artifact must be UTF-8")
    }
}

#[derive(Default)]
struct TextOutput {
    blocks: Vec<String>,
}

impl TextOutput {
    fn push(&mut self, block: String) {
        self.blocks.push(block);
    }

    fn finish(self) -> Result<PlainTextArtifact, PlainTextExportError> {
        let mut text = self.blocks.join("\n\n");
        if !text.is_empty() {
            text.push('\n');
        }
        if text.len() > MAX_TEXT_OUTPUT_BYTES {
            return Err(PlainTextExportError::OutputTooLarge);
        }
        Ok(PlainTextArtifact {
            bytes: text.into_bytes(),
        })
    }
}

#[derive(Default)]
struct TraversalBudget {
    nodes: usize,
}

impl TraversalBudget {
    fn visit(&mut self, depth: usize) -> Result<(), PlainTextExportError> {
        if depth > MAX_TEXT_DEPTH {
            return Err(PlainTextExportError::NestingTooDeep);
        }
        self.nodes += 1;
        if self.nodes > MAX_TEXT_NODES {
            return Err(PlainTextExportError::TooManyNodes);
        }
        Ok(())
    }
}

fn flatten_root(
    value: &Value,
    output: &mut TextOutput,
    budget: &mut TraversalBudget,
) -> Result<(), PlainTextExportError> {
    let root = node_fields(value)?;
    if node_type(root)? != "doc" {
        return Err(PlainTextExportError::InvalidDocumentStructure);
    }
    for block in child_nodes(root)? {
        flatten_block(block, output, budget, 1)?;
    }
    Ok(())
}

fn flatten_block(
    value: &Value,
    output: &mut TextOutput,
    budget: &mut TraversalBudget,
    depth: usize,
) -> Result<(), PlainTextExportError> {
    budget.visit(depth)?;
    let fields = node_fields(value)?;
    let block = match node_type(fields)? {
        "pageBreak" => "\u{000c}".to_owned(),
        "bulletList" => flatten_list(fields, false, budget, depth + 1)?,
        "orderedList" => flatten_list(fields, true, budget, depth + 1)?,
        "blockquote" => flatten_quote(fields, budget, depth + 1)?,
        "horizontalRule" => "---".to_owned(),
        _ => flatten_visible_text(fields, budget, depth + 1)?,
    };
    output.push(block);
    Ok(())
}

fn flatten_list(
    fields: &Map<String, Value>,
    ordered: bool,
    budget: &mut TraversalBudget,
    depth: usize,
) -> Result<String, PlainTextExportError> {
    child_nodes(fields)?
        .iter()
        .enumerate()
        .map(|(index, item)| flatten_list_item(item, index, ordered, budget, depth))
        .collect::<Result<Vec<_>, _>>()
        .map(|items| items.join("\n"))
}

fn flatten_list_item(
    value: &Value,
    index: usize,
    ordered: bool,
    budget: &mut TraversalBudget,
    depth: usize,
) -> Result<String, PlainTextExportError> {
    budget.visit(depth)?;
    let fields = node_fields(value)?;
    let prefix = if ordered {
        format!("{}. ", index + 1)
    } else {
        "- ".to_owned()
    };
    Ok(format!(
        "{prefix}{}",
        flatten_visible_text(fields, budget, depth + 1)?
    ))
}

fn flatten_quote(
    fields: &Map<String, Value>,
    budget: &mut TraversalBudget,
    depth: usize,
) -> Result<String, PlainTextExportError> {
    let text = flatten_visible_text(fields, budget, depth)?;
    Ok(text
        .lines()
        .map(|line| format!("> {line}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn flatten_visible_text(
    fields: &Map<String, Value>,
    budget: &mut TraversalBudget,
    depth: usize,
) -> Result<String, PlainTextExportError> {
    match node_type(fields)? {
        "text" => return text_value(fields),
        "hardBreak" => return Ok("\n".to_owned()),
        "citation" => return citation_text(fields),
        "pageBreak" => return Ok("\u{000c}".to_owned()),
        _ => {}
    }
    let mut text = String::new();
    for child in optional_child_nodes(fields)? {
        budget.visit(depth)?;
        let child_fields = node_fields(child)?;
        let child_text = flatten_visible_text(child_fields, budget, depth + 1)?;
        append_child_text(&mut text, node_type(child_fields)?, &child_text);
    }
    Ok(text)
}

fn append_child_text(output: &mut String, child_type: &str, child: &str) {
    let separates_blocks = matches!(
        child_type,
        "paragraph" | "heading" | "listItem" | "blockquote" | "bulletList" | "orderedList"
    );
    if separates_blocks && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(child);
}

fn text_value(fields: &Map<String, Value>) -> Result<String, PlainTextExportError> {
    fields
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(PlainTextExportError::InvalidDocumentStructure)
}

fn citation_text(fields: &Map<String, Value>) -> Result<String, PlainTextExportError> {
    let attrs = fields
        .get("attrs")
        .cloned()
        .ok_or(PlainTextExportError::InvalidDocumentStructure)?;
    let citation = CitationNodeAttributes::from_json_value(attrs)
        .map_err(|_| PlainTextExportError::InvalidDocumentStructure)?;
    Ok(format!("[@{}]", citation.citekey()))
}

fn node_fields(value: &Value) -> Result<&Map<String, Value>, PlainTextExportError> {
    value
        .as_object()
        .ok_or(PlainTextExportError::InvalidDocumentStructure)
}

fn node_type(fields: &Map<String, Value>) -> Result<&str, PlainTextExportError> {
    fields
        .get("type")
        .and_then(Value::as_str)
        .ok_or(PlainTextExportError::InvalidDocumentStructure)
}

fn child_nodes(fields: &Map<String, Value>) -> Result<&[Value], PlainTextExportError> {
    fields
        .get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(PlainTextExportError::InvalidDocumentStructure)
}

fn optional_child_nodes(fields: &Map<String, Value>) -> Result<&[Value], PlainTextExportError> {
    match fields.get("content") {
        None => Ok(&[]),
        Some(Value::Array(children)) => Ok(children),
        Some(_) => Err(PlainTextExportError::InvalidDocumentStructure),
    }
}

fn require_txt_target(path: &Path) -> Result<(), PlainTextExportError> {
    let valid = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"));
    valid
        .then_some(())
        .ok_or(PlainTextExportError::InvalidTarget)
}

fn map_write_error(error: AtomicDocumentWriteError) -> PlainTextExportError {
    match error {
        AtomicDocumentWriteError::OpenTemporaryFile => {
            write_failure(PlainTextWriteStage::OpenTemporaryFile)
        }
        AtomicDocumentWriteError::WriteTemporaryFile => {
            write_failure(PlainTextWriteStage::WriteTemporaryFile)
        }
        AtomicDocumentWriteError::SyncTemporaryFile => {
            write_failure(PlainTextWriteStage::SyncTemporaryFile)
        }
        AtomicDocumentWriteError::ReplaceTarget => {
            write_failure(PlainTextWriteStage::ReplaceTarget)
        }
        AtomicDocumentWriteError::CleanupTemporaryFile => {
            write_failure(PlainTextWriteStage::CleanupTemporaryFile)
        }
        AtomicDocumentWriteError::SyncParentDirectory => PlainTextExportError::DurabilityUncertain,
    }
}

fn write_failure(stage: PlainTextWriteStage) -> PlainTextExportError {
    PlainTextExportError::WriteFailed { stage }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn plain_text_flattening_is_deterministic_and_visible_only() {
        let document = envelope(json!({
            "type": "doc",
            "content": [
                { "type": "heading", "attrs": { "level": 1 }, "content": [{ "type": "text", "text": "Title" }] },
                { "type": "paragraph", "content": [{ "type": "text", "text": "First" }, { "type": "hardBreak" }, { "type": "text", "text": "Second" }] },
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "One" }] }] },
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Two" }] }] }
                ] },
                { "type": "blockquote", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Quoted" }] }] },
                { "type": "pageBreak" },
                { "type": "paragraph", "content": [{ "type": "citation", "attrs": { "schema_version": 1, "citekey": "smith2026", "render_style": "apa7" } }] }
            ]
        }));

        assert_eq!(
            compile_plain_text(&document).unwrap().as_text(),
            "Title\n\nFirst\nSecond\n\n- One\n- Two\n\n> Quoted\n\n\u{000c}\n\n[@smith2026]\n"
        );
    }

    #[test]
    fn invalid_structure_fails_before_output() {
        let document = envelope(json!({
            "type": "doc",
            "content": [{ "type": "paragraph", "content": "invalid" }]
        }));
        assert_eq!(
            compile_plain_text(&document),
            Err(PlainTextExportError::InvalidDocumentStructure)
        );
    }

    fn envelope(document: Value) -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(json!({
            "schema_version": 2,
            "document_id": "00000000-0000-4000-8000-000000000001",
            "title": "Text output",
            "document": document
        }))
        .unwrap()
    }
}
