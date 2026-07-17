use std::io::{Cursor, Write};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::*;
use crate::{
    documents::envelope::{DOCUMENT_ENVELOPE_SCHEMA_VERSION, DocumentEnvelope},
    exports::docx::compile_docx,
    interoperability::fidelity::{ExternalFeature, ExternalSafetyReason},
};

const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#;

#[test]
fn supported_paragraph_properties_map_to_canonical_data() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:pPr><w:jc w:val="both"/><w:spacing w:lineRule="auto" w:line="276" w:before="120" w:after="240"/><w:ind w:left="360" w:right="180" w:hanging="720"/></w:pPr><w:r><w:t>Styled</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(parsed.fidelity, ExternalFidelity::Exact);
    assert_eq!(
        paragraph_style(&parsed.document),
        &json!({
            "schemaVersion": 1,
            "alignment": "justify",
            "lineSpacingHundredths": 115,
            "spaceBeforeTwips": 120,
            "spaceAfterTwips": 240,
            "leftIndentTwips": 360,
            "rightIndentTwips": 180,
            "specialIndent": { "kind": "hanging", "twips": 720 }
        })
    );
}

#[test]
fn absent_paragraph_properties_remain_absent() {
    let parsed =
        parse_docx_package(&package(&document_xml(r#"<w:r><w:t>Defaults</w:t></w:r>"#))).unwrap();

    assert_eq!(parsed.fidelity, ExternalFidelity::Exact);
    assert!(parsed.document["content"][0].get("attrs").is_none());
}

#[test]
fn word_authored_fixture_imports_visible_content() {
    let bytes = include_bytes!("../../../tests/fixtures/docx/word-custom-xml.docx");
    let digest = format!("{:x}", Sha256::digest(bytes));
    let parsed = parse_docx_package(bytes).expect("Word-authored fixture should import");

    assert_eq!(
        digest,
        "9929f84423e135a5100ab43b8c454a6734d78cfaf41eea1e4274e707c0d1cbe6"
    );
    assert_eq!(
        document_text(&parsed.document),
        [
            "DRAFT DOCX Round Trip",
            "This document was created in Microsoft Word for DRAFT interoperability testing.",
            "The visible content must survive import and export.",
        ]
        .join("\n")
    );
}

#[test]
fn alternate_heading_name_is_canonically_normalized() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:pPr><w:pStyle w:val="Heading 2"/></w:pPr><w:r><w:t>Heading</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(parsed.document["content"][0]["type"], "heading");
    assert_eq!(parsed.document["content"][0]["attrs"]["level"], 2);
    assert_eq!(
        parsed.fidelity,
        ExternalFidelity::CanonicallyNormalized {
            features: vec![ExternalFeature::AlternateHeadingStyleName],
        }
    );
}

#[test]
fn valid_unsupported_properties_require_source_preservation() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:pPr><w:pBdr/><w:shd/><w:tabs/><w:contextualSpacing/><w:keepNext/></w:pPr><w:r><w:t>Text</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(
        parsed.fidelity,
        ExternalFidelity::UnsupportedPreservable {
            features: vec![
                ExternalFeature::ContextualSpacing,
                ExternalFeature::PaginationControl,
                ExternalFeature::ParagraphBorder,
                ExternalFeature::ParagraphShading,
                ExternalFeature::ParagraphTab,
            ],
        }
    );
}

#[test]
fn supported_run_formatting_survives_unrelated_unsupported_properties() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:pPr><w:pBdr><w:top w:val="single"/></w:pBdr><w:tabs><w:tab w:val="left" w:pos="720"/></w:tabs></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Text</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(parsed.document["content"][0]["content"][0]["text"], "Text");
    assert_eq!(
        parsed.document["content"][0]["content"][0]["marks"],
        json!([{ "type": "bold" }])
    );
    assert_eq!(
        parsed.fidelity,
        ExternalFidelity::UnsupportedPreservable {
            features: vec![
                ExternalFeature::ParagraphBorder,
                ExternalFeature::ParagraphTab,
            ],
        }
    );
}

#[test]
fn supported_direct_run_properties_map_to_exact_canonical_marks() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="Times New Roman"/><w:b/><w:i/><w:u w:val="single"/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr><w:t>Formatted</w:t></w:r><w:r><w:rPr><w:b w:val="0"/><w:i w:val="false"/></w:rPr><w:t> plain</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(parsed.fidelity, ExternalFidelity::Exact);
    assert_eq!(
        parsed.document["content"][0]["content"],
        json!([
            {
                "type": "text",
                "text": "Formatted",
                "marks": [
                    { "type": "bold" },
                    { "type": "italic" },
                    { "type": "underline" },
                    { "type": "fontFamily", "attrs": { "family": "times_new_roman" } },
                    { "type": "fontSize", "attrs": { "points": 12 } }
                ]
            },
            { "type": "text", "text": " plain" }
        ])
    );
}

#[test]
fn page_break_runs_become_canonical_blocks_and_export_back_to_docx() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:r><w:t>Before</w:t><w:br w:type="page"/><w:t>After</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(parsed.fidelity, ExternalFidelity::Exact);
    assert_eq!(
        parsed.document["content"],
        json!([
            { "type": "paragraph", "content": [{ "type": "text", "text": "Before" }] },
            { "type": "pageBreak" },
            { "type": "paragraph", "content": [{ "type": "text", "text": "After" }] }
        ])
    );

    let artifact = compile_docx(&envelope_with_document(parsed.document)).unwrap();
    let reparsed = parse_docx_package(artifact.as_bytes()).unwrap();
    assert_eq!(reparsed.fidelity, ExternalFidelity::Exact);
    assert_eq!(
        reparsed.document["content"][1],
        json!({ "type": "pageBreak" })
    );
}

#[test]
fn page_break_before_is_a_disclosed_canonical_normalization() {
    let parsed = parse_docx_package(&package(&document_xml(
        r#"<w:pPr><w:pageBreakBefore/></w:pPr><w:r><w:t>Next page</w:t></w:r>"#,
    )))
    .unwrap();

    assert_eq!(
        parsed.document["content"][0],
        json!({ "type": "pageBreak" })
    );
    assert_eq!(
        parsed.fidelity,
        ExternalFidelity::CanonicallyNormalized {
            features: vec![ExternalFeature::PaginationControl],
        }
    );
}

#[test]
fn package_semantics_classify_valid_uneditable_behavior() {
    let external_relationships = DOCUMENT_RELS.replace(
        "</Relationships>",
        r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/></Relationships>"#,
    );
    let bytes = package_with_parts(vec![
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes()),
        ("word/document.xml", document_xml("").as_bytes()),
        ("word/_rels/document.xml.rels", external_relationships.as_bytes()),
        (
            "word/styles.xml",
            br#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Custom"/></w:styles>"#,
        ),
        ("word/theme/theme1.xml", b"<theme/>")
    ]);

    assert_eq!(
        parse_docx_package(&bytes).unwrap().fidelity,
        ExternalFidelity::UnsupportedPreservable {
            features: vec![
                ExternalFeature::ExternalRelationship,
                ExternalFeature::PackagePart,
                ExternalFeature::UnsupportedStyleInheritance,
            ],
        }
    );
}

#[test]
fn optional_relationship_and_style_parts_are_not_required() {
    let bytes = package_with_parts(vec![
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes()),
        ("word/document.xml", document_xml("").as_bytes()),
    ]);

    assert_eq!(
        parse_docx_package(&bytes).unwrap().fidelity,
        ExternalFidelity::Exact
    );
}

#[test]
fn exact_and_at_least_line_rules_are_unsupported_not_malformed() {
    for (rule, feature) in [
        ("exact", ExternalFeature::ExactLineSpacing),
        ("atLeast", ExternalFeature::AtLeastLineSpacing),
    ] {
        let xml = document_xml(&format!(
            r#"<w:pPr><w:spacing w:lineRule="{rule}" w:line="240"/></w:pPr>"#
        ));
        assert_eq!(
            parse_docx_package(&package(&xml)),
            Err(DocxImportError::unsupported(vec![feature]))
        );
    }
}

#[test]
fn unsupported_style_and_list_indentation_are_distinct_valid_features() {
    for (property, feature) in [
        (
            r#"<w:pStyle w:val="CustomBody"/>"#,
            ExternalFeature::UnsupportedStyleInheritance,
        ),
        (r#"<w:numPr/>"#, ExternalFeature::ListIndentation),
    ] {
        let xml = document_xml(&format!("<w:pPr>{property}</w:pPr>"));
        assert_eq!(
            parse_docx_package(&package(&xml)),
            Err(DocxImportError::unsupported(vec![feature]))
        );
    }
}

#[test]
fn malformed_properties_fail_without_fidelity_guessing() {
    for property in [
        r#"<w:jc w:val="bogus"/>"#,
        r#"<w:jc w:val="left"/><w:jc w:val="right"/>"#,
        r#"<w:ind w:firstLine="120" w:hanging="120"/>"#,
        r#"<w:spacing w:before="1.5"/>"#,
    ] {
        let xml = document_xml(&format!("<w:pPr>{property}</w:pPr>"));
        assert_eq!(
            parse_docx_package(&package(&xml)),
            Err(DocxImportError::malformed())
        );
    }
}

#[test]
fn unrepresentable_bounds_are_lossy_and_never_clamped() {
    for property in [
        r#"<w:spacing w:lineRule="auto" w:line="241"/>"#,
        r#"<w:spacing w:before="2881"/>"#,
        r#"<w:ind w:left="2881"/>"#,
        r#"<w:ind w:firstLine="1441"/>"#,
    ] {
        let xml = document_xml(&format!("<w:pPr>{property}</w:pPr>"));
        assert!(matches!(
            parse_docx_package(&package(&xml)),
            Err(DocxImportError::LossyImportDenied { .. })
        ));
    }
}

#[test]
fn exported_supported_paragraph_data_reimports_exactly() {
    let envelope = styled_envelope();
    let artifact = compile_docx(&envelope).unwrap();
    let parsed = parse_docx_package(artifact.as_bytes()).unwrap();

    assert_eq!(parsed.fidelity, ExternalFidelity::Exact);
    assert_eq!(
        paragraph_style(&parsed.document),
        paragraph_style(envelope.document())
    );
}

#[test]
fn package_and_xml_safety_fail_with_closed_reasons() {
    let traversal = package_with_parts(vec![("../word/document.xml", document_xml("").as_bytes())]);
    assert_unsafe(traversal, ExternalSafetyReason::ArchivePath);

    let entity_xml = document_xml("<!DOCTYPE x [<!ENTITY e 'bad'>]>");
    assert_unsafe(package(&entity_xml), ExternalSafetyReason::XmlDoctype);

    let unsafe_rels = ROOT_RELS.replace("word/document.xml", "../document.xml");
    let bytes = package_with_overrides(&document_xml(""), Some(&unsafe_rels));
    assert_unsafe(bytes, ExternalSafetyReason::RelationshipTarget);

    let escaping_document_rels = DOCUMENT_RELS.replace("styles.xml", "../../outside.xml");
    assert_unsafe(
        package_with_document_relationships(&escaping_document_rels),
        ExternalSafetyReason::RelationshipTarget,
    );
}

#[test]
fn malformed_package_contracts_are_not_reported_as_unsafe_content() {
    let invalid_content_type = CONTENT_TYPES.replace(DOCUMENT_CONTENT_TYPE_FOR_TESTS, "text/xml");
    let invalid_target_mode = DOCUMENT_RELS.replace(
        "Target=\"styles.xml\"",
        "Target=\"styles.xml\" TargetMode=\"Remote\"",
    );
    let duplicate_office_relationship = ROOT_RELS.replace(
        "</Relationships>",
        r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
    );

    for bytes in [
        package_with_parts(vec![
            ("[Content_Types].xml", invalid_content_type.as_bytes()),
            ("_rels/.rels", ROOT_RELS.as_bytes()),
            ("word/document.xml", document_xml("").as_bytes()),
        ]),
        package_with_document_relationships(&invalid_target_mode),
        package_with_overrides(&document_xml(""), Some(&duplicate_office_relationship)),
    ] {
        assert_eq!(
            parse_docx_package(&bytes),
            Err(DocxImportError::malformed())
        );
    }
}

#[test]
fn archive_compression_ratio_fails_closed_before_extraction() {
    let repeated = vec![0_u8; 1024 * 1024];
    let bytes = package_with_method(
        vec![
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELS.as_bytes()),
            ("word/document.xml", document_xml("").as_bytes()),
            ("word/media/repeated.bin", repeated.as_slice()),
        ],
        CompressionMethod::Deflated,
    );

    assert_unsafe(bytes, ExternalSafetyReason::CompressionRatio);
}

#[test]
fn package_resource_limits_have_stable_safety_reasons() {
    assert_unsafe(
        vec![0_u8; MAX_DOCX_IMPORT_PACKAGE_BYTES + 1],
        ExternalSafetyReason::PackageSize,
    );
    assert_unsafe(
        package_with_entry_count(MAX_DOCX_IMPORT_ENTRIES + 1),
        ExternalSafetyReason::ArchiveEntryCount,
    );
    assert_unsafe(
        package_with_duplicate_document_part(),
        ExternalSafetyReason::DuplicateEntry,
    );

    let oversized_xml = vec![b'x'; MAX_DOCX_IMPORT_XML_BYTES + 1];
    assert_unsafe(
        package_with_parts(vec![
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELS.as_bytes()),
            ("word/document.xml", oversized_xml.as_slice()),
        ]),
        ExternalSafetyReason::XmlSize,
    );

    let nested = format!(
        "{}{}",
        "<w:sdt>".repeat(MAX_DOCX_IMPORT_XML_DEPTH + 1),
        "</w:sdt>".repeat(MAX_DOCX_IMPORT_XML_DEPTH + 1)
    );
    assert_unsafe(
        package(&document_xml(&nested)),
        ExternalSafetyReason::XmlDepth,
    );
}

#[test]
fn package_fixture_is_deterministic_and_hashable() {
    let first = package(&document_xml(r#"<w:r><w:t>Evidence</w:t></w:r>"#));
    let second = package(&document_xml(r#"<w:r><w:t>Evidence</w:t></w:r>"#));
    let digest = format!("{:x}", Sha256::digest(&first));

    assert_eq!(first, second);
    assert_eq!(
        digest,
        "c284d54886d21d2fda1d0fa51099ac2db65cbaf830ce133d8f6608c21c4bf35a"
    );
}

const DOCUMENT_CONTENT_TYPE_FOR_TESTS: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";

fn styled_envelope() -> DocumentEnvelope {
    envelope_with_document(json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "attrs": { "paragraphStyle": supported_style() },
            "content": [{ "type": "text", "text": "Round trip" }]
        }]
    }))
}

fn envelope_with_document(document: Value) -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": DOCUMENT_ID,
        "title": "Round trip",
        "document": document
    }))
    .unwrap()
}

fn supported_style() -> Value {
    json!({
        "schemaVersion": 1,
        "alignment": "justify",
        "lineSpacingHundredths": 115,
        "spaceBeforeTwips": 120,
        "spaceAfterTwips": 240,
        "leftIndentTwips": 360,
        "rightIndentTwips": 180,
        "specialIndent": { "kind": "hanging", "twips": 720 }
    })
}

fn paragraph_style(document: &Value) -> &Value {
    &document["content"][0]["attrs"]["paragraphStyle"]
}

fn document_text(document: &Value) -> String {
    document["content"]
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["content"][0]["text"].as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn document_xml(paragraph_content: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p>{paragraph_content}</w:p></w:body></w:document>"#
    )
}

fn package(document_xml: &str) -> Vec<u8> {
    package_with_overrides(document_xml, None)
}

fn package_with_overrides(document_xml: &str, root_rels: Option<&str>) -> Vec<u8> {
    package_with_parts(vec![
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", root_rels.unwrap_or(ROOT_RELS).as_bytes()),
        ("word/document.xml", document_xml.as_bytes()),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS.as_bytes()),
        ("word/styles.xml", STYLES.as_bytes()),
    ])
}

fn package_with_document_relationships(document_relationships: &str) -> Vec<u8> {
    package_with_parts(vec![
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes()),
        ("word/document.xml", document_xml("").as_bytes()),
        (
            "word/_rels/document.xml.rels",
            document_relationships.as_bytes(),
        ),
        ("word/styles.xml", STYLES.as_bytes()),
    ])
}

fn package_with_parts(parts: Vec<(&str, &[u8])>) -> Vec<u8> {
    package_with_method(parts, CompressionMethod::Stored)
}

fn package_with_method(parts: Vec<(&str, &[u8])>, method: CompressionMethod) -> Vec<u8> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::DEFAULT
        .compression_method(method)
        .unix_permissions(0o644);
    for (path, contents) in parts {
        archive.start_file(path, options).unwrap();
        archive.write_all(contents).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

fn package_with_entry_count(entry_count: usize) -> Vec<u8> {
    let mut archive = base_archive();
    let options = stored_options();
    for index in 0..entry_count.saturating_sub(3) {
        archive
            .start_file(format!("extras/{index}.xml"), options)
            .unwrap();
        archive.write_all(b"<extra/>").unwrap();
    }
    archive.finish().unwrap().into_inner()
}

fn package_with_duplicate_document_part() -> Vec<u8> {
    let alias = "word/document2xml";
    let canonical = "word/document.xml";
    assert_eq!(alias.len(), canonical.len());
    let mut bytes = package_with_parts(vec![
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes()),
        (canonical, document_xml("").as_bytes()),
        (alias, document_xml("").as_bytes()),
    ]);
    for index in 0..=bytes.len() - alias.len() {
        if &bytes[index..index + alias.len()] == alias.as_bytes() {
            bytes[index..index + canonical.len()].copy_from_slice(canonical.as_bytes());
        }
    }
    bytes
}

fn base_archive() -> ZipWriter<Cursor<Vec<u8>>> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, contents) in [
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", ROOT_RELS.as_bytes()),
        ("word/document.xml", document_xml("").as_bytes()),
    ] {
        archive.start_file(path, stored_options()).unwrap();
        archive.write_all(contents).unwrap();
    }
    archive
}

fn stored_options() -> SimpleFileOptions {
    SimpleFileOptions::DEFAULT
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644)
}

fn assert_unsafe(bytes: Vec<u8>, reason: ExternalSafetyReason) {
    assert_eq!(
        parse_docx_package(&bytes),
        Err(DocxImportError::unsafe_input(reason))
    );
}
