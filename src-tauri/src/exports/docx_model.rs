use std::io::{self, Write};

use serde_json::{Map, Value};

use crate::documents::paragraph_format::{ParagraphStyle, parse_paragraph_style_attrs};
use crate::documents::text_format::{
    FontFamily, FontSizePoints, parse_font_family_attrs, parse_font_size_attrs,
};

use super::docx::{
    DocxContentPath, DocxExportError, MAX_DOCX_NESTING_DEPTH, MAX_DOCX_NODES, MAX_DOCX_SOURCE_BYTES,
};

const ROOT_FIELDS: [&str; 2] = ["type", "content"];
const PARAGRAPH_FIELDS: [&str; 3] = ["type", "attrs", "content"];
const HEADING_FIELDS: [&str; 3] = ["type", "attrs", "content"];
const PARAGRAPH_ATTR_FIELDS: [&str; 1] = ["paragraphStyle"];
const HEADING_ATTR_FIELDS: [&str; 2] = ["level", "paragraphStyle"];
const TEXT_FIELDS: [&str; 3] = ["type", "text", "marks"];
const HARD_BREAK_FIELDS: [&str; 1] = ["type"];
const PAGE_BREAK_FIELDS: [&str; 1] = ["type"];
const MARK_FIELDS: [&str; 1] = ["type"];
const FONT_MARK_FIELDS: [&str; 2] = ["type", "attrs"];

pub(super) struct DocxDocument {
    pub(super) blocks: Vec<DocxBlock>,
}

pub(super) enum DocxBlock {
    Paragraph {
        style: Option<ParagraphStyle>,
        content: Vec<DocxInline>,
    },
    Heading {
        level: u8,
        style: Option<ParagraphStyle>,
        content: Vec<DocxInline>,
    },
    PageBreak,
}

pub(super) enum DocxInline {
    Text { value: String, marks: TextMarks },
    HardBreak,
}

#[derive(Clone, Copy, Default)]
pub(super) struct TextMarks {
    pub(super) bold: bool,
    pub(super) font_family: Option<FontFamily>,
    pub(super) font_size: Option<FontSizePoints>,
    pub(super) italic: bool,
    pub(super) underline: bool,
}

#[derive(Clone, Copy)]
struct ResourceLimits {
    source_bytes: usize,
    nodes: usize,
    depth: usize,
}

pub(super) fn parse_docx_document(value: &Value) -> Result<DocxDocument, DocxExportError> {
    require_resource_limits(value, ResourceLimits::production())?;
    parse_root(value)
}

impl ResourceLimits {
    fn production() -> Self {
        Self {
            source_bytes: MAX_DOCX_SOURCE_BYTES,
            nodes: MAX_DOCX_NODES,
            depth: MAX_DOCX_NESTING_DEPTH,
        }
    }
}

fn parse_root(value: &Value) -> Result<DocxDocument, DocxExportError> {
    let path = DocxContentPath::root();
    let fields = node_fields(value, &path)?;
    require_exact_fields(fields, &ROOT_FIELDS, &path)?;
    require_node_type(fields, "doc", &path)?;
    let content = required_array(fields, "content", &path)?;
    Ok(DocxDocument {
        blocks: parse_blocks(content, &path)?,
    })
}

fn parse_blocks(
    values: &[Value],
    parent_path: &DocxContentPath,
) -> Result<Vec<DocxBlock>, DocxExportError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_block(value, &parent_path.child(index)))
        .collect()
}

fn parse_block(value: &Value, path: &DocxContentPath) -> Result<DocxBlock, DocxExportError> {
    let fields = node_fields(value, path)?;
    match node_type(fields, path)? {
        "paragraph" => parse_paragraph(fields, path),
        "heading" => parse_heading(fields, path),
        "pageBreak" => parse_page_break(fields, path),
        "citation" => Err(unsupported_citation(path)),
        _ => Err(unsupported_content(path)),
    }
}

fn parse_page_break(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<DocxBlock, DocxExportError> {
    require_exact_fields(fields, &PAGE_BREAK_FIELDS, path)?;
    Ok(DocxBlock::PageBreak)
}

fn parse_paragraph(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<DocxBlock, DocxExportError> {
    require_exact_fields(fields, &PARAGRAPH_FIELDS, path)?;
    Ok(DocxBlock::Paragraph {
        style: parse_block_style(fields, &PARAGRAPH_ATTR_FIELDS, path)?,
        content: parse_optional_inline_content(fields, path)?,
    })
}

fn parse_heading(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<DocxBlock, DocxExportError> {
    require_exact_fields(fields, &HEADING_FIELDS, path)?;
    let level = parse_heading_level(fields, path)?;
    let style = parse_block_style(fields, &HEADING_ATTR_FIELDS, path)?;
    let content = parse_optional_inline_content(fields, path)?;
    Ok(DocxBlock::Heading {
        level,
        style,
        content,
    })
}

fn parse_block_style(
    fields: &Map<String, Value>,
    allowed_attrs: &[&str],
    path: &DocxContentPath,
) -> Result<Option<ParagraphStyle>, DocxExportError> {
    let Some(attrs) = fields.get("attrs") else {
        return Ok(None);
    };
    let attr_fields = attrs.as_object().ok_or_else(|| invalid_structure(path))?;
    require_exact_fields(attr_fields, allowed_attrs, path)?;
    parse_paragraph_style_attrs(Some(attrs)).map_err(|_| invalid_structure(path))
}

fn parse_heading_level(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<u8, DocxExportError> {
    let attrs = required_object(fields, "attrs", path)?;
    require_exact_fields(attrs, &HEADING_ATTR_FIELDS, path)?;
    match attrs.get("level").and_then(Value::as_u64) {
        Some(level @ 1..=6) => Ok(level as u8),
        _ => Err(invalid_structure(path)),
    }
}

fn parse_optional_inline_content(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<Vec<DocxInline>, DocxExportError> {
    match fields.get("content") {
        None => Ok(vec![]),
        Some(Value::Array(content)) => parse_inlines(content, path),
        Some(_) => Err(invalid_structure(path)),
    }
}

fn parse_inlines(
    values: &[Value],
    parent_path: &DocxContentPath,
) -> Result<Vec<DocxInline>, DocxExportError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_inline(value, &parent_path.child(index)))
        .collect()
}

fn parse_inline(value: &Value, path: &DocxContentPath) -> Result<DocxInline, DocxExportError> {
    let fields = node_fields(value, path)?;
    match node_type(fields, path)? {
        "text" => parse_text(fields, path),
        "hardBreak" => parse_hard_break(fields, path),
        "citation" => Err(unsupported_citation(path)),
        _ => Err(unsupported_content(path)),
    }
}

fn parse_text(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<DocxInline, DocxExportError> {
    require_exact_fields(fields, &TEXT_FIELDS, path)?;
    let value = fields.get("text").and_then(Value::as_str);
    let Some(value) = value.filter(|value| is_valid_xml_text(value)) else {
        return Err(invalid_structure(path));
    };
    Ok(DocxInline::Text {
        value: value.to_owned(),
        marks: parse_marks(fields.get("marks"), path)?,
    })
}

fn parse_hard_break(
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<DocxInline, DocxExportError> {
    require_exact_fields(fields, &HARD_BREAK_FIELDS, path)?;
    Ok(DocxInline::HardBreak)
}

fn parse_marks(
    value: Option<&Value>,
    path: &DocxContentPath,
) -> Result<TextMarks, DocxExportError> {
    match value {
        None => Ok(TextMarks::default()),
        Some(Value::Array(marks)) => collect_marks(marks, path),
        Some(_) => Err(invalid_structure(path)),
    }
}

fn collect_marks(values: &[Value], path: &DocxContentPath) -> Result<TextMarks, DocxExportError> {
    let mut marks = TextMarks::default();
    for (index, value) in values.iter().enumerate() {
        add_mark(&mut marks, value, &path.child(index))?;
    }
    Ok(marks)
}

fn add_mark(
    marks: &mut TextMarks,
    value: &Value,
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    let fields = node_fields(value, path)?;
    match node_type(fields, path)? {
        "fontFamily" => add_font_family(marks, fields, path),
        "fontSize" => add_font_size(marks, fields, path),
        mark_type => add_boolean_mark(marks, fields, mark_type, path),
    }
}

fn add_boolean_mark(
    marks: &mut TextMarks,
    fields: &Map<String, Value>,
    mark_type: &str,
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    require_exact_fields(fields, &MARK_FIELDS, path)?;
    let slot = mark_slot(marks, mark_type, path)?;
    if *slot {
        return Err(invalid_structure(path));
    }
    *slot = true;
    Ok(())
}

fn add_font_family(
    marks: &mut TextMarks,
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    require_exact_fields(fields, &FONT_MARK_FIELDS, path)?;
    let attrs = fields.get("attrs").ok_or_else(|| invalid_structure(path))?;
    let family = parse_font_family_attrs(attrs).map_err(|_| unsupported_content(path))?;
    if marks.font_family.replace(family).is_some() {
        return Err(invalid_structure(path));
    }
    Ok(())
}

fn add_font_size(
    marks: &mut TextMarks,
    fields: &Map<String, Value>,
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    require_exact_fields(fields, &FONT_MARK_FIELDS, path)?;
    let attrs = fields.get("attrs").ok_or_else(|| invalid_structure(path))?;
    let size = parse_font_size_attrs(attrs).map_err(|_| unsupported_content(path))?;
    if marks.font_size.replace(size).is_some() {
        return Err(invalid_structure(path));
    }
    Ok(())
}

fn mark_slot<'a>(
    marks: &'a mut TextMarks,
    mark_type: &str,
    path: &DocxContentPath,
) -> Result<&'a mut bool, DocxExportError> {
    match mark_type {
        "bold" => Ok(&mut marks.bold),
        "italic" => Ok(&mut marks.italic),
        "underline" => Ok(&mut marks.underline),
        _ => Err(unsupported_content(path)),
    }
}

fn require_resource_limits(value: &Value, limits: ResourceLimits) -> Result<(), DocxExportError> {
    require_serialized_size(value, limits.source_bytes)?;
    let mut nodes = 0;
    count_nodes(value, 0, limits, &mut nodes)
}

fn require_serialized_size(value: &Value, limit: usize) -> Result<(), DocxExportError> {
    let mut counter = ByteCounter::new(limit);
    serde_json::to_writer(&mut counter, value).map_err(|_| DocxExportError::SourceTooLarge)
}

fn count_nodes(
    value: &Value,
    depth: usize,
    limits: ResourceLimits,
    nodes: &mut usize,
) -> Result<(), DocxExportError> {
    if depth > limits.depth {
        return Err(DocxExportError::NestingTooDeep);
    }
    match value {
        Value::Object(fields) => count_object_nodes(fields, depth, limits, nodes),
        Value::Array(values) => count_array_nodes(values, depth, limits, nodes),
        _ => Ok(()),
    }
}

fn count_object_nodes(
    fields: &Map<String, Value>,
    depth: usize,
    limits: ResourceLimits,
    nodes: &mut usize,
) -> Result<(), DocxExportError> {
    if fields.contains_key("type") {
        *nodes += 1;
        if *nodes > limits.nodes {
            return Err(DocxExportError::TooManyNodes);
        }
    }
    for value in fields.values() {
        count_nodes(value, depth + 1, limits, nodes)?;
    }
    Ok(())
}

fn count_array_nodes(
    values: &[Value],
    depth: usize,
    limits: ResourceLimits,
    nodes: &mut usize,
) -> Result<(), DocxExportError> {
    for value in values {
        count_nodes(value, depth + 1, limits, nodes)?;
    }
    Ok(())
}

fn node_fields<'a>(
    value: &'a Value,
    path: &DocxContentPath,
) -> Result<&'a Map<String, Value>, DocxExportError> {
    value.as_object().ok_or_else(|| invalid_structure(path))
}

fn node_type<'a>(
    fields: &'a Map<String, Value>,
    path: &DocxContentPath,
) -> Result<&'a str, DocxExportError> {
    fields
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_structure(path))
}

fn require_node_type(
    fields: &Map<String, Value>,
    expected: &str,
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    if node_type(fields, path)? == expected {
        Ok(())
    } else {
        Err(invalid_structure(path))
    }
}

fn require_exact_fields(
    fields: &Map<String, Value>,
    allowed: &[&str],
    path: &DocxContentPath,
) -> Result<(), DocxExportError> {
    if fields.keys().all(|field| allowed.contains(&field.as_str())) {
        Ok(())
    } else {
        Err(unsupported_content(path))
    }
}

fn required_array<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
    path: &DocxContentPath,
) -> Result<&'a [Value], DocxExportError> {
    fields
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid_structure(path))
}

fn required_object<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
    path: &DocxContentPath,
) -> Result<&'a Map<String, Value>, DocxExportError> {
    fields
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_structure(path))
}

fn is_valid_xml_text(value: &str) -> bool {
    value.chars().all(|character| {
        matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
            || matches!(character as u32, 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
    })
}

fn invalid_structure(path: &DocxContentPath) -> DocxExportError {
    DocxExportError::InvalidDocumentStructure { path: path.clone() }
}

fn unsupported_content(path: &DocxContentPath) -> DocxExportError {
    DocxExportError::UnsupportedDocumentContent { path: path.clone() }
}

fn unsupported_citation(path: &DocxContentPath) -> DocxExportError {
    DocxExportError::UnsupportedCitation { path: path.clone() }
}

struct ByteCounter {
    bytes: usize,
    limit: usize,
}

impl ByteCounter {
    fn new(limit: usize) -> Self {
        Self { bytes: 0, limit }
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let next = self.bytes.saturating_add(buffer.len());
        if next > self.limit {
            return Err(io::Error::other("byte limit exceeded"));
        }
        self.bytes = next;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
pub(super) fn require_test_resource_limits(
    value: &Value,
    source_bytes: usize,
    nodes: usize,
    depth: usize,
) -> Result<(), DocxExportError> {
    require_resource_limits(
        value,
        ResourceLimits {
            source_bytes,
            nodes,
            depth,
        },
    )
}
