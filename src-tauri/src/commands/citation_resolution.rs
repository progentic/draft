use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use crate::{
    citations::resolution::{
        CitationResolutionError, ResolvedCitation, resolve_citation as resolve_from_store,
    },
    references::store::ReferenceStore,
};

/// Untrusted citation attrs submitted for Rust validation and local resolution.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveCitationRequest {
    attrs: Value,
}

/// Resolves one citation without returning reference metadata to the frontend.
#[tauri::command]
pub(crate) fn resolve_citation(
    store: State<'_, ReferenceStore>,
    request: ResolveCitationRequest,
) -> Result<ResolvedCitation, CitationResolutionError> {
    resolve_from_store(&store, request.attrs)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        citations::node::CitationNodeError,
        citations::resolution::CitationStoreError,
        references::{record::ReferenceRecord, test_support::TestReferenceStorePath},
    };

    const REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000001";
    const TYPED_COMMAND: for<'a> fn(
        State<'a, ReferenceStore>,
        ResolveCitationRequest,
    ) -> Result<ResolvedCitation, CitationResolutionError> = resolve_citation;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<ResolveCitationRequest>(json!({
            "attrs": valid_attrs()
        }))
        .unwrap();
        let unknown = serde_json::from_value::<ResolveCitationRequest>(json!({
            "attrs": valid_attrs(),
            "reference": {}
        }));

        assert_eq!(request.attrs, valid_attrs());
        assert!(unknown.is_err());
    }

    #[test]
    fn response_serialization_is_stable() {
        let target = TestReferenceStorePath::new("command-response");
        let store = ReferenceStore::open(target.path()).unwrap();
        store.create(&reference()).unwrap();
        let response = resolve_from_store(&store, valid_attrs()).unwrap();

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "schemaVersion": 1,
                "citekey": "smith2025",
                "renderStyle": "apa7",
                "displayMarker": "[@smith2025]"
            }),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            CitationResolutionError::InvalidCitation {
                cause: CitationNodeError::MissingSchemaVersion,
            },
            CitationResolutionError::ReferenceNotFound,
            CitationResolutionError::ReferenceStore {
                cause: CitationStoreError::ReadFailed,
            },
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                {
                    "code": "invalid_citation",
                    "cause": { "code": "missing_schema_version" }
                },
                { "code": "reference_not_found" },
                {
                    "code": "reference_store",
                    "cause": { "code": "read_failed" }
                }
            ]),
        );
    }

    fn reference() -> ReferenceRecord {
        ReferenceRecord::from_json_value(json!({
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
        }))
        .unwrap()
    }

    fn valid_attrs() -> Value {
        json!({
            "schema_version": 1,
            "citekey": "smith2025",
            "render_style": "apa7"
        })
    }
}
