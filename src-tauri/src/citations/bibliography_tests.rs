use serde_json::json;

use super::*;

const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
const FIRST_REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000011";
const SECOND_REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000012";
const THIRD_REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000013";

#[test]
fn matching_citations_and_records_are_consistent() {
    let document = document(&["smith2025", "jones2024"]);
    let bibliography = vec![
        reference(FIRST_REFERENCE_ID, "jones2024"),
        reference(SECOND_REFERENCE_ID, "smith2025"),
    ];

    assert!(report(&document, &bibliography).is_consistent());
}

#[test]
fn missing_citekeys_are_reported() {
    let document = document(&["present2025", "missing2025"]);
    let bibliography = vec![reference(FIRST_REFERENCE_ID, "present2025")];

    assert_eq!(
        report(&document, &bibliography).missing_citekeys(),
        ["missing2025"]
    );
}

#[test]
fn orphaned_citekeys_are_reported() {
    let document = document(&["present2025"]);
    let bibliography = vec![
        reference(FIRST_REFERENCE_ID, "orphaned2025"),
        reference(SECOND_REFERENCE_ID, "present2025"),
    ];

    assert_eq!(
        report(&document, &bibliography).orphaned_citekeys(),
        ["orphaned2025"]
    );
}

#[test]
fn duplicate_bibliography_citekeys_are_reported() {
    let document = document(&["smith2025"]);
    let bibliography = vec![
        reference(FIRST_REFERENCE_ID, "smith2025"),
        reference(SECOND_REFERENCE_ID, "smith2025"),
    ];

    assert_eq!(
        report(&document, &bibliography).duplicate_citekeys(),
        ["smith2025"]
    );
}

#[test]
fn repeated_in_text_citations_are_not_duplicates() {
    let document = document(&["smith2025", "smith2025"]);
    let bibliography = vec![reference(FIRST_REFERENCE_ID, "smith2025")];

    assert!(report(&document, &bibliography).is_consistent());
}

#[test]
fn orphaned_duplicate_categories_are_independent() {
    let document = document(&[]);
    let bibliography = vec![
        reference(FIRST_REFERENCE_ID, "unused2025"),
        reference(SECOND_REFERENCE_ID, "unused2025"),
    ];
    let report = report(&document, &bibliography);

    assert_eq!(report.orphaned_citekeys(), ["unused2025"]);
    assert_eq!(report.duplicate_citekeys(), ["unused2025"]);
}

#[test]
fn consistency_results_are_sorted_and_case_sensitive() {
    let document = document(&["zeta2025", "Alpha2025"]);
    let bibliography = vec![
        reference(FIRST_REFERENCE_ID, "beta2025"),
        reference(SECOND_REFERENCE_ID, "alpha2025"),
        reference(THIRD_REFERENCE_ID, "alpha2025"),
    ];
    let report = report(&document, &bibliography);

    assert_eq!(report.missing_citekeys(), ["Alpha2025", "zeta2025"]);
    assert_eq!(report.orphaned_citekeys(), ["alpha2025", "beta2025"]);
    assert_eq!(report.duplicate_citekeys(), ["alpha2025"]);
}

#[test]
fn empty_document_and_bibliography_are_consistent() {
    assert!(report(&document(&[]), &[]).is_consistent());
}

fn report(
    document: &DocumentEnvelope,
    bibliography: &[ReferenceRecord],
) -> BibliographyConsistencyReport {
    check_bibliography_consistency(document, bibliography).unwrap()
}

fn document(citekeys: &[&str]) -> DocumentEnvelope {
    let content = citekeys
        .iter()
        .map(|citekey| {
            json!({
                "type": "citation",
                "attrs": {
                    "schema_version": 1,
                    "citekey": citekey,
                    "render_style": "apa7"
                }
            })
        })
        .collect::<Vec<_>>();
    DocumentEnvelope::from_json_value(json!({
        "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": DOCUMENT_ID,
        "title": "Consistency test",
        "document": { "type": "doc", "content": content }
    }))
    .unwrap()
}

fn reference(reference_id: &str, citekey: &str) -> ReferenceRecord {
    ReferenceRecord::from_json_value(json!({
        "schema_version": 1,
        "reference_id": reference_id,
        "citekey": citekey,
        "kind": "article",
        "title": "Reference",
        "contributors": [],
        "issued": null,
        "container_title": null,
        "publisher": null,
        "volume": null,
        "issue": null,
        "pages": null,
        "resolution_state": "unresolved",
        "identifiers": { "doi": null, "isbn": [], "url": null },
        "provenance": {
            "source": "manual",
            "source_record_id": null,
            "manual_overrides": []
        }
    }))
    .unwrap()
}
