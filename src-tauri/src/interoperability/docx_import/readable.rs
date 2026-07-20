use quick_xml::{
    Reader, XmlVersion,
    events::{BytesRef, BytesStart, BytesText, Event},
};
use serde_json::{Value, json};

use super::{
    DocxImportError, ExternalFeature, ExternalSafetyReason, FidelityAccumulator,
    document::MAX_IMPORTED_NODES, footnotes::FootnoteCatalog,
};

pub(super) fn parse_readable_document(
    xml: &[u8],
    footnotes: &FootnoteCatalog,
    fidelity: &mut FidelityAccumulator,
) -> Result<Value, DocxImportError> {
    ReadableDocumentParser::new(footnotes, fidelity).parse(xml)
}

struct ReadableDocumentParser<'a> {
    blocks: Vec<Value>,
    content: Vec<Value>,
    fidelity: &'a mut FidelityAccumulator,
    footnote_ids: Vec<String>,
    footnotes: &'a FootnoteCatalog,
    paragraph_depth: usize,
    stack: Vec<String>,
    suppressed_depth: usize,
    xml_nodes: usize,
}

impl<'a> ReadableDocumentParser<'a> {
    fn new(footnotes: &'a FootnoteCatalog, fidelity: &'a mut FidelityAccumulator) -> Self {
        Self {
            blocks: Vec::new(),
            content: Vec::new(),
            fidelity,
            footnote_ids: Vec::new(),
            footnotes,
            paragraph_depth: 0,
            stack: Vec::new(),
            suppressed_depth: 0,
            xml_nodes: 0,
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
        self.increment_xml_nodes()?;
        self.handle_start(&name, element)?;
        self.stack.push(name_string(&name)?);
        Ok(())
    }

    fn empty(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let name = local_name(element.name().as_ref()).to_vec();
        self.increment_xml_nodes()?;
        self.handle_start(&name, element)?;
        self.handle_end(&name)
    }

    fn end(&mut self, name: &[u8]) -> Result<(), DocxImportError> {
        let expected = self.stack.pop().ok_or_else(DocxImportError::malformed)?;
        if expected.as_bytes() != name {
            return Err(DocxImportError::malformed());
        }
        self.handle_end(name)
    }

    fn handle_start(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        if self.suppressed_depth > 0 {
            self.suppressed_depth += 1;
            return Ok(());
        }
        match name {
            b"del" | b"moveFrom" | b"instrText" => self.suppressed_depth = 1,
            b"p" => self.start_paragraph(),
            b"tab" => self.push_text(" "),
            b"br" => self.push_break(element)?,
            b"footnoteReference" => self.push_footnote_reference(element)?,
            b"tbl" => self.record_lossy(ExternalFeature::TableStructure),
            b"drawing" | b"pict" | b"object" | b"altChunk" => {
                self.record_lossy(ExternalFeature::UnsupportedDocumentStructure);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_end(&mut self, name: &[u8]) -> Result<(), DocxImportError> {
        if self.suppressed_depth > 0 {
            self.suppressed_depth -= 1;
            return Ok(());
        }
        if name == b"p" {
            self.finish_paragraph()?;
        }
        Ok(())
    }

    fn start_paragraph(&mut self) {
        if self.paragraph_depth > 0 && !self.content.is_empty() {
            self.content.push(json!({ "type": "hardBreak" }));
        }
        self.paragraph_depth += 1;
    }

    fn finish_paragraph(&mut self) -> Result<(), DocxImportError> {
        self.paragraph_depth = self
            .paragraph_depth
            .checked_sub(1)
            .ok_or_else(DocxImportError::malformed)?;
        if self.paragraph_depth == 0 {
            self.flush_paragraph();
        }
        Ok(())
    }

    fn text(&mut self, text: &BytesText<'_>) -> Result<(), DocxImportError> {
        if self.suppressed_depth > 0 || self.stack.last().is_none_or(|name| name != "t") {
            return Ok(());
        }
        let decoded = text.decode().map_err(|_| DocxImportError::malformed())?;
        self.push_text(&decoded);
        Ok(())
    }

    fn reference(&mut self, reference: &BytesRef<'_>) -> Result<(), DocxImportError> {
        if self.suppressed_depth > 0 {
            return Ok(());
        }
        if self.stack.last().is_none_or(|name| name != "t") {
            return Err(unsafe_xml_entity());
        }
        self.push_text(&resolve_reference(reference)?.to_string());
        Ok(())
    }

    fn push_text(&mut self, text: &str) {
        if self.paragraph_depth == 0 {
            self.paragraph_depth = 1;
        }
        if text.is_empty() {
            return;
        }
        if let Some(current) = self
            .content
            .last_mut()
            .filter(|node| node.get("type").and_then(Value::as_str) == Some("text"))
        {
            let existing = current["text"].as_str().unwrap_or_default();
            current["text"] = Value::String(format!("{existing}{text}"));
        } else {
            self.content.push(json!({ "type": "text", "text": text }));
        }
    }

    fn push_break(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        if break_type(element)?.as_deref() == Some("page") {
            self.flush_paragraph();
            self.blocks.push(json!({ "type": "pageBreak" }));
            return Ok(());
        }
        self.content.push(json!({ "type": "hardBreak" }));
        Ok(())
    }

    fn push_footnote_reference(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let id = required_id(element)?;
        if self.footnotes.text(&id).is_none() {
            return Err(DocxImportError::malformed());
        }
        self.record_lossy(ExternalFeature::Footnote);
        self.push_text(&format!("[{id}]"));
        if !self.footnote_ids.contains(&id) {
            self.footnote_ids.push(id);
        }
        Ok(())
    }

    fn flush_paragraph(&mut self) {
        let content = std::mem::take(&mut self.content);
        let paragraph = if content.is_empty() {
            json!({ "type": "paragraph" })
        } else {
            json!({ "type": "paragraph", "content": content })
        };
        self.blocks.push(paragraph);
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

    fn finish(mut self) -> Result<Value, DocxImportError> {
        if !self.stack.is_empty() || self.suppressed_depth != 0 {
            return Err(DocxImportError::malformed());
        }
        if !self.content.is_empty() {
            self.flush_paragraph();
        }
        self.append_footnotes()?;
        require_readable_content(&self.blocks)?;
        require_canonical_node_count(&self.blocks)?;
        Ok(json!({ "type": "doc", "content": self.blocks }))
    }

    fn increment_xml_nodes(&mut self) -> Result<(), DocxImportError> {
        self.xml_nodes += 1;
        if self.xml_nodes > MAX_IMPORTED_NODES {
            Err(DocxImportError::unsafe_input(
                ExternalSafetyReason::XmlNodeCount,
            ))
        } else {
            Ok(())
        }
    }

    fn record_lossy(&mut self, feature: ExternalFeature) {
        self.fidelity.record_lossy(feature);
    }
}

fn require_readable_content(blocks: &[Value]) -> Result<(), DocxImportError> {
    if blocks.iter().any(block_has_visible_content) {
        return Ok(());
    }
    Err(DocxImportError::lossy(vec![
        ExternalFeature::UnsupportedDocumentStructure,
    ]))
}

fn block_has_visible_content(block: &Value) -> bool {
    block.get("type").and_then(Value::as_str) == Some("pageBreak")
        || block
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|content| !content.is_empty())
}

fn require_canonical_node_count(blocks: &[Value]) -> Result<(), DocxImportError> {
    let nodes = blocks.iter().try_fold(0_usize, |count, block| {
        count.checked_add(canonical_node_count(block))
    });
    if nodes.is_some_and(|count| count <= MAX_IMPORTED_NODES) {
        return Ok(());
    }
    Err(DocxImportError::unsafe_input(
        ExternalSafetyReason::XmlNodeCount,
    ))
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

fn required_id(element: &BytesStart<'_>) -> Result<String, DocxImportError> {
    let mut id = None;
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DocxImportError::malformed())?;
        if local_name(attribute.key.as_ref()) != b"id" || id.is_some() {
            return Err(DocxImportError::malformed());
        }
        id = Some(
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .map_err(|_| DocxImportError::malformed())?
                .into_owned(),
        );
    }
    id.ok_or_else(DocxImportError::malformed)
}

fn break_type(element: &BytesStart<'_>) -> Result<Option<String>, DocxImportError> {
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DocxImportError::malformed())?;
        if local_name(attribute.key.as_ref()) == b"type" {
            return attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .map(|value| Some(value.into_owned()))
                .map_err(|_| DocxImportError::malformed());
        }
    }
    Ok(None)
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

fn unsafe_xml_entity() -> DocxImportError {
    DocxImportError::unsafe_input(ExternalSafetyReason::XmlEntity)
}
