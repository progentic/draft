use std::{fs, io::Cursor, io::Read, path::Path};

use serde_json::{Value, json};

use crate::{
    citations::resolution::resolve_citation,
    documents::{
        envelope::DocumentEnvelope,
        envelope::DocumentId,
        persistence::{
            OpenDocumentError, OpenDocumentOutcome, SaveDocumentError, SaveDocumentOutcome,
            open_document, save_document,
        },
        registry::{DocumentRegistry, DocumentRegistryError},
        test_support::TestDocumentPath,
    },
    exports::docx::{DocxExportError, export_docx},
    references::{
        record::ReferenceRecord, store::ReferenceStore, test_support::TestReferenceStorePath,
    },
};

const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000041";
const REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000041";

#[test]
fn critical_document_path_is_durable_citable_and_exportable() {
    let document_target = TestDocumentPath::new("critical-path");
    let registry = DocumentRegistry::new();

    create_document(&registry, document_target.path());
    let cited = save_cited_document(&registry);
    let reopened = close_and_reopen(&registry, document_target.path(), &cited);
    assert_single_live_handle(&registry, document_target.path(), &reopened);

    resolve_reopened_citation(&reopened);
    assert_citation_export_is_rejected(&reopened, document_target.path());
    save_and_export_supported_document(&registry, document_target.path());
    close_document(&registry, reopened.document_id());
}

fn create_document(registry: &DocumentRegistry, path: &Path) {
    let snapshot = document_snapshot("Created", plain_content("Initial draft"));
    let outcome = save_document(registry, snapshot, || Ok(Some(path.to_owned())))
        .expect("first save should create and register the document");

    assert_eq!(outcome, saved_outcome(path, true));
}

fn save_cited_document(registry: &DocumentRegistry) -> DocumentEnvelope {
    let snapshot = document_snapshot("Cited", cited_content());
    let outcome = save_document(registry, snapshot.clone(), no_path_selection)
        .expect("existing document should save to its retained path");
    assert!(matches!(
        outcome,
        SaveDocumentOutcome::Saved {
            was_save_as: false,
            ..
        }
    ));
    validated_envelope(snapshot)
}

fn close_and_reopen(
    registry: &DocumentRegistry,
    path: &Path,
    expected: &DocumentEnvelope,
) -> DocumentEnvelope {
    registry
        .close(expected.document_id())
        .expect("document should close");
    let reopened = opened_envelope(registry, path);
    assert_eq!(&reopened, expected);
    reopened
}

fn assert_single_live_handle(
    registry: &DocumentRegistry,
    path: &Path,
    reopened: &DocumentEnvelope,
) {
    assert_eq!(
        registry.source_path(reopened.document_id()),
        Ok(Some(path.to_owned()))
    );
    assert_eq!(
        open_document(registry, Some(path.to_owned())),
        Err(OpenDocumentError::Registry {
            cause: DocumentRegistryError::AlreadyOpen,
        })
    );
}

fn resolve_reopened_citation(document: &DocumentEnvelope) {
    let target = TestReferenceStorePath::new("critical-path");
    let store = ReferenceStore::open(target.path()).expect("reference store should open");
    store
        .create(&validated_reference())
        .expect("reference should persist");

    let resolved = resolve_citation(&store, citation_attrs(document))
        .expect("persisted citation should resolve");
    assert_eq!(
        serde_json::to_value(resolved).unwrap(),
        json!({
            "schemaVersion": 1,
            "citekey": "smith2025",
            "renderStyle": "apa7",
            "displayMarker": "[@smith2025]"
        })
    );
}

fn assert_citation_export_is_rejected(document: &DocumentEnvelope, source_path: &Path) {
    let export_path = source_path.with_extension("docx");
    assert!(matches!(
        export_docx(document, &export_path),
        Err(DocxExportError::UnsupportedCitation { .. })
    ));
    assert!(!export_path.exists());
}

fn save_and_export_supported_document(registry: &DocumentRegistry, source_path: &Path) {
    let snapshot = document_snapshot("Exportable", plain_content("Final draft"));
    let expected = validated_envelope(snapshot.clone());
    let outcome = save_document(registry, snapshot.clone(), no_path_selection)
        .expect("supported document should save");
    assert!(matches!(
        outcome,
        SaveDocumentOutcome::Saved {
            was_save_as: false,
            ..
        }
    ));
    let source_before_export = fs::read(source_path).expect("saved source should read");
    assert_eq!(
        serde_json::from_slice::<Value>(&source_before_export).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    let export_path = source_path.with_extension("docx");

    let outcome = export_docx(&expected, &export_path).expect("supported document should export");

    let artifact = fs::read(export_path).expect("DOCX artifact should read");
    assert_exported_document(&artifact, outcome.bytes_written());
    assert_eq!(fs::read(source_path).unwrap(), source_before_export);
}

fn assert_exported_document(artifact: &[u8], expected_bytes: usize) {
    let mut archive = zip::ZipArchive::new(Cursor::new(artifact)).expect("DOCX should reopen");
    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")
        .expect("DOCX document part should exist")
        .read_to_string(&mut document_xml)
        .expect("DOCX document part should be readable");

    assert_eq!(expected_bytes, artifact.len());
    assert!(document_xml.contains("Final draft"));
}

fn close_document(registry: &DocumentRegistry, document_id: DocumentId) {
    registry.close(document_id).expect("document should close");
    assert_eq!(
        registry.close(document_id),
        Err(DocumentRegistryError::NotOpen)
    );
}

fn opened_envelope(registry: &DocumentRegistry, path: &Path) -> DocumentEnvelope {
    match open_document(registry, Some(path.to_owned())).expect("document should reopen") {
        OpenDocumentOutcome::OpenedDraft { envelope } => envelope,
        OpenDocumentOutcome::ImportedText { .. } => panic!("DRAFT source must reopen natively"),
        OpenDocumentOutcome::Cancelled => panic!("explicit path must not cancel"),
    }
}

fn saved_outcome(path: &Path, was_save_as: bool) -> SaveDocumentOutcome {
    SaveDocumentOutcome::Saved {
        document_id: validated_envelope(document_snapshot("Saved", plain_content("Saved")))
            .document_id(),
        display_name: path.file_name().unwrap().to_str().unwrap().to_owned(),
        was_save_as,
    }
}

fn no_path_selection() -> Result<Option<std::path::PathBuf>, SaveDocumentError> {
    panic!("existing document must not request another path")
}

fn citation_attrs(document: &DocumentEnvelope) -> Value {
    serde_json::to_value(document).unwrap()["document"]["content"][0]["content"][1]["attrs"].clone()
}

fn validated_envelope(value: Value) -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(value).expect("fixture envelope should validate")
}

fn document_snapshot(title: &str, content: Vec<Value>) -> Value {
    json!({
        "schema_version": 1,
        "document_id": DOCUMENT_ID,
        "title": title,
        "document": { "type": "doc", "content": content }
    })
}

fn plain_content(text: &str) -> Vec<Value> {
    vec![paragraph(vec![json!({ "type": "text", "text": text })])]
}

fn cited_content() -> Vec<Value> {
    vec![paragraph(vec![
        json!({ "type": "text", "text": "Supported claim " }),
        json!({
            "type": "citation",
            "attrs": {
                "schema_version": 1,
                "citekey": "smith2025",
                "render_style": "apa7"
            }
        }),
    ])]
}

fn paragraph(content: Vec<Value>) -> Value {
    json!({ "type": "paragraph", "content": content })
}

fn validated_reference() -> ReferenceRecord {
    ReferenceRecord::from_json_value(reference_fixture())
        .expect("reference fixture should validate")
}

fn reference_fixture() -> Value {
    json!({
        "schema_version": 1,
        "reference_id": REFERENCE_ID,
        "citekey": "smith2025",
        "kind": "article",
        "title": "Critical path evidence",
        "contributors": [{
            "role": "author",
            "name": { "type": "person", "given": "Ada", "family": "Smith" }
        }],
        "issued": { "year": 2025, "month": null, "day": null },
        "container_title": "Journal of Examples",
        "publisher": null,
        "volume": "12",
        "issue": "3",
        "pages": "1-12",
        "resolution_state": "resolved",
        "identifiers": { "doi": null, "isbn": [], "url": null },
        "provenance": {
            "source": "manual",
            "source_record_id": null,
            "manual_overrides": []
        }
    })
}
