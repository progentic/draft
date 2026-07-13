use std::io::{Cursor, Write};

use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::documents::paragraph_format::{ParagraphStyle, SpecialIndentKind};

use super::{
    docx::DocxExportError,
    docx_model::{DocxBlock, DocxDocument, DocxInline, TextMarks},
};

const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";
const ROOT_RELATIONSHIPS_PATH: &str = "_rels/.rels";
const DOCUMENT_PATH: &str = "word/document.xml";
const DOCUMENT_RELATIONSHIPS_PATH: &str = "word/_rels/document.xml.rels";
const STYLES_PATH: &str = "word/styles.xml";
const CONTENT_TYPES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/package/2006/content-types";
const RELATIONSHIPS_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships";
const WORD_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const STYLES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const DOCUMENT_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const STYLES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";

type XmlWriter = Writer<Vec<u8>>;

struct PackagePart {
    path: &'static str,
    contents: Vec<u8>,
}

struct StyleDefinition {
    id: &'static str,
    name: &'static str,
    is_default: bool,
}

pub(super) fn build_docx_package(
    document: &DocxDocument,
    artifact_limit: usize,
) -> Result<Vec<u8>, DocxExportError> {
    let parts = build_package_parts(document)?;
    let bytes = write_package(parts)?;
    if bytes.len() > artifact_limit {
        return Err(DocxExportError::ArtifactTooLarge);
    }
    Ok(bytes)
}

fn build_package_parts(document: &DocxDocument) -> Result<Vec<PackagePart>, DocxExportError> {
    Ok(vec![
        part(CONTENT_TYPES_PATH, build_content_types_xml()?),
        part(ROOT_RELATIONSHIPS_PATH, build_root_relationships_xml()?),
        part(DOCUMENT_PATH, build_document_xml(document)?),
        part(
            DOCUMENT_RELATIONSHIPS_PATH,
            build_document_relationships_xml()?,
        ),
        part(STYLES_PATH, build_styles_xml()?),
    ])
}

fn part(path: &'static str, contents: Vec<u8>) -> PackagePart {
    PackagePart { path, contents }
}

fn write_package(parts: Vec<PackagePart>) -> Result<Vec<u8>, DocxExportError> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = package_file_options();
    for part in parts {
        archive
            .start_file(part.path, options)
            .map_err(package_failure)?;
        archive
            .write_all(&part.contents)
            .map_err(|_| DocxExportError::PackageConstructionFailed)?;
    }
    archive
        .finish()
        .map(Cursor::into_inner)
        .map_err(package_failure)
}

fn package_file_options() -> SimpleFileOptions {
    SimpleFileOptions::DEFAULT
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644)
}

fn package_failure(_error: zip::result::ZipError) -> DocxExportError {
    DocxExportError::PackageConstructionFailed
}

fn build_content_types_xml() -> Result<Vec<u8>, DocxExportError> {
    let mut writer = new_xml_writer()?;
    start(&mut writer, "Types", &[("xmlns", CONTENT_TYPES_NAMESPACE)])?;
    empty(
        &mut writer,
        "Default",
        &[
            ("Extension", "rels"),
            (
                "ContentType",
                "application/vnd.openxmlformats-package.relationships+xml",
            ),
        ],
    )?;
    empty(
        &mut writer,
        "Default",
        &[("Extension", "xml"), ("ContentType", "application/xml")],
    )?;
    empty(
        &mut writer,
        "Override",
        &[
            ("PartName", "/word/document.xml"),
            ("ContentType", DOCUMENT_CONTENT_TYPE),
        ],
    )?;
    empty(
        &mut writer,
        "Override",
        &[
            ("PartName", "/word/styles.xml"),
            ("ContentType", STYLES_CONTENT_TYPE),
        ],
    )?;
    finish_xml(writer, "Types")
}

fn build_root_relationships_xml() -> Result<Vec<u8>, DocxExportError> {
    let mut writer = relationships_writer()?;
    empty(
        &mut writer,
        "Relationship",
        &[
            ("Id", "rId1"),
            ("Type", OFFICE_DOCUMENT_RELATIONSHIP),
            ("Target", "word/document.xml"),
        ],
    )?;
    finish_xml(writer, "Relationships")
}

fn build_document_relationships_xml() -> Result<Vec<u8>, DocxExportError> {
    let mut writer = relationships_writer()?;
    empty(
        &mut writer,
        "Relationship",
        &[
            ("Id", "rId1"),
            ("Type", STYLES_RELATIONSHIP),
            ("Target", "styles.xml"),
        ],
    )?;
    finish_xml(writer, "Relationships")
}

fn relationships_writer() -> Result<XmlWriter, DocxExportError> {
    let mut writer = new_xml_writer()?;
    start(
        &mut writer,
        "Relationships",
        &[("xmlns", RELATIONSHIPS_NAMESPACE)],
    )?;
    Ok(writer)
}

fn build_document_xml(document: &DocxDocument) -> Result<Vec<u8>, DocxExportError> {
    let mut writer = new_xml_writer()?;
    start(&mut writer, "w:document", &[("xmlns:w", WORD_NAMESPACE)])?;
    start(&mut writer, "w:body", &[])?;
    for block in &document.blocks {
        write_block(&mut writer, block)?;
    }
    end(&mut writer, "w:body")?;
    finish_xml(writer, "w:document")
}

fn write_block(writer: &mut XmlWriter, block: &DocxBlock) -> Result<(), DocxExportError> {
    match block {
        DocxBlock::Paragraph { style, content } => write_paragraph(writer, None, *style, content),
        DocxBlock::Heading {
            level,
            style,
            content,
        } => write_paragraph(writer, Some(*level), *style, content),
    }
}

fn write_paragraph(
    writer: &mut XmlWriter,
    heading_level: Option<u8>,
    style: Option<ParagraphStyle>,
    content: &[DocxInline],
) -> Result<(), DocxExportError> {
    start(writer, "w:p", &[])?;
    write_paragraph_properties(writer, heading_level, style)?;
    for inline in content {
        write_inline(writer, inline)?;
    }
    end(writer, "w:p")
}

fn write_paragraph_properties(
    writer: &mut XmlWriter,
    heading_level: Option<u8>,
    style: Option<ParagraphStyle>,
) -> Result<(), DocxExportError> {
    if heading_level.is_none() && style.is_none() {
        return Ok(());
    }
    start(writer, "w:pPr", &[])?;
    write_heading_style(writer, heading_level)?;
    if let Some(style) = style {
        write_paragraph_style(writer, style)?;
    }
    end(writer, "w:pPr")
}

fn write_heading_style(
    writer: &mut XmlWriter,
    heading_level: Option<u8>,
) -> Result<(), DocxExportError> {
    let Some(level) = heading_level else {
        return Ok(());
    };
    empty(writer, "w:pStyle", &[("w:val", heading_style(level)?)])
}

fn write_paragraph_style(
    writer: &mut XmlWriter,
    style: ParagraphStyle,
) -> Result<(), DocxExportError> {
    empty(writer, "w:jc", &[("w:val", style.alignment().docx_value())])?;
    write_paragraph_spacing(writer, style)?;
    write_paragraph_indentation(writer, style)
}

fn write_paragraph_spacing(
    writer: &mut XmlWriter,
    style: ParagraphStyle,
) -> Result<(), DocxExportError> {
    let line = style.line_spacing_docx_units().to_string();
    let before = style.space_before_twips().to_string();
    let after = style.space_after_twips().to_string();
    empty(
        writer,
        "w:spacing",
        &[
            ("w:lineRule", "auto"),
            ("w:line", line.as_str()),
            ("w:before", before.as_str()),
            ("w:after", after.as_str()),
        ],
    )
}

fn write_paragraph_indentation(
    writer: &mut XmlWriter,
    style: ParagraphStyle,
) -> Result<(), DocxExportError> {
    let left = style.left_indent_twips().to_string();
    let right = style.right_indent_twips().to_string();
    let special = style.special_indent().twips().to_string();
    let mut attributes = vec![("w:left", left.as_str()), ("w:right", right.as_str())];
    match style.special_indent().kind() {
        SpecialIndentKind::None => {}
        SpecialIndentKind::FirstLine => attributes.push(("w:firstLine", special.as_str())),
        SpecialIndentKind::Hanging => attributes.push(("w:hanging", special.as_str())),
    }
    empty(writer, "w:ind", &attributes)
}

fn heading_style(level: u8) -> Result<&'static str, DocxExportError> {
    match level {
        1 => Ok("Heading1"),
        2 => Ok("Heading2"),
        3 => Ok("Heading3"),
        4 => Ok("Heading4"),
        5 => Ok("Heading5"),
        6 => Ok("Heading6"),
        _ => Err(DocxExportError::PackageConstructionFailed),
    }
}

fn write_inline(writer: &mut XmlWriter, inline: &DocxInline) -> Result<(), DocxExportError> {
    match inline {
        DocxInline::Text { value, marks } => write_text_run(writer, value, *marks),
        DocxInline::HardBreak => write_hard_break(writer),
    }
}

fn write_text_run(
    writer: &mut XmlWriter,
    value: &str,
    marks: TextMarks,
) -> Result<(), DocxExportError> {
    start(writer, "w:r", &[])?;
    write_run_properties(writer, marks)?;
    text_element(writer, "w:t", &[("xml:space", "preserve")], value)?;
    end(writer, "w:r")
}

fn write_run_properties(writer: &mut XmlWriter, marks: TextMarks) -> Result<(), DocxExportError> {
    if !has_run_properties(marks) {
        return Ok(());
    }
    start(writer, "w:rPr", &[])?;
    write_font_properties(writer, marks)?;
    write_enabled_marks(writer, marks)?;
    end(writer, "w:rPr")
}

fn has_run_properties(marks: TextMarks) -> bool {
    marks.bold
        || marks.italic
        || marks.underline
        || marks.font_family.is_some()
        || marks.font_size.is_some()
}

fn write_font_properties(writer: &mut XmlWriter, marks: TextMarks) -> Result<(), DocxExportError> {
    if let Some(family) = marks.font_family {
        let name = family.docx_name();
        empty(
            writer,
            "w:rFonts",
            &[
                ("w:ascii", name),
                ("w:hAnsi", name),
                ("w:eastAsia", name),
                ("w:cs", name),
            ],
        )?;
    }
    if let Some(size) = marks.font_size {
        write_font_size(writer, size.half_points())?;
    }
    Ok(())
}

fn write_font_size(writer: &mut XmlWriter, half_points: u16) -> Result<(), DocxExportError> {
    let value = half_points.to_string();
    empty(writer, "w:sz", &[("w:val", value.as_str())])?;
    empty(writer, "w:szCs", &[("w:val", value.as_str())])
}

fn write_enabled_marks(writer: &mut XmlWriter, marks: TextMarks) -> Result<(), DocxExportError> {
    if marks.bold {
        empty(writer, "w:b", &[])?;
    }
    if marks.italic {
        empty(writer, "w:i", &[])?;
    }
    if marks.underline {
        empty(writer, "w:u", &[("w:val", "single")])?;
    }
    Ok(())
}

fn write_hard_break(writer: &mut XmlWriter) -> Result<(), DocxExportError> {
    start(writer, "w:r", &[])?;
    empty(writer, "w:br", &[])?;
    end(writer, "w:r")
}

fn build_styles_xml() -> Result<Vec<u8>, DocxExportError> {
    let mut writer = new_xml_writer()?;
    start(&mut writer, "w:styles", &[("xmlns:w", WORD_NAMESPACE)])?;
    for definition in style_definitions() {
        write_style(&mut writer, definition)?;
    }
    finish_xml(writer, "w:styles")
}

fn style_definitions() -> [StyleDefinition; 7] {
    [
        StyleDefinition {
            id: "Normal",
            name: "Normal",
            is_default: true,
        },
        StyleDefinition {
            id: "Heading1",
            name: "heading 1",
            is_default: false,
        },
        StyleDefinition {
            id: "Heading2",
            name: "heading 2",
            is_default: false,
        },
        StyleDefinition {
            id: "Heading3",
            name: "heading 3",
            is_default: false,
        },
        StyleDefinition {
            id: "Heading4",
            name: "heading 4",
            is_default: false,
        },
        StyleDefinition {
            id: "Heading5",
            name: "heading 5",
            is_default: false,
        },
        StyleDefinition {
            id: "Heading6",
            name: "heading 6",
            is_default: false,
        },
    ]
}

fn write_style(writer: &mut XmlWriter, definition: StyleDefinition) -> Result<(), DocxExportError> {
    let mut attributes = vec![("w:type", "paragraph"), ("w:styleId", definition.id)];
    if definition.is_default {
        attributes.push(("w:default", "1"));
    }
    start(writer, "w:style", &attributes)?;
    empty(writer, "w:name", &[("w:val", definition.name)])?;
    end(writer, "w:style")
}

fn new_xml_writer() -> Result<XmlWriter, DocxExportError> {
    let mut writer = Writer::new(Vec::new());
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(xml_failure)?;
    Ok(writer)
}

fn finish_xml(mut writer: XmlWriter, root: &str) -> Result<Vec<u8>, DocxExportError> {
    end(&mut writer, root)?;
    Ok(writer.into_inner())
}

fn start(
    writer: &mut XmlWriter,
    name: &str,
    attributes: &[(&str, &str)],
) -> Result<(), DocxExportError> {
    let mut element = BytesStart::new(name);
    element.extend_attributes(attributes.iter().copied());
    writer
        .write_event(Event::Start(element))
        .map_err(xml_failure)
}

fn empty(
    writer: &mut XmlWriter,
    name: &str,
    attributes: &[(&str, &str)],
) -> Result<(), DocxExportError> {
    let mut element = BytesStart::new(name);
    element.extend_attributes(attributes.iter().copied());
    writer
        .write_event(Event::Empty(element))
        .map_err(xml_failure)
}

fn text_element(
    writer: &mut XmlWriter,
    name: &str,
    attributes: &[(&str, &str)],
    value: &str,
) -> Result<(), DocxExportError> {
    start(writer, name, attributes)?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(xml_failure)?;
    end(writer, name)
}

fn end(writer: &mut XmlWriter, name: &str) -> Result<(), DocxExportError> {
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(xml_failure)
}

fn xml_failure(_error: std::io::Error) -> DocxExportError {
    DocxExportError::PackageConstructionFailed
}
