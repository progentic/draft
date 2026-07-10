use std::{
    cell::Cell,
    fs,
    io::{Cursor, Read},
    path::Path,
};

use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};
use zip::{CompressionMethod, ZipArchive};

use super::*;
use crate::{
    documents::{
        atomic_write::{AtomicDocumentWriteError, write_document_atomically},
        envelope::DocumentEnvelope,
        test_support::TestDocumentPath,
    },
    exports::docx_model::require_test_resource_limits,
};

const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
const PACKAGE_PATHS: [&str; 5] = [
    "[Content_Types].xml",
    "_rels/.rels",
    "word/document.xml",
    "word/_rels/document.xml.rels",
    "word/styles.xml",
];

#[test]
fn package_has_stable_safe_entries_and_reopens() {
    let artifact = compile_docx(&minimal_envelope()).unwrap();
    let mut archive = open_archive(&artifact);

    assert_eq!(archive.file_names().collect::<Vec<_>>(), PACKAGE_PATHS);
    for index in 0..archive.len() {
        let file = archive.by_index(index).unwrap();
        assert_eq!(file.compression(), CompressionMethod::Stored);
        assert!(file.enclosed_name().is_some());
        assert!(!file.is_dir());
    }
}

#[test]
fn equal_documents_compile_to_equal_bytes() {
    let document = rich_envelope();

    let first = compile_docx(&document).unwrap();
    let second = compile_docx(&document).unwrap();

    assert_eq!(first, second);
    assert!(!first.is_empty());
}

#[test]
fn every_package_xml_part_is_well_formed() {
    let artifact = compile_docx(&rich_envelope()).unwrap();
    let mut archive = open_archive(&artifact);

    for path in PACKAGE_PATHS {
        assert_well_formed_xml(&read_part(&mut archive, path));
    }
}

#[test]
fn unicode_headings_breaks_and_marks_render_without_raw_markup() {
    let artifact = compile_docx(&rich_envelope()).unwrap();
    let mut archive = open_archive(&artifact);
    let xml = read_part(&mut archive, "word/document.xml");

    assert!(xml.contains("Heading2"));
    assert!(xml.contains("Café &amp; &lt;review&gt;"));
    assert!(!xml.contains("Café & <review>"));
    assert!(xml.contains("<w:b/>"));
    assert!(xml.contains("<w:i/>"));
    assert!(xml.contains("<w:u w:val=\"single\"/>"));
    assert!(xml.contains("<w:br/>"));
    assert_source_order(&xml, &["Heading text", "Café", "After break"]);
}

#[test]
fn empty_paragraphs_and_headings_are_preserved() {
    let document = envelope(document(vec![
        json!({ "type": "paragraph" }),
        json!({ "type": "heading", "attrs": { "level": 1 }, "content": [] }),
    ]));
    let artifact = compile_docx(&document).unwrap();
    let mut archive = open_archive(&artifact);
    let xml = read_part(&mut archive, "word/document.xml");

    assert_eq!(xml.matches("<w:p>").count(), 2);
    assert!(xml.contains("<w:pStyle w:val=\"Heading1\"/>"));
}

#[test]
fn unknown_fields_nodes_and_marks_fail_without_silent_omission() {
    let cases = [
        document(vec![json!({ "type": "paragraph", "unknown": true })]),
        document(vec![json!({ "type": "bulletList", "content": [] })]),
        document(vec![paragraph(vec![json!({
            "type": "text",
            "text": "linked",
            "marks": [{ "type": "link" }]
        })])]),
    ];

    for value in cases {
        assert!(matches!(
            compile_docx(&envelope(value)),
            Err(DocxExportError::UnsupportedDocumentContent { .. })
        ));
    }
}

#[test]
fn malformed_nested_shapes_and_xml_controls_fail_typed() {
    let cases = [
        document(vec![json!({ "type": "paragraph", "content": "invalid" })]),
        document(vec![json!({ "type": "heading", "attrs": { "level": 7 } })]),
        document(vec![paragraph(vec![json!({ "type": "text", "text": 7 })])]),
        document(vec![paragraph(vec![
            json!({ "type": "text", "text": "bad\u{1}" }),
        ])]),
    ];

    for value in cases {
        assert!(matches!(
            compile_docx(&envelope(value)),
            Err(DocxExportError::InvalidDocumentStructure { .. })
        ));
    }
}

#[test]
fn citation_nodes_fail_instead_of_exporting_editor_markers() {
    let citation = json!({
        "type": "citation",
        "attrs": {
            "schema_version": 1,
            "citekey": "smith2025",
            "render_style": "apa7"
        }
    });
    let error = compile_docx(&envelope(document(vec![paragraph(vec![citation])]))).unwrap_err();

    assert_eq!(
        error,
        DocxExportError::UnsupportedCitation {
            path: DocxContentPath::root().child(0).child(0)
        }
    );
}

#[test]
fn source_byte_node_and_depth_limits_fail_before_parsing() {
    let value = document(vec![paragraph(vec![json!({
        "type": "text",
        "text": "bounded"
    })])]);

    assert_eq!(
        require_test_resource_limits(&value, 1, 100, 16),
        Err(DocxExportError::SourceTooLarge)
    );
    assert_eq!(
        require_test_resource_limits(&value, 1024, 2, 16),
        Err(DocxExportError::TooManyNodes)
    );
    assert_eq!(
        require_test_resource_limits(&value, 1024, 100, 2),
        Err(DocxExportError::NestingTooDeep)
    );
}

#[test]
fn compiled_artifact_limit_fails_before_filesystem_work() {
    assert_eq!(
        compile_docx_with_limit(&minimal_envelope(), 1),
        Err(DocxExportError::ArtifactTooLarge)
    );
}

#[test]
fn target_validation_precedes_compilation_and_write() {
    let called = Cell::new(false);
    let result = export_docx_with_writer(&minimal_envelope(), Path::new("report.txt"), |_, _| {
        called.set(true);
        Ok(())
    });

    assert_eq!(result, Err(DocxExportError::InvalidTarget));
    assert!(!called.get());
}

#[test]
fn uppercase_docx_extension_is_accepted_for_rust_owned_targets() {
    let called = Cell::new(false);
    let outcome = export_docx_with_writer(&minimal_envelope(), Path::new("report.DOCX"), |_, _| {
        called.set(true);
        Ok(())
    })
    .unwrap();

    assert!(called.get());
    assert!(outcome.bytes_written() > 0);
}

#[test]
fn atomic_export_creates_and_replaces_target_without_changing_source() {
    let source = TestDocumentPath::new("docx-source");
    source.write(b"complete DRAFT source");
    let target = source.path().with_extension("docx");

    export_docx(&minimal_envelope(), &target).unwrap();
    let first = fs::read(&target).unwrap();
    export_docx(&rich_envelope(), &target).unwrap();
    let second = fs::read(&target).unwrap();

    assert_ne!(first, second);
    assert_eq!(second, compile_docx(&rich_envelope()).unwrap().as_bytes());
    assert_eq!(fs::read(source.path()).unwrap(), b"complete DRAFT source");
}

#[test]
fn compilation_failure_preserves_prior_complete_export() {
    let source = TestDocumentPath::new("docx-failed-compile");
    let target = source.path().with_extension("docx");
    write_document_atomically(&target, b"prior complete DOCX").unwrap();
    let unsupported = envelope(document(vec![json!({ "type": "table" })]));

    assert!(matches!(
        export_docx(&unsupported, &target),
        Err(DocxExportError::UnsupportedDocumentContent { .. })
    ));
    assert_eq!(fs::read(target).unwrap(), b"prior complete DOCX");
}

#[test]
fn atomic_write_failures_map_to_closed_export_stages() {
    let cases = [
        (
            AtomicDocumentWriteError::OpenTemporaryFile,
            DocxWriteStage::OpenTemporaryFile,
        ),
        (
            AtomicDocumentWriteError::WriteTemporaryFile,
            DocxWriteStage::WriteTemporaryFile,
        ),
        (
            AtomicDocumentWriteError::SyncTemporaryFile,
            DocxWriteStage::SyncTemporaryFile,
        ),
        (
            AtomicDocumentWriteError::ReplaceTarget,
            DocxWriteStage::ReplaceTarget,
        ),
        (
            AtomicDocumentWriteError::CleanupTemporaryFile,
            DocxWriteStage::CleanupTemporaryFile,
        ),
    ];

    for (cause, stage) in cases {
        assert_eq!(failed_write(cause), DocxExportError::WriteFailed { stage });
    }
}

#[test]
fn post_replacement_sync_failure_is_durability_uncertain() {
    assert_eq!(
        failed_write(AtomicDocumentWriteError::SyncParentDirectory),
        DocxExportError::DurabilityUncertain
    );
}

#[test]
fn errors_are_content_free_and_structural_paths_are_bounded() {
    let source_text = "private source text";
    let text = json!({
        "type": "text",
        "text": source_text,
        "unknown": true
    });
    let document = envelope(document(vec![paragraph(vec![text])]));
    let error = compile_docx(&document).unwrap_err();

    assert_eq!(
        error.to_string(),
        "document contains content not supported by DOCX export"
    );
    assert!(!error.to_string().contains(source_text));
    let DocxExportError::UnsupportedDocumentContent { path } = error else {
        panic!("expected unsupported content path");
    };
    assert_eq!(path.indexes(), [0, 0]);
}

#[test]
fn relationships_contain_no_external_targets_or_active_content() {
    let artifact = compile_docx(&minimal_envelope()).unwrap();
    let mut archive = open_archive(&artifact);
    let root_relationships = read_part(&mut archive, "_rels/.rels");
    let document_relationships = read_part(&mut archive, "word/_rels/document.xml.rels");

    assert!(!root_relationships.contains("TargetMode"));
    assert!(!document_relationships.contains("TargetMode"));
    assert!(!PACKAGE_PATHS.iter().any(|path| path.ends_with(".bin")));
    assert!(!PACKAGE_PATHS.iter().any(|path| path.contains("vbaProject")));
}

fn failed_write(cause: AtomicDocumentWriteError) -> DocxExportError {
    export_docx_with_writer(&minimal_envelope(), Path::new("report.docx"), |_, _| {
        Err(cause)
    })
    .unwrap_err()
}

fn minimal_envelope() -> DocumentEnvelope {
    envelope(document(vec![paragraph(vec![json!({
        "type": "text",
        "text": "Minimal document"
    })])]))
}

fn rich_envelope() -> DocumentEnvelope {
    envelope(document(vec![
        json!({
            "type": "heading",
            "attrs": { "level": 2 },
            "content": [{ "type": "text", "text": "Heading text" }]
        }),
        paragraph(vec![
            json!({
                "type": "text",
                "text": "Café & <review>",
                "marks": [
                    { "type": "bold" },
                    { "type": "italic" },
                    { "type": "underline" }
                ]
            }),
            json!({ "type": "hardBreak" }),
            json!({ "type": "text", "text": "After break" }),
        ]),
    ]))
}

fn envelope(document: Value) -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": 1,
        "document_id": DOCUMENT_ID,
        "title": "DOCX export test",
        "document": document
    }))
    .unwrap()
}

fn document(content: Vec<Value>) -> Value {
    json!({ "type": "doc", "content": content })
}

fn paragraph(content: Vec<Value>) -> Value {
    json!({ "type": "paragraph", "content": content })
}

fn open_archive(artifact: &DocxArtifact) -> ZipArchive<Cursor<&[u8]>> {
    ZipArchive::new(Cursor::new(artifact.as_bytes())).unwrap()
}

fn read_part(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> String {
    let mut contents = String::new();
    archive
        .by_name(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    contents
}

fn assert_well_formed_xml(xml: &str) {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => panic!("invalid XML: {error}"),
        }
    }
}

fn assert_source_order(haystack: &str, values: &[&str]) {
    let mut previous = 0;
    for value in values {
        let current = haystack[previous..].find(value).unwrap() + previous;
        assert!(current >= previous);
        previous = current + value.len();
    }
}
