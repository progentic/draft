use std::collections::{BTreeSet, HashMap};

use quick_xml::{
    Reader, XmlVersion,
    events::{BytesRef, BytesStart, BytesText, Event},
};
use serde_json::{Value, json};

use crate::documents::paragraph_format::{
    LINE_SPACING_INCREMENT, MAX_LINE_SPACING_HUNDREDTHS, MAX_PARAGRAPH_SPACING_TWIPS,
    MAX_SPECIAL_INDENT_TWIPS, MIN_LINE_SPACING_HUNDREDTHS, PARAGRAPH_STYLE_SCHEMA_VERSION,
};
use crate::documents::text_format::{FontFamily, FontSizePoints};

use super::{
    DocxImportError, ExternalFeature, ExternalSafetyReason, FidelityAccumulator,
    footnotes::FootnoteCatalog, table::TableBuilder,
};

const MAX_IMPORTED_NODES: usize = 100_000;
const SUPPORTED_ALIGNMENT: [&str; 4] = ["left", "center", "right", "both"];
const VALID_UNSUPPORTED_ALIGNMENT: [&str; 5] =
    ["distribute", "end", "highKashida", "lowKashida", "start"];

pub(super) fn parse_document(
    xml: &[u8],
    footnotes: &FootnoteCatalog,
    fidelity: &mut FidelityAccumulator,
) -> Result<Value, DocxImportError> {
    DocumentParser::new(footnotes, fidelity).parse(xml)
}

struct DocumentParser<'a> {
    fidelity: &'a mut FidelityAccumulator,
    footnote_ids: Vec<String>,
    footnotes: &'a FootnoteCatalog,
    blocks: Vec<Value>,
    paragraph: Option<ParagraphBuilder>,
    run: Option<RunBuilder>,
    stack: Vec<String>,
    table: Option<TableBuilder>,
    ignored_depth: usize,
    nodes: usize,
}

impl<'a> DocumentParser<'a> {
    fn new(footnotes: &'a FootnoteCatalog, fidelity: &'a mut FidelityAccumulator) -> Self {
        Self {
            fidelity,
            footnote_ids: Vec::new(),
            footnotes,
            blocks: Vec::new(),
            paragraph: None,
            run: None,
            stack: Vec::new(),
            table: None,
            ignored_depth: 0,
            nodes: 0,
        }
    }

    fn parse(mut self, xml: &[u8]) -> Result<Value, DocxImportError> {
        let mut reader = Reader::from_reader(xml);
        loop {
            match reader
                .read_event()
                .map_err(|_| DocxImportError::malformed())?
            {
                Event::Start(element) => self.start(&element)?,
                Event::Empty(element) => self.empty(&element)?,
                Event::End(element) => self.end(local_name(element.name().as_ref()))?,
                Event::Text(text) => self.text(&text)?,
                Event::GeneralRef(reference) => self.reference(&reference)?,
                Event::DocType(_) => return Err(unsafe_xml_entity()),
                Event::CData(_) => return Err(DocxImportError::malformed()),
                Event::Eof => return self.finish(),
                _ => {}
            }
        }
    }

    fn start(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let name = local_name(element.name().as_ref()).to_vec();
        if self.ignored_depth > 0 {
            self.ignored_depth += 1;
        } else {
            self.handle_element(&name, element)?;
        }
        self.stack.push(name_string(&name)?);
        Ok(())
    }

    fn empty(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let name = local_name(element.name().as_ref()).to_vec();
        if self.ignored_depth > 0 {
            return Ok(());
        }
        self.handle_element(&name, element)?;
        self.handle_empty_content(&name)?;
        self.ignored_depth = 0;
        Ok(())
    }

    fn end(&mut self, name: &[u8]) -> Result<(), DocxImportError> {
        let expected = self.stack.pop().ok_or_else(DocxImportError::malformed)?;
        if expected.as_bytes() != name {
            return Err(DocxImportError::malformed());
        }
        if self.ignored_depth > 0 {
            self.ignored_depth -= 1;
            return Ok(());
        }
        match name {
            b"t" => self.finish_text(),
            b"r" => self.finish_run(),
            b"p" => self.finish_paragraph(),
            b"tc" => self.finish_table_cell(),
            b"tr" => self.finish_table_row(),
            b"tbl" => self.finish_table(),
            _ => Ok(()),
        }
    }

    fn text(&mut self, text: &BytesText<'_>) -> Result<(), DocxImportError> {
        if self.ignored_depth > 0 {
            return Ok(());
        }
        if self.stack.last().is_some_and(|name| name == "t") {
            let decoded = text.decode().map_err(|_| DocxImportError::malformed())?;
            self.current_run()?.text.push_str(&decoded);
        }
        Ok(())
    }

    fn reference(&mut self, reference: &BytesRef<'_>) -> Result<(), DocxImportError> {
        if self.ignored_depth > 0 {
            return Ok(());
        }
        if self.stack.last().is_none_or(|name| name != "t") {
            return Err(unsafe_xml_entity());
        }
        let character = resolve_reference(reference)?;
        self.current_run()?.text.push(character);
        Ok(())
    }

    fn finish(mut self) -> Result<Value, DocxImportError> {
        if !self.stack.is_empty()
            || self.ignored_depth != 0
            || self.paragraph.is_some()
            || self.run.is_some()
            || self.table.is_some()
        {
            return Err(DocxImportError::malformed());
        }
        self.append_footnotes()?;
        require_canonical_node_count(&self.blocks)?;
        Ok(json!({ "type": "doc", "content": self.blocks }))
    }

    fn handle_element(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        match name {
            b"tbl" => self.start_table(),
            b"tr" if self.table.is_some() => self.start_table_row(),
            b"tc" if self.table.is_some() => self.start_table_cell(),
            b"tblPr" | b"tblGrid" | b"tblPrEx" | b"trPr" | b"tcPr"
                if self.table.is_some() && self.paragraph.is_none() =>
            {
                self.ignore_lossy(ExternalFeature::TableStructure)
            }
            b"rPr" if self.run.is_none() && self.in_paragraph_properties() => {
                self.ignore_unsupported(ExternalFeature::RunFormatting)
            }
            b"document" | b"body" | b"pPr" | b"rPr" | b"t" => Ok(()),
            b"p" => self.start_paragraph(),
            b"r" => self.start_run(),
            b"br" => self.push_break(element),
            b"footnoteReference" if self.run.is_some() => self.push_footnote_reference(element),
            b"tab" if self.run.is_some() => self.push_inline_tab(),
            b"lastRenderedPageBreak" if self.run.is_some() => {
                self.record_unsupported(ExternalFeature::PaginationControl)
            }
            b"hyperlink" if self.paragraph.is_some() && self.run.is_none() => {
                self.record_unsupported(ExternalFeature::UnsupportedDocumentStructure)
            }
            b"proofErr" if self.paragraph.is_some() && self.run.is_none() => {
                self.record_unsupported(ExternalFeature::RunFormatting)
            }
            _ if self.in_paragraph_properties() => self.paragraph_property(name, element),
            _ if self.in_run_properties() => self.run_property(name, element),
            b"sectPr" => self.ignore_unsupported(ExternalFeature::UnsupportedDocumentStructure),
            _ => self.reject_unknown_structure(name),
        }
    }

    fn handle_empty_content(&mut self, name: &[u8]) -> Result<(), DocxImportError> {
        match name {
            b"p" => self.finish_paragraph(),
            b"r" => self.finish_run(),
            b"tc" => self.finish_table_cell(),
            b"tr" => self.finish_table_row(),
            b"tbl" => self.finish_table(),
            _ => Ok(()),
        }
    }

    fn start_paragraph(&mut self) -> Result<(), DocxImportError> {
        if self.paragraph.is_some() || self.run.is_some() {
            return Err(DocxImportError::malformed());
        }
        self.increment_nodes()?;
        self.paragraph = Some(ParagraphBuilder::default());
        Ok(())
    }

    fn start_table(&mut self) -> Result<(), DocxImportError> {
        if self.table.is_some() || self.paragraph.is_some() || self.run.is_some() {
            return Err(unsupported(ExternalFeature::TableStructure));
        }
        self.fidelity.record_lossy(ExternalFeature::TableStructure);
        self.table = Some(TableBuilder::default());
        Ok(())
    }

    fn start_table_row(&mut self) -> Result<(), DocxImportError> {
        self.table
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .start_row()
    }

    fn start_table_cell(&mut self) -> Result<(), DocxImportError> {
        self.table
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .start_cell()
    }

    fn start_run(&mut self) -> Result<(), DocxImportError> {
        if self.paragraph.is_none() || self.run.is_some() {
            return Err(DocxImportError::malformed());
        }
        self.run = Some(RunBuilder::default());
        Ok(())
    }

    fn push_break(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.flush_text_node()?;
        match break_kind(element)? {
            BreakKind::Line => self.push_run_node(json!({ "type": "hardBreak" })),
            BreakKind::Page => self.push_run_node(json!({ "type": "pageBreak" })),
            BreakKind::Column => {
                self.record_unsupported(ExternalFeature::UnsupportedDocumentStructure)
            }
        }
    }

    fn push_inline_tab(&mut self) -> Result<(), DocxImportError> {
        self.record_unsupported(ExternalFeature::ParagraphTab)?;
        self.current_run()?.text.push(' ');
        Ok(())
    }

    fn push_footnote_reference(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let values = attributes(element)?;
        let id = values
            .get("id")
            .filter(|_| values.len() == 1)
            .cloned()
            .ok_or_else(DocxImportError::malformed)?;
        if self.footnotes.text(&id).is_none() {
            return Err(DocxImportError::malformed());
        }
        self.fidelity.record_lossy(ExternalFeature::Footnote);
        self.current_run()?.text.push_str(&format!("[{id}]"));
        if !self.footnote_ids.contains(&id) {
            self.footnote_ids.push(id);
        }
        Ok(())
    }

    fn finish_text(&mut self) -> Result<(), DocxImportError> {
        self.flush_text_node()
    }

    fn finish_run(&mut self) -> Result<(), DocxImportError> {
        self.flush_text_node()?;
        let run = self.run.take().ok_or_else(DocxImportError::malformed)?;
        self.current_paragraph()?.content.extend(run.content);
        Ok(())
    }

    fn finish_paragraph(&mut self) -> Result<(), DocxImportError> {
        if self.run.is_some() {
            return Err(DocxImportError::malformed());
        }
        let paragraph = self
            .paragraph
            .take()
            .ok_or_else(DocxImportError::malformed)?;
        let blocks = paragraph.into_blocks();
        match self.table.as_mut() {
            Some(table) => table.push_blocks(blocks),
            None => {
                self.blocks.extend(blocks);
                Ok(())
            }
        }
    }

    fn finish_table_cell(&mut self) -> Result<(), DocxImportError> {
        self.table
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .finish_cell()
    }

    fn finish_table_row(&mut self) -> Result<(), DocxImportError> {
        self.table
            .as_mut()
            .ok_or_else(DocxImportError::malformed)?
            .finish_row()
    }

    fn finish_table(&mut self) -> Result<(), DocxImportError> {
        let table = self.table.take().ok_or_else(DocxImportError::malformed)?;
        self.blocks.extend(table.finish()?);
        Ok(())
    }

    fn append_footnotes(&mut self) -> Result<(), DocxImportError> {
        for id in std::mem::take(&mut self.footnote_ids) {
            let text = self
                .footnotes
                .text(&id)
                .ok_or_else(DocxImportError::malformed)?;
            self.blocks.push(footnote_block(&id, text));
        }
        Ok(())
    }

    fn paragraph_property(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        match name {
            b"pStyle" => self.apply_paragraph_style(element),
            b"jc" => self.apply_alignment(element),
            b"spacing" => self.apply_spacing(element),
            b"ind" => self.apply_indentation(element),
            b"pBdr" => self.ignore_unsupported(ExternalFeature::ParagraphBorder),
            b"shd" => self.record_unsupported(ExternalFeature::ParagraphShading),
            b"tabs" => self.ignore_unsupported(ExternalFeature::ParagraphTab),
            b"contextualSpacing" => self.record_unsupported(ExternalFeature::ContextualSpacing),
            b"pageBreakBefore" => self.apply_page_break_before(element),
            b"keepNext" | b"keepLines" | b"widowControl" => {
                self.record_unsupported(ExternalFeature::PaginationControl)
            }
            b"numPr" => self.ignore_lossy(ExternalFeature::ListIndentation),
            _ => self.ignore_unsupported(ExternalFeature::UnsupportedDocumentStructure),
        }
    }

    fn run_property(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        match name {
            b"rFonts" => self.apply_font_family(element),
            b"sz" | b"szCs" => self.apply_font_size(name, element),
            b"b" => self.apply_boolean_mark("b", element, BooleanMark::Bold),
            b"i" => self.apply_boolean_mark("i", element, BooleanMark::Italic),
            b"u" => self.apply_underline(element),
            _ => self.ignore_unsupported(ExternalFeature::RunFormatting),
        }
    }

    fn reject_unknown_structure(&self, name: &[u8]) -> Result<(), DocxImportError> {
        if self.paragraph.is_some() || matches!(name, b"hyperlink" | b"sdt" | b"altChunk") {
            Err(unsupported(ExternalFeature::UnsupportedDocumentStructure))
        } else {
            Err(DocxImportError::malformed())
        }
    }

    fn apply_paragraph_style(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let value = required_value(element)?;
        let paragraph = self.current_paragraph()?;
        paragraph.mark_once("pStyle")?;
        if value == "Normal" {
            return Ok(());
        }
        if let Some(level) = heading_level(&value) {
            paragraph.heading_level = Some(level);
            return Ok(());
        }
        if let Some(level) = heading_level_with_space(&value) {
            paragraph.heading_level = Some(level);
            self.fidelity
                .record_normalization(ExternalFeature::AlternateHeadingStyleName);
            return Ok(());
        }
        self.record_unsupported(ExternalFeature::UnsupportedStyleInheritance)
    }

    fn apply_alignment(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let value = required_value(element)?;
        self.current_paragraph()?.mark_once("jc")?;
        if SUPPORTED_ALIGNMENT.contains(&value.as_str()) {
            self.current_paragraph()?.style.alignment = match value.as_str() {
                "left" => "left",
                "center" => "center",
                "right" => "right",
                "both" => "justify",
                _ => unreachable!("supported alignment checked above"),
            };
            self.current_paragraph()?.style.touched = true;
            return Ok(());
        }
        if VALID_UNSUPPORTED_ALIGNMENT.contains(&value.as_str()) {
            return Err(unsupported(ExternalFeature::UnsupportedDocumentStructure));
        }
        Err(DocxImportError::malformed())
    }

    fn apply_spacing(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.current_paragraph()?.mark_once("spacing")?;
        let attributes = attributes(element)?;
        reject_spacing_extensions(&attributes)?;
        let paragraph = self.current_paragraph()?;
        paragraph.style.apply_spacing(&attributes)?;
        Ok(())
    }

    fn apply_indentation(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.current_paragraph()?.mark_once("ind")?;
        let attributes = attributes(element)?;
        reject_indent_extensions(&attributes)?;
        let paragraph = self.current_paragraph()?;
        paragraph.style.apply_indentation(&attributes)
    }

    fn apply_page_break_before(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.current_paragraph()?.mark_once("pageBreakBefore")?;
        if on_off_value(element)? {
            self.current_paragraph()?.page_break_before = true;
            self.fidelity
                .record_normalization(ExternalFeature::PaginationControl);
        }
        Ok(())
    }

    fn apply_font_family(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.current_run()?.mark_once("rFonts")?;
        let values = attributes(element)?;
        let family = explicit_font_family(&values);
        if has_unsupported_font_attributes(&values) || family.is_none() {
            self.fidelity
                .record_unsupported(ExternalFeature::RunFormatting);
        }
        if let Some(family) = family {
            self.current_run()?.formatting.font_family = Some(family);
        }
        Ok(())
    }

    fn apply_font_size(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        let property = if name == b"sz" { "sz" } else { "szCs" };
        self.current_run()?.mark_once(property)?;
        let half_points = unsigned_integer(&required_value(element)?)?;
        let size = (half_points % 2 == 0)
            .then(|| FontSizePoints::from_u64(half_points / 2))
            .flatten();
        let Some(size) = size else {
            self.fidelity
                .record_unsupported(ExternalFeature::RunFormatting);
            return Ok(());
        };
        if self
            .current_run()?
            .formatting
            .font_size
            .is_some_and(|current| current != size)
        {
            self.fidelity
                .record_unsupported(ExternalFeature::RunFormatting);
            return Ok(());
        }
        self.current_run()?.formatting.font_size = Some(size);
        Ok(())
    }

    fn apply_boolean_mark(
        &mut self,
        property: &'static str,
        element: &BytesStart<'_>,
        mark: BooleanMark,
    ) -> Result<(), DocxImportError> {
        self.current_run()?.mark_once(property)?;
        let enabled = on_off_value(element)?;
        let formatting = &mut self.current_run()?.formatting;
        match mark {
            BooleanMark::Bold => formatting.bold = enabled,
            BooleanMark::Italic => formatting.italic = enabled,
        }
        Ok(())
    }

    fn apply_underline(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        self.current_run()?.mark_once("u")?;
        let values = attributes(element)?;
        let value = values.get("val").map(String::as_str).unwrap_or("single");
        if values.len() > usize::from(values.contains_key("val")) {
            return Err(DocxImportError::malformed());
        }
        match value {
            "single" | "1" | "true" | "on" | "yes" => {
                self.current_run()?.formatting.underline = true;
            }
            "none" | "0" | "false" | "off" | "no" => {}
            _ => self
                .fidelity
                .record_unsupported(ExternalFeature::RunFormatting),
        }
        Ok(())
    }

    fn record_unsupported(&mut self, feature: ExternalFeature) -> Result<(), DocxImportError> {
        self.fidelity.record_unsupported(feature);
        Ok(())
    }

    fn in_paragraph_properties(&self) -> bool {
        self.stack.last().is_some_and(|name| name == "pPr")
    }

    fn in_run_properties(&self) -> bool {
        self.stack.last().is_some_and(|name| name == "rPr")
    }

    fn current_paragraph(&mut self) -> Result<&mut ParagraphBuilder, DocxImportError> {
        self.paragraph
            .as_mut()
            .ok_or_else(DocxImportError::malformed)
    }

    fn current_run(&mut self) -> Result<&mut RunBuilder, DocxImportError> {
        self.run.as_mut().ok_or_else(DocxImportError::malformed)
    }

    fn flush_text_node(&mut self) -> Result<(), DocxImportError> {
        if self.current_run()?.has_text() {
            self.increment_nodes()?;
            self.current_run()?.flush_text();
        }
        Ok(())
    }

    fn push_run_node(&mut self, node: Value) -> Result<(), DocxImportError> {
        self.increment_nodes()?;
        self.current_run()?.content.push(node);
        Ok(())
    }

    fn ignore_unsupported(&mut self, feature: ExternalFeature) -> Result<(), DocxImportError> {
        self.record_unsupported(feature)?;
        self.ignored_depth = 1;
        Ok(())
    }

    fn ignore_lossy(&mut self, feature: ExternalFeature) -> Result<(), DocxImportError> {
        self.fidelity.record_lossy(feature);
        self.ignored_depth = 1;
        Ok(())
    }

    fn increment_nodes(&mut self) -> Result<(), DocxImportError> {
        self.nodes += 1;
        if self.nodes > MAX_IMPORTED_NODES {
            Err(DocxImportError::unsafe_input(
                ExternalSafetyReason::XmlNodeCount,
            ))
        } else {
            Ok(())
        }
    }
}

fn footnote_block(id: &str, text: &str) -> Value {
    let mut content = vec![json!({ "type": "text", "text": format!("[{id}]") })];
    for (index, line) in text.lines().enumerate() {
        if index > 0 {
            content.push(json!({ "type": "hardBreak" }));
        } else if !line.is_empty() {
            content.push(json!({ "type": "text", "text": " " }));
        }
        if !line.is_empty() {
            content.push(json!({ "type": "text", "text": line }));
        }
    }
    json!({ "type": "paragraph", "content": content })
}

fn require_canonical_node_count(blocks: &[Value]) -> Result<(), DocxImportError> {
    let nodes = blocks.iter().try_fold(0_usize, |count, block| {
        count.checked_add(canonical_node_count(block))
    });
    if nodes.is_some_and(|count| count <= MAX_IMPORTED_NODES) {
        Ok(())
    } else {
        Err(DocxImportError::unsafe_input(
            ExternalSafetyReason::XmlNodeCount,
        ))
    }
}

fn canonical_node_count(node: &Value) -> usize {
    1_usize.saturating_add(
        node.get("content")
            .and_then(Value::as_array)
            .map_or(0, |content| {
                content.iter().fold(0_usize, |count, child| {
                    count.saturating_add(canonical_node_count(child))
                })
            }),
    )
}

#[derive(Default)]
struct ParagraphBuilder {
    heading_level: Option<u8>,
    page_break_before: bool,
    style: ParagraphStyleBuilder,
    content: Vec<Value>,
    seen: BTreeSet<&'static str>,
}

impl ParagraphBuilder {
    fn mark_once(&mut self, property: &'static str) -> Result<(), DocxImportError> {
        if self.seen.insert(property) {
            Ok(())
        } else {
            Err(DocxImportError::malformed())
        }
    }

    fn into_blocks(self) -> Vec<Value> {
        let mut blocks = Vec::new();
        if self.page_break_before {
            blocks.push(json!({ "type": "pageBreak" }));
        }
        let mut segment = Vec::new();
        let mut saw_inline_page_break = false;
        for node in &self.content {
            if node.get("type").and_then(Value::as_str) == Some("pageBreak") {
                if !segment.is_empty() {
                    blocks.push(self.block_json(std::mem::take(&mut segment)));
                }
                blocks.push(json!({ "type": "pageBreak" }));
                saw_inline_page_break = true;
            } else {
                segment.push(node.clone());
            }
        }
        if !segment.is_empty() || (!saw_inline_page_break && self.content.is_empty()) {
            blocks.push(self.block_json(segment));
        }
        blocks
    }

    fn block_json(&self, content: Vec<Value>) -> Value {
        let mut fields = serde_json::Map::new();
        fields.insert(
            "type".to_owned(),
            Value::String(self.block_type().to_owned()),
        );
        if let Some(attrs) = self.attrs() {
            fields.insert("attrs".to_owned(), attrs);
        }
        if !content.is_empty() {
            fields.insert("content".to_owned(), Value::Array(content));
        }
        Value::Object(fields)
    }

    fn block_type(&self) -> &'static str {
        if self.heading_level.is_some() {
            "heading"
        } else {
            "paragraph"
        }
    }

    fn attrs(&self) -> Option<Value> {
        let mut attrs = serde_json::Map::new();
        if let Some(level) = self.heading_level {
            attrs.insert("level".to_owned(), Value::from(level));
        }
        if self.style.touched {
            attrs.insert("paragraphStyle".to_owned(), self.style.to_json());
        }
        (!attrs.is_empty()).then_some(Value::Object(attrs))
    }
}

#[derive(Default)]
struct RunBuilder {
    text: String,
    content: Vec<Value>,
    formatting: RunFormatting,
    seen: BTreeSet<&'static str>,
}

impl RunBuilder {
    fn has_text(&self) -> bool {
        !self.text.is_empty()
    }

    fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let mut fields = serde_json::Map::new();
        fields.insert("type".to_owned(), Value::String("text".to_owned()));
        fields.insert("text".to_owned(), Value::String(self.text.clone()));
        let marks = self.formatting.marks();
        if !marks.is_empty() {
            fields.insert("marks".to_owned(), Value::Array(marks));
        }
        self.content.push(Value::Object(fields));
        self.text.clear();
    }

    fn mark_once(&mut self, property: &'static str) -> Result<(), DocxImportError> {
        if self.seen.insert(property) {
            Ok(())
        } else {
            Err(DocxImportError::malformed())
        }
    }
}

#[derive(Default)]
struct RunFormatting {
    bold: bool,
    font_family: Option<FontFamily>,
    font_size: Option<FontSizePoints>,
    italic: bool,
    underline: bool,
}

impl RunFormatting {
    fn marks(&self) -> Vec<Value> {
        let mut marks = Vec::new();
        push_enabled_mark(&mut marks, self.bold, "bold");
        push_enabled_mark(&mut marks, self.italic, "italic");
        push_enabled_mark(&mut marks, self.underline, "underline");
        if let Some(family) = self.font_family {
            marks.push(json!({
                "type": "fontFamily",
                "attrs": { "family": family.identifier() }
            }));
        }
        if let Some(size) = self.font_size {
            marks.push(json!({
                "type": "fontSize",
                "attrs": { "points": size.points() }
            }));
        }
        marks
    }
}

enum BooleanMark {
    Bold,
    Italic,
}

enum BreakKind {
    Line,
    Page,
    Column,
}

struct ParagraphStyleBuilder {
    alignment: &'static str,
    line_spacing_hundredths: u64,
    space_before_twips: u64,
    space_after_twips: u64,
    left_indent_twips: u64,
    right_indent_twips: u64,
    special_kind: &'static str,
    special_twips: u64,
    touched: bool,
}

impl Default for ParagraphStyleBuilder {
    fn default() -> Self {
        Self {
            alignment: "left",
            line_spacing_hundredths: 100,
            space_before_twips: 0,
            space_after_twips: 0,
            left_indent_twips: 0,
            right_indent_twips: 0,
            special_kind: "none",
            special_twips: 0,
            touched: false,
        }
    }
}

impl ParagraphStyleBuilder {
    fn apply_spacing(
        &mut self,
        attributes: &HashMap<String, String>,
    ) -> Result<(), DocxImportError> {
        if let Some(value) = attributes.get("before") {
            self.space_before_twips = bounded_twips(value, MAX_PARAGRAPH_SPACING_TWIPS)?;
            self.touched = true;
        }
        if let Some(value) = attributes.get("after") {
            self.space_after_twips = bounded_twips(value, MAX_PARAGRAPH_SPACING_TWIPS)?;
            self.touched = true;
        }
        self.apply_line_spacing(attributes)
    }

    fn apply_line_spacing(
        &mut self,
        attributes: &HashMap<String, String>,
    ) -> Result<(), DocxImportError> {
        let Some(line) = attributes.get("line") else {
            return Ok(());
        };
        match attributes
            .get("lineRule")
            .map(String::as_str)
            .unwrap_or("auto")
        {
            "auto" => self.line_spacing_hundredths = automatic_line_spacing(line)?,
            "exact" => return Err(unsupported(ExternalFeature::ExactLineSpacing)),
            "atLeast" => return Err(unsupported(ExternalFeature::AtLeastLineSpacing)),
            _ => return Err(DocxImportError::malformed()),
        }
        self.touched = true;
        Ok(())
    }

    fn apply_indentation(
        &mut self,
        attributes: &HashMap<String, String>,
    ) -> Result<(), DocxImportError> {
        if let Some(value) = attributes.get("left") {
            self.left_indent_twips = bounded_twips(value, MAX_PARAGRAPH_SPACING_TWIPS)?;
            self.touched = true;
        }
        if let Some(value) = attributes.get("right") {
            self.right_indent_twips = bounded_twips(value, MAX_PARAGRAPH_SPACING_TWIPS)?;
            self.touched = true;
        }
        self.apply_special_indent(attributes)
    }

    fn apply_special_indent(
        &mut self,
        attributes: &HashMap<String, String>,
    ) -> Result<(), DocxImportError> {
        if attributes.contains_key("firstLine") && attributes.contains_key("hanging") {
            return Err(DocxImportError::malformed());
        }
        if let Some(value) = attributes.get("firstLine") {
            self.special_kind = "first_line";
            self.special_twips = bounded_twips(value, MAX_SPECIAL_INDENT_TWIPS)?;
            self.touched = true;
        }
        if let Some(value) = attributes.get("hanging") {
            self.special_kind = "hanging";
            self.special_twips = bounded_twips(value, MAX_SPECIAL_INDENT_TWIPS)?;
            self.touched = true;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        json!({
            "schemaVersion": PARAGRAPH_STYLE_SCHEMA_VERSION,
            "alignment": self.alignment,
            "lineSpacingHundredths": self.line_spacing_hundredths,
            "spaceBeforeTwips": self.space_before_twips,
            "spaceAfterTwips": self.space_after_twips,
            "leftIndentTwips": self.left_indent_twips,
            "rightIndentTwips": self.right_indent_twips,
            "specialIndent": { "kind": self.special_kind, "twips": self.special_twips }
        })
    }
}

fn required_value(element: &BytesStart<'_>) -> Result<String, DocxImportError> {
    let values = attributes(element)?;
    if values.len() != 1 {
        return Err(DocxImportError::malformed());
    }
    values
        .get("val")
        .cloned()
        .ok_or_else(DocxImportError::malformed)
}

fn break_kind(element: &BytesStart<'_>) -> Result<BreakKind, DocxImportError> {
    let values = attributes(element)?;
    match values.get("type").map(String::as_str) {
        None | Some("textWrapping") if values.len() <= 1 => Ok(BreakKind::Line),
        Some("page") if values.len() == 1 => Ok(BreakKind::Page),
        Some("column") if values.len() == 1 => Ok(BreakKind::Column),
        _ => Err(DocxImportError::malformed()),
    }
}

fn on_off_value(element: &BytesStart<'_>) -> Result<bool, DocxImportError> {
    let values = attributes(element)?;
    if values.len() > usize::from(values.contains_key("val")) {
        return Err(DocxImportError::malformed());
    }
    match values.get("val").map(String::as_str) {
        None | Some("1" | "true" | "on" | "yes") => Ok(true),
        Some("0" | "false" | "off" | "no") => Ok(false),
        Some(_) => Err(DocxImportError::malformed()),
    }
}

fn explicit_font_family(values: &HashMap<String, String>) -> Option<FontFamily> {
    let names = ["ascii", "hAnsi", "eastAsia", "cs"]
        .iter()
        .filter_map(|name| values.get(*name))
        .collect::<BTreeSet<_>>();
    if names.len() != 1 {
        return None;
    }
    FontFamily::from_docx_name(names.first()?)
}

fn has_unsupported_font_attributes(values: &HashMap<String, String>) -> bool {
    const EXPLICIT_NAMES: [&str; 4] = ["ascii", "hAnsi", "eastAsia", "cs"];
    values
        .keys()
        .any(|name| !EXPLICIT_NAMES.contains(&name.as_str()))
}

fn push_enabled_mark(marks: &mut Vec<Value>, enabled: bool, mark_type: &'static str) {
    if enabled {
        marks.push(json!({ "type": mark_type }));
    }
}

fn attributes(element: &BytesStart<'_>) -> Result<HashMap<String, String>, DocxImportError> {
    let mut values = HashMap::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DocxImportError::malformed())?;
        let name = name_string(local_name(attribute.key.as_ref()))?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
            .map_err(|_| DocxImportError::malformed())?
            .into_owned();
        if values.insert(name, value).is_some() {
            return Err(DocxImportError::malformed());
        }
    }
    Ok(values)
}

fn reject_spacing_extensions(attributes: &HashMap<String, String>) -> Result<(), DocxImportError> {
    let allowed = ["after", "before", "line", "lineRule"];
    if attributes
        .keys()
        .all(|name| allowed.contains(&name.as_str()))
    {
        Ok(())
    } else {
        Err(unsupported(ExternalFeature::ContextualSpacing))
    }
}

fn reject_indent_extensions(attributes: &HashMap<String, String>) -> Result<(), DocxImportError> {
    let allowed = ["firstLine", "hanging", "left", "right"];
    if attributes
        .keys()
        .all(|name| allowed.contains(&name.as_str()))
    {
        Ok(())
    } else {
        Err(unsupported(ExternalFeature::ListIndentation))
    }
}

fn automatic_line_spacing(value: &str) -> Result<u64, DocxImportError> {
    let units = unsigned_integer(value)?;
    let scaled = units.checked_mul(100).ok_or_else(lossy_line_spacing)?;
    if scaled % 240 != 0 {
        return Err(lossy_line_spacing());
    }
    let hundredths = scaled / 240;
    let in_range =
        (MIN_LINE_SPACING_HUNDREDTHS..=MAX_LINE_SPACING_HUNDREDTHS).contains(&hundredths);
    if in_range && hundredths.is_multiple_of(LINE_SPACING_INCREMENT) {
        Ok(hundredths)
    } else {
        Err(lossy_line_spacing())
    }
}

fn bounded_twips(value: &str, maximum: u64) -> Result<u64, DocxImportError> {
    let twips = unsigned_integer(value)?;
    if twips <= maximum {
        Ok(twips)
    } else {
        Err(DocxImportError::lossy(vec![
            ExternalFeature::UnsupportedDocumentStructure,
        ]))
    }
}

fn unsigned_integer(value: &str) -> Result<u64, DocxImportError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DocxImportError::malformed());
    }
    value.parse().map_err(|_| DocxImportError::malformed())
}

fn heading_level(value: &str) -> Option<u8> {
    accepted_heading_level(value.strip_prefix("Heading")?)
}

fn heading_level_with_space(value: &str) -> Option<u8> {
    accepted_heading_level(value.strip_prefix("Heading ")?)
}

fn accepted_heading_level(value: &str) -> Option<u8> {
    value.parse().ok().filter(|level| (1..=6).contains(level))
}

fn resolve_reference(reference: &BytesRef<'_>) -> Result<char, DocxImportError> {
    if let Some(character) = reference
        .resolve_char_ref()
        .map_err(|_| unsafe_xml_entity())?
    {
        return Ok(character);
    }
    let value: &[u8] = reference;
    match value {
        b"amp" => Ok('&'),
        b"apos" => Ok('\''),
        b"gt" => Ok('>'),
        b"lt" => Ok('<'),
        b"quot" => Ok('"'),
        _ => Err(unsafe_xml_entity()),
    }
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn name_string(name: &[u8]) -> Result<String, DocxImportError> {
    String::from_utf8(name.to_vec()).map_err(|_| DocxImportError::malformed())
}

fn unsupported(feature: ExternalFeature) -> DocxImportError {
    DocxImportError::unsupported(vec![feature])
}

fn lossy_line_spacing() -> DocxImportError {
    DocxImportError::lossy(vec![ExternalFeature::UnsupportedDocumentStructure])
}

fn unsafe_xml_entity() -> DocxImportError {
    DocxImportError::unsafe_input(ExternalSafetyReason::XmlEntity)
}
