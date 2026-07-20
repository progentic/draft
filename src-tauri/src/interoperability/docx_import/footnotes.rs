use std::collections::HashMap;

use quick_xml::{
    Reader, XmlVersion,
    events::{BytesRef, BytesStart, BytesText, Event},
};

use super::{DocxImportError, ExternalSafetyReason};

const MAX_FOOTNOTES: usize = 4_096;
const MAX_FOOTNOTE_NODES: usize = 100_000;
const MAX_FOOTNOTE_TEXT_BYTES: usize = 1_048_576;

#[derive(Default)]
pub(super) struct FootnoteCatalog {
    notes: HashMap<String, String>,
}

impl FootnoteCatalog {
    pub(super) fn parse(xml: Option<&[u8]>) -> Result<Self, DocxImportError> {
        xml.map(FootnoteParser::parse)
            .unwrap_or_else(|| Ok(Self::default()))
    }

    pub(super) fn text(&self, id: &str) -> Option<&str> {
        self.notes.get(id).map(String::as_str)
    }

    pub(super) fn len(&self) -> usize {
        self.notes.len()
    }
}

#[derive(Default)]
struct FootnoteParser {
    catalog: FootnoteCatalog,
    current: Option<FootnoteBuilder>,
    in_text: bool,
    nodes: usize,
    stack: Vec<String>,
    text_bytes: usize,
}

impl FootnoteParser {
    fn parse(xml: &[u8]) -> Result<FootnoteCatalog, DocxImportError> {
        Self::default().parse_events(xml)
    }

    fn parse_events(mut self, xml: &[u8]) -> Result<FootnoteCatalog, DocxImportError> {
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
        self.increment_nodes()?;
        self.handle_start(&name, element)?;
        self.stack.push(name_string(&name)?);
        Ok(())
    }

    fn empty(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        let name = local_name(element.name().as_ref()).to_vec();
        self.increment_nodes()?;
        self.handle_start(&name, element)?;
        if name == b"footnote" {
            self.finish_note()?;
        }
        if name == b"t" {
            self.in_text = false;
        }
        Ok(())
    }

    fn end(&mut self, name: &[u8]) -> Result<(), DocxImportError> {
        let expected = self.stack.pop().ok_or_else(DocxImportError::malformed)?;
        if expected.as_bytes() != name {
            return Err(DocxImportError::malformed());
        }
        match name {
            b"footnote" => self.finish_note(),
            b"t" => {
                self.in_text = false;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_start(
        &mut self,
        name: &[u8],
        element: &BytesStart<'_>,
    ) -> Result<(), DocxImportError> {
        match name {
            b"footnote" => self.start_note(element),
            b"p" => self.start_paragraph(),
            b"t" => {
                self.in_text = true;
                Ok(())
            }
            b"tab" => self.push_text(" "),
            b"br" => self.push_text("\n"),
            _ => Ok(()),
        }
    }

    fn start_note(&mut self, element: &BytesStart<'_>) -> Result<(), DocxImportError> {
        if self.current.is_some() {
            return Err(DocxImportError::malformed());
        }
        self.current = Some(FootnoteBuilder::from_element(element)?);
        Ok(())
    }

    fn start_paragraph(&mut self) -> Result<(), DocxImportError> {
        let Some(note) = self.current.as_ref() else {
            return Ok(());
        };
        let needs_break = note.saw_paragraph && !note.text.ends_with('\n');
        if needs_break {
            self.push_text("\n")?;
        }
        self.current
            .as_mut()
            .expect("footnote checked above")
            .saw_paragraph = true;
        Ok(())
    }

    fn text(&mut self, text: &BytesText<'_>) -> Result<(), DocxImportError> {
        if !self.in_text {
            return Ok(());
        }
        let decoded = text.decode().map_err(|_| DocxImportError::malformed())?;
        self.push_text(&decoded)
    }

    fn reference(&mut self, reference: &BytesRef<'_>) -> Result<(), DocxImportError> {
        if !self.in_text {
            return Err(unsafe_xml_entity());
        }
        self.push_text(&resolve_reference(reference)?.to_string())
    }

    fn push_text(&mut self, text: &str) -> Result<(), DocxImportError> {
        let Some(note) = self.current.as_mut() else {
            return Ok(());
        };
        self.text_bytes = self
            .text_bytes
            .checked_add(text.len())
            .ok_or_else(xml_size_error)?;
        if self.text_bytes > MAX_FOOTNOTE_TEXT_BYTES {
            return Err(xml_size_error());
        }
        note.text.push_str(text);
        Ok(())
    }

    fn finish_note(&mut self) -> Result<(), DocxImportError> {
        let note = self.current.take().ok_or_else(DocxImportError::malformed)?;
        if note.reserved {
            return Ok(());
        }
        let text = note.text.trim().to_owned();
        if self.catalog.notes.len() >= MAX_FOOTNOTES
            || self.catalog.notes.insert(note.id, text).is_some()
        {
            return Err(DocxImportError::malformed());
        }
        Ok(())
    }

    fn increment_nodes(&mut self) -> Result<(), DocxImportError> {
        self.nodes += 1;
        if self.nodes > MAX_FOOTNOTE_NODES {
            Err(DocxImportError::unsafe_input(
                ExternalSafetyReason::XmlNodeCount,
            ))
        } else {
            Ok(())
        }
    }

    fn finish(self) -> Result<FootnoteCatalog, DocxImportError> {
        if self.stack.is_empty() && self.current.is_none() && !self.in_text {
            Ok(self.catalog)
        } else {
            Err(DocxImportError::malformed())
        }
    }
}

struct FootnoteBuilder {
    id: String,
    reserved: bool,
    saw_paragraph: bool,
    text: String,
}

impl FootnoteBuilder {
    fn from_element(element: &BytesStart<'_>) -> Result<Self, DocxImportError> {
        let attributes = xml_attributes(element)?;
        let id = attributes
            .get("id")
            .filter(|value| valid_note_id(value))
            .cloned()
            .ok_or_else(DocxImportError::malformed)?;
        let reserved = matches!(
            attributes.get("type").map(String::as_str),
            Some("separator" | "continuationSeparator" | "continuationNotice")
        ) || id.starts_with('-')
            || id == "0";
        Ok(Self {
            id,
            reserved,
            saw_paragraph: false,
            text: String::new(),
        })
    }
}

fn xml_attributes(element: &BytesStart<'_>) -> Result<HashMap<String, String>, DocxImportError> {
    let mut attributes = HashMap::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DocxImportError::malformed())?;
        let name = name_string(local_name(attribute.key.as_ref()))?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
            .map_err(|_| DocxImportError::malformed())?
            .into_owned();
        if attributes.insert(name, value).is_some() {
            return Err(DocxImportError::malformed());
        }
    }
    Ok(attributes)
}

fn valid_note_id(value: &str) -> bool {
    value
        .strip_prefix('-')
        .unwrap_or(value)
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        && value != "-"
        && !value.is_empty()
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

fn xml_size_error() -> DocxImportError {
    DocxImportError::unsafe_input(ExternalSafetyReason::XmlSize)
}

fn unsafe_xml_entity() -> DocxImportError {
    DocxImportError::unsafe_input(ExternalSafetyReason::XmlEntity)
}
