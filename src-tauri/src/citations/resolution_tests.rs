use serde_json::{Value, json};

use super::*;
use crate::references::{
    record::ReferenceRecord, store::ReferenceStore, test_support::TestReferenceStorePath,
};

const REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000001";

#[test]
fn known_citation_resolves_to_disposable_marker() {
    let (_target, store) = store_with_reference();

    assert_eq!(
        serde_json::to_value(resolve_citation(&store, valid_attrs()).unwrap()).unwrap(),
        json!({
            "schemaVersion": 1,
            "citekey": "smith2025",
            "renderStyle": "apa7",
            "displayMarker": "[@smith2025]"
        }),
    );
}

#[test]
fn invalid_citation_fails_before_store_lookup() {
    let target = TestReferenceStorePath::new("invalid-citation");
    let store = ReferenceStore::open(target.path()).unwrap();
    let mut attrs = valid_attrs();
    attrs["schema_version"] = json!(2);

    assert_eq!(
        resolve_citation(&store, attrs),
        Err(CitationResolutionError::InvalidCitation {
            cause: CitationNodeError::UnsupportedSchemaVersion { found: 2 },
        }),
    );
}

#[test]
fn missing_reference_fails_explicitly() {
    let target = TestReferenceStorePath::new("missing-reference");
    let store = ReferenceStore::open(target.path()).unwrap();

    assert_eq!(
        resolve_citation(&store, valid_attrs()),
        Err(CitationResolutionError::ReferenceNotFound),
    );
}

#[test]
fn corrupt_reference_store_failure_is_preserved() {
    let (_target, store) = store_with_reference();
    store.replace_payload_for_test("smith2025", "not-json");

    assert_eq!(
        resolve_citation(&store, valid_attrs()),
        Err(CitationResolutionError::ReferenceStore {
            cause: CitationStoreError::CorruptReference,
        }),
    );
}

#[test]
fn citation_resolution_failure_shape_is_stable() {
    let errors = [
        CitationResolutionError::InvalidCitation {
            cause: CitationNodeError::MissingCitekey,
        },
        CitationResolutionError::ReferenceNotFound,
        CitationResolutionError::ReferenceStore {
            cause: CitationStoreError::Unavailable,
        },
    ];

    assert_eq!(
        serde_json::to_value(errors).unwrap(),
        json!([
            {
                "code": "invalid_citation",
                "cause": { "code": "missing_citekey" }
            },
            { "code": "reference_not_found" },
            {
                "code": "reference_store",
                "cause": { "code": "unavailable" }
            }
        ]),
    );
}

fn store_with_reference() -> (TestReferenceStorePath, ReferenceStore) {
    let target = TestReferenceStorePath::new("citation-resolution");
    let store = ReferenceStore::open(target.path()).unwrap();
    store.create(&reference()).unwrap();
    (target, store)
}

fn reference() -> ReferenceRecord {
    ReferenceRecord::from_json_value(reference_fixture()).unwrap()
}

fn valid_attrs() -> Value {
    json!({
        "schema_version": 1,
        "citekey": "smith2025",
        "render_style": "apa7"
    })
}

fn reference_fixture() -> Value {
    json!({
        "schema_version": 1,
        "reference_id": REFERENCE_ID,
        "citekey": "smith2025",
        "kind": "article",
        "title": "A resolved reference",
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
