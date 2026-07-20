use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Component, Path},
};

use quick_xml::{Reader, XmlVersion, events::Event};
use zip::ZipArchive;

use crate::docx_trace;

use super::{
    DocxImportError, ExternalFeature, ExternalSafetyReason, MAX_DOCX_IMPORT_COMPRESSION_RATIO,
    MAX_DOCX_IMPORT_ENTRIES, MAX_DOCX_IMPORT_UNCOMPRESSED_BYTES, MAX_DOCX_IMPORT_XML_BYTES,
    MAX_DOCX_IMPORT_XML_DEPTH,
};

const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";
const ROOT_RELATIONSHIPS_PATH: &str = "_rels/.rels";
const DOCUMENT_PATH: &str = "word/document.xml";
const DOCUMENT_RELATIONSHIPS_PATH: &str = "word/_rels/document.xml.rels";
const FOOTNOTES_PATH: &str = "word/footnotes.xml";
const STYLES_PATH: &str = "word/styles.xml";
const REQUIRED_PARTS: [&str; 3] = [CONTENT_TYPES_PATH, ROOT_RELATIONSHIPS_PATH, DOCUMENT_PATH];
const KNOWN_PARTS: [&str; 6] = [
    CONTENT_TYPES_PATH,
    ROOT_RELATIONSHIPS_PATH,
    DOCUMENT_PATH,
    DOCUMENT_RELATIONSHIPS_PATH,
    FOOTNOTES_PATH,
    STYLES_PATH,
];
const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const STYLES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const FOOTNOTES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";
const DOCUMENT_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";

pub(super) struct DocxPackage {
    pub(super) document_xml: Vec<u8>,
    pub(super) features: Vec<ExternalFeature>,
    pub(super) footnotes_xml: Option<Vec<u8>>,
}

pub(super) fn read_package(bytes: &[u8]) -> Result<DocxPackage, DocxImportError> {
    let declared_entries = declared_entry_count(bytes)?;
    docx_trace::emit(
        "zip_entry_count",
        format_args!("entries={declared_entries}"),
    );
    require_entry_count(declared_entries)?;
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|_| DocxImportError::malformed())?;
    require_entry_count(archive.len())?;
    if archive.len() != declared_entries {
        return Err(unsafe_error(ExternalSafetyReason::DuplicateEntry));
    }
    let (parts, total_size) = read_parts(&mut archive)?;
    docx_trace::emit(
        "uncompressed_package_size",
        format_args!("bytes={total_size}"),
    );
    trace_xml_part_sizes(&parts);
    validate_parts(&parts)
}

fn declared_entry_count(bytes: &[u8]) -> Result<usize, DocxImportError> {
    const EOCD_SIGNATURE: &[u8; 4] = b"PK\x05\x06";
    const EOCD_LENGTH: usize = 22;
    const MAX_COMMENT_LENGTH: usize = u16::MAX as usize;
    let search_start = bytes.len().saturating_sub(EOCD_LENGTH + MAX_COMMENT_LENGTH);
    let Some(offset) = (search_start..=bytes.len().saturating_sub(EOCD_LENGTH))
        .rev()
        .find(|offset| valid_eocd(bytes, *offset, EOCD_SIGNATURE))
    else {
        return Err(DocxImportError::malformed());
    };
    let entries_on_disk = read_u16(bytes, offset + 8)?;
    let total_entries = read_u16(bytes, offset + 10)?;
    if read_u16(bytes, offset + 4)? != 0
        || read_u16(bytes, offset + 6)? != 0
        || entries_on_disk != total_entries
        || total_entries == u16::MAX
    {
        return Err(unsafe_error(ExternalSafetyReason::ArchiveEntryCount));
    }
    Ok(total_entries as usize)
}

fn valid_eocd(bytes: &[u8], offset: usize, signature: &[u8; 4]) -> bool {
    let Some(header) = bytes.get(offset..offset + 22) else {
        return false;
    };
    if &header[..4] != signature {
        return false;
    }
    let comment_length = u16::from_le_bytes([header[20], header[21]]) as usize;
    offset + 22 + comment_length == bytes.len()
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, DocxImportError> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(DocxImportError::malformed)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn require_entry_count(entries: usize) -> Result<(), DocxImportError> {
    if entries <= MAX_DOCX_IMPORT_ENTRIES {
        Ok(())
    } else {
        Err(DocxImportError::unsafe_input(
            ExternalSafetyReason::ArchiveEntryCount,
        ))
    }
}

fn read_parts(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<(HashMap<String, Vec<u8>>, u64), DocxImportError> {
    let mut parts = HashMap::new();
    let mut total_size = 0_u64;
    for index in 0..archive.len() {
        read_part(archive, index, &mut total_size, &mut parts)?;
    }
    Ok((parts, total_size))
}

fn trace_xml_part_sizes(parts: &HashMap<String, Vec<u8>>) {
    for path in KNOWN_PARTS {
        if let Some(xml) = parts.get(path) {
            docx_trace::emit(
                "xml_part_size",
                format_args!("part={path} bytes={}", xml.len()),
            );
        }
    }
}

fn read_part(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    index: usize,
    total_size: &mut u64,
    parts: &mut HashMap<String, Vec<u8>>,
) -> Result<(), DocxImportError> {
    let mut file = archive
        .by_index(index)
        .map_err(|_| DocxImportError::malformed())?;
    validate_entry(&file, total_size)?;
    if file.is_dir() {
        return Ok(());
    }
    let name = file.name().to_owned();
    if parts.contains_key(&name) {
        return Err(DocxImportError::unsafe_input(
            ExternalSafetyReason::DuplicateEntry,
        ));
    }
    let contents = read_bounded_entry(&mut file)?;
    parts.insert(name, contents);
    Ok(())
}

fn validate_entry<R: Read>(
    file: &zip::read::ZipFile<'_, R>,
    total_size: &mut u64,
) -> Result<(), DocxImportError> {
    validate_entry_identity(file)?;
    validate_entry_sizes(file)?;
    *total_size = total_size
        .checked_add(file.size())
        .ok_or_else(|| unsafe_error(ExternalSafetyReason::ArchiveUncompressedSize))?;
    if *total_size > MAX_DOCX_IMPORT_UNCOMPRESSED_BYTES {
        return Err(unsafe_error(ExternalSafetyReason::ArchiveUncompressedSize));
    }
    Ok(())
}

fn validate_entry_identity<R: Read>(
    file: &zip::read::ZipFile<'_, R>,
) -> Result<(), DocxImportError> {
    let enclosed = file
        .enclosed_name()
        .ok_or_else(|| unsafe_error(ExternalSafetyReason::ArchivePath))?;
    if !is_relative_archive_path(&enclosed) {
        return Err(unsafe_error(ExternalSafetyReason::ArchivePath));
    }
    if file.is_symlink() {
        return Err(unsafe_error(ExternalSafetyReason::SymlinkEntry));
    }
    if file.encrypted() {
        return Err(unsafe_error(ExternalSafetyReason::EncryptedEntry));
    }
    Ok(())
}

fn is_relative_archive_path(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn validate_entry_sizes<R: Read>(file: &zip::read::ZipFile<'_, R>) -> Result<(), DocxImportError> {
    if file.size() > MAX_DOCX_IMPORT_UNCOMPRESSED_BYTES {
        return Err(unsafe_error(ExternalSafetyReason::ArchiveEntrySize));
    }
    let compressed = file.compressed_size();
    if file.size() > 0 && compressed == 0 {
        return Err(unsafe_error(ExternalSafetyReason::CompressionRatio));
    }
    let ratio_limit = compressed
        .checked_mul(MAX_DOCX_IMPORT_COMPRESSION_RATIO)
        .ok_or_else(|| unsafe_error(ExternalSafetyReason::CompressionRatio))?;
    if compressed > 0 && file.size() > ratio_limit {
        return Err(unsafe_error(ExternalSafetyReason::CompressionRatio));
    }
    Ok(())
}

fn read_bounded_entry<R: Read>(file: &mut R) -> Result<Vec<u8>, DocxImportError> {
    let mut contents = Vec::new();
    file.take((MAX_DOCX_IMPORT_XML_BYTES + 1) as u64)
        .read_to_end(&mut contents)
        .map_err(|_| DocxImportError::malformed())?;
    if contents.len() > MAX_DOCX_IMPORT_XML_BYTES {
        return Err(unsafe_error(ExternalSafetyReason::XmlSize));
    }
    Ok(contents)
}

fn validate_parts(parts: &HashMap<String, Vec<u8>>) -> Result<DocxPackage, DocxImportError> {
    for path in REQUIRED_PARTS {
        if !parts.contains_key(path) {
            return Err(DocxImportError::malformed());
        }
    }
    let mut features = validate_package_xml(parts)?;
    features.extend(package_features(parts));
    features.sort_unstable();
    features.dedup();
    Ok(DocxPackage {
        document_xml: parts[DOCUMENT_PATH].clone(),
        features,
        footnotes_xml: parts.get(FOOTNOTES_PATH).cloned(),
    })
}

fn validate_package_xml(
    parts: &HashMap<String, Vec<u8>>,
) -> Result<Vec<ExternalFeature>, DocxImportError> {
    for path in KNOWN_PARTS {
        if let Some(xml) = parts.get(path) {
            validate_xml_safety(xml)?;
        }
    }
    validate_content_types(&parts[CONTENT_TYPES_PATH])?;
    validate_root_relationships(&parts[ROOT_RELATIONSHIPS_PATH])?;
    let has_external_relationship = parts
        .get(DOCUMENT_RELATIONSHIPS_PATH)
        .map(|xml| validate_document_relationships(xml))
        .transpose()?
        .unwrap_or(false);
    let mut features = Vec::new();
    if has_external_relationship {
        features.push(ExternalFeature::ExternalRelationship);
    }
    if parts
        .get(STYLES_PATH)
        .map(|xml| styles_require_preservation(xml))
        .transpose()?
        .unwrap_or(false)
    {
        features.push(ExternalFeature::UnsupportedStyleInheritance);
    }
    Ok(features)
}

fn styles_require_preservation(xml: &[u8]) -> Result<bool, DocxImportError> {
    let mut reader = Reader::from_reader(xml);
    loop {
        match reader
            .read_event()
            .map_err(|_| DocxImportError::malformed())?
        {
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) == b"style" =>
            {
                if !is_canonical_style(&element)? {
                    return Ok(true);
                }
            }
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) == b"name" =>
            {
                let attributes = xml_attributes(&element)?;
                if attributes.len() != 1 || !attributes.contains_key("val") {
                    return Ok(true);
                }
            }
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) != b"styles" =>
            {
                return Ok(true);
            }
            Event::Eof => return Ok(false),
            _ => {}
        }
    }
}

fn is_canonical_style(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<bool, DocxImportError> {
    let attributes = xml_attributes(element)?;
    let Some(style_id) = attributes.get("styleId") else {
        return Ok(false);
    };
    let known_style = style_id == "Normal"
        || style_id
            .strip_prefix("Heading")
            .and_then(|level| level.parse::<u8>().ok())
            .is_some_and(|level| (1..=6).contains(&level));
    let valid_default = match attributes.get("default").map(String::as_str) {
        None => true,
        Some("1") => style_id == "Normal",
        Some(_) => false,
    };
    Ok(known_style
        && valid_default
        && attributes.get("type").map(String::as_str) == Some("paragraph")
        && attributes.len() <= 3)
}

fn package_features(parts: &HashMap<String, Vec<u8>>) -> Vec<ExternalFeature> {
    let has_extra_part = parts
        .keys()
        .any(|path| !KNOWN_PARTS.contains(&path.as_str()));
    if has_extra_part {
        vec![ExternalFeature::PackagePart]
    } else {
        vec![]
    }
}

fn validate_xml_safety(xml: &[u8]) -> Result<(), DocxImportError> {
    let mut reader = Reader::from_reader(xml);
    let mut depth = 0_usize;
    loop {
        match reader
            .read_event()
            .map_err(|_| DocxImportError::malformed())?
        {
            Event::Start(_) => increase_depth(&mut depth)?,
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::DocType(_) => return Err(unsafe_error(ExternalSafetyReason::XmlDoctype)),
            Event::GeneralRef(reference) if !is_safe_xml_reference(reference.as_ref()) => {
                return Err(unsafe_error(ExternalSafetyReason::XmlEntity));
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

fn increase_depth(depth: &mut usize) -> Result<(), DocxImportError> {
    *depth += 1;
    if *depth > MAX_DOCX_IMPORT_XML_DEPTH {
        Err(unsafe_error(ExternalSafetyReason::XmlDepth))
    } else {
        Ok(())
    }
}

fn is_safe_xml_reference(reference: &[u8]) -> bool {
    matches!(reference, b"amp" | b"apos" | b"gt" | b"lt" | b"quot") || reference.starts_with(b"#")
}

fn validate_root_relationships(xml: &[u8]) -> Result<(), DocxImportError> {
    let relationships = parse_relationships(xml)?;
    let office_relationships = relationships
        .iter()
        .filter(|relationship| relationship.kind == OFFICE_DOCUMENT_RELATIONSHIP)
        .collect::<Vec<_>>();
    let [office] = office_relationships.as_slice() else {
        return Err(DocxImportError::malformed());
    };
    require_internal_target(office, DOCUMENT_PATH)
}

fn validate_document_relationships(xml: &[u8]) -> Result<bool, DocxImportError> {
    let mut has_external_relationship = false;
    for relationship in parse_relationships(xml)? {
        if relationship.external {
            has_external_relationship = true;
            continue;
        }
        let target = resolve_relationship_target(DOCUMENT_PATH, &relationship.target)?;
        if relationship.kind == STYLES_RELATIONSHIP && target != STYLES_PATH {
            return Err(unsafe_error(ExternalSafetyReason::RelationshipTarget));
        }
        if relationship.kind == FOOTNOTES_RELATIONSHIP && target != FOOTNOTES_PATH {
            return Err(unsafe_error(ExternalSafetyReason::RelationshipTarget));
        }
    }
    Ok(has_external_relationship)
}

fn validate_content_types(xml: &[u8]) -> Result<(), DocxImportError> {
    let mut reader = Reader::from_reader(xml);
    let mut document_overrides = 0_usize;
    loop {
        match reader
            .read_event()
            .map_err(|_| DocxImportError::malformed())?
        {
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) == b"Override" =>
            {
                let attributes = xml_attributes(&element)?;
                if attributes.get("PartName").map(String::as_str) == Some("/word/document.xml") {
                    document_overrides += 1;
                    if attributes.get("ContentType").map(String::as_str)
                        != Some(DOCUMENT_CONTENT_TYPE)
                    {
                        return Err(DocxImportError::malformed());
                    }
                }
            }
            Event::Eof if document_overrides == 1 => return Ok(()),
            Event::Eof => return Err(DocxImportError::malformed()),
            _ => {}
        }
    }
}

fn require_internal_target(
    relationship: &Relationship,
    expected: &str,
) -> Result<(), DocxImportError> {
    if relationship.external || resolve_relationship_target("", &relationship.target)? != expected {
        Err(unsafe_error(ExternalSafetyReason::RelationshipTarget))
    } else {
        Ok(())
    }
}

fn resolve_relationship_target(source: &str, target: &str) -> Result<String, DocxImportError> {
    if target.is_empty() || target.contains('\\') || target.contains(':') {
        return Err(unsafe_error(ExternalSafetyReason::RelationshipTarget));
    }
    let mut segments = relationship_base_segments(source);
    resolve_target_segments(&mut segments, Path::new(target))?;
    if segments.is_empty() {
        return Err(unsafe_error(ExternalSafetyReason::RelationshipTarget));
    }
    Ok(segments.join("/"))
}

fn relationship_base_segments(source: &str) -> Vec<String> {
    source
        .rsplit_once('/')
        .map(|(parent, _)| parent.split('/').map(str::to_owned).collect())
        .unwrap_or_default()
}

fn resolve_target_segments(
    segments: &mut Vec<String>,
    target: &Path,
) -> Result<(), DocxImportError> {
    for component in target.components() {
        match component {
            Component::Normal(segment) => segments.push(segment.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir if segments.pop().is_some() => {}
            _ => return Err(unsafe_error(ExternalSafetyReason::RelationshipTarget)),
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Relationship {
    kind: String,
    target: String,
    external: bool,
}

fn parse_relationships(xml: &[u8]) -> Result<Vec<Relationship>, DocxImportError> {
    let mut reader = Reader::from_reader(xml);
    let mut relationships = Vec::new();
    loop {
        match reader
            .read_event()
            .map_err(|_| DocxImportError::malformed())?
        {
            Event::Start(element) | Event::Empty(element)
                if local_name(element.name().as_ref()) == b"Relationship" =>
            {
                relationships.push(parse_relationship(&element)?);
            }
            Event::Eof => return Ok(relationships),
            _ => {}
        }
    }
}

fn parse_relationship(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<Relationship, DocxImportError> {
    let attributes = xml_attributes(element)?;
    let external = match attributes.get("TargetMode").map(String::as_str) {
        None | Some("Internal") => false,
        Some("External") => true,
        Some(_) => return Err(DocxImportError::malformed()),
    };
    Ok(Relationship {
        kind: required_attribute(&attributes, "Type")?.to_owned(),
        target: required_attribute(&attributes, "Target")?.to_owned(),
        external,
    })
}

fn xml_attributes(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<HashMap<String, String>, DocxImportError> {
    let mut attributes = HashMap::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DocxImportError::malformed())?;
        let key = String::from_utf8(local_name(attribute.key.as_ref()).to_vec())
            .map_err(|_| DocxImportError::malformed())?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
            .map_err(|_| DocxImportError::malformed())?
            .into_owned();
        if attributes.insert(key, value).is_some() {
            return Err(DocxImportError::malformed());
        }
    }
    Ok(attributes)
}

fn required_attribute<'a>(
    attributes: &'a HashMap<String, String>,
    name: &str,
) -> Result<&'a str, DocxImportError> {
    attributes
        .get(name)
        .map(String::as_str)
        .ok_or_else(DocxImportError::malformed)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn unsafe_error(reason: ExternalSafetyReason) -> DocxImportError {
    DocxImportError::unsafe_input(reason)
}
