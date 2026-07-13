use serde::{Deserialize, Serialize};
use tauri::State;

use crate::references::{
    record::ReferenceRecord,
    store::{ReferenceStore, ReferenceStoreError},
};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct AddReferenceRequest {
    citekey: String,
    title: String,
    author: String,
    year: u16,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListReferencesRequest {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReferenceSummary {
    citekey: String,
    title: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ReferenceLibraryError {
    InvalidReference,
    DuplicateCitekey,
    StoreUnavailable,
    ReadFailed,
    WriteFailed,
}

#[tauri::command]
pub(crate) fn add_reference(
    store: State<'_, ReferenceStore>,
    request: AddReferenceRequest,
) -> Result<ReferenceSummary, ReferenceLibraryError> {
    add_to_store(&store, request)
}

#[tauri::command]
pub(crate) fn list_references(
    store: State<'_, ReferenceStore>,
    request: ListReferencesRequest,
) -> Result<Vec<ReferenceSummary>, ReferenceLibraryError> {
    let ListReferencesRequest {} = request;
    summaries_from_store(&store)
}

fn add_to_store(
    store: &ReferenceStore,
    request: AddReferenceRequest,
) -> Result<ReferenceSummary, ReferenceLibraryError> {
    let record = manual_record(request)?;
    store.create(&record).map_err(add_store_error)?;
    Ok(summary(&record))
}

fn summaries_from_store(
    store: &ReferenceStore,
) -> Result<Vec<ReferenceSummary>, ReferenceLibraryError> {
    store
        .list()
        .map_err(list_store_error)
        .map(|records| records.iter().map(summary).collect())
}

fn manual_record(request: AddReferenceRequest) -> Result<ReferenceRecord, ReferenceLibraryError> {
    ReferenceRecord::manual(request.citekey, request.title, request.author, request.year)
        .map_err(|_| ReferenceLibraryError::InvalidReference)
}

fn summary(record: &ReferenceRecord) -> ReferenceSummary {
    ReferenceSummary {
        citekey: record.citekey().to_owned(),
        title: record.title().to_owned(),
    }
}

fn add_store_error(cause: ReferenceStoreError) -> ReferenceLibraryError {
    match cause {
        ReferenceStoreError::DuplicateCitekey => ReferenceLibraryError::DuplicateCitekey,
        ReferenceStoreError::SerializationFailed | ReferenceStoreError::WriteFailed => {
            ReferenceLibraryError::WriteFailed
        }
        _ => ReferenceLibraryError::StoreUnavailable,
    }
}

fn list_store_error(cause: ReferenceStoreError) -> ReferenceLibraryError {
    match cause {
        ReferenceStoreError::ReadFailed
        | ReferenceStoreError::MalformedStoredJson
        | ReferenceStoreError::InvalidStoredRecord { .. }
        | ReferenceStoreError::StoredSchemaMismatch
        | ReferenceStoreError::StoredIdentityMismatch => ReferenceLibraryError::ReadFailed,
        _ => ReferenceLibraryError::StoreUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::references::test_support::TestReferenceStorePath;

    mod add_command {
        use super::*;

        const TYPED_COMMAND: for<'a> fn(
            State<'a, ReferenceStore>,
            AddReferenceRequest,
        )
            -> Result<ReferenceSummary, ReferenceLibraryError> = add_reference;

        #[test]
        fn command_signature_is_typed() {
            let _ = TYPED_COMMAND;
        }

        #[test]
        fn request_deserialization_is_stable() {
            let request = serde_json::from_value::<AddReferenceRequest>(json!({
                "citekey": "smith2025", "title": "Source", "author": "Ada Smith",
                "year": 2025
            }));
            let unknown = serde_json::from_value::<AddReferenceRequest>(json!({
                "citekey": "smith2025", "title": "Source", "author": "Ada Smith",
                "year": 2025, "url": "https://example.com"
            }));
            assert!(request.is_ok());
            assert!(unknown.is_err());
        }

        #[test]
        fn response_serialization_is_stable() {
            let response = ReferenceSummary {
                citekey: "smith2025".to_owned(),
                title: "Source".to_owned(),
            };
            assert_eq!(
                serde_json::to_value(response).unwrap(),
                json!({ "citekey": "smith2025", "title": "Source" })
            );
        }

        #[test]
        fn error_serialization_is_stable() {
            assert_eq!(
                serde_json::to_value(ReferenceLibraryError::DuplicateCitekey).unwrap(),
                json!({ "code": "duplicate_citekey" })
            );
        }
    }

    mod list_command {
        use super::*;

        const TYPED_COMMAND: for<'a> fn(
            State<'a, ReferenceStore>,
            ListReferencesRequest,
        )
            -> Result<Vec<ReferenceSummary>, ReferenceLibraryError> = list_references;

        #[test]
        fn command_signature_is_typed() {
            let _ = TYPED_COMMAND;
        }

        #[test]
        fn request_deserialization_is_stable() {
            assert!(serde_json::from_value::<ListReferencesRequest>(json!({})).is_ok());
            assert!(
                serde_json::from_value::<ListReferencesRequest>(json!({ "limit": 1 })).is_err()
            );
        }

        #[test]
        fn response_serialization_is_stable() {
            let response = vec![ReferenceSummary {
                citekey: "smith2025".to_owned(),
                title: "Source".to_owned(),
            }];
            assert_eq!(
                serde_json::to_value(response).unwrap(),
                json!([{ "citekey": "smith2025", "title": "Source" }])
            );
        }

        #[test]
        fn error_serialization_is_stable() {
            assert_eq!(
                serde_json::to_value(ReferenceLibraryError::ReadFailed).unwrap(),
                json!({ "code": "read_failed" })
            );
        }
    }

    #[test]
    fn add_and_list_use_the_existing_store() {
        let path = TestReferenceStorePath::new("visible-reference-library");
        let store = ReferenceStore::open(path.path()).unwrap();

        let added = add_to_store(&store, request()).unwrap();
        let listed = summaries_from_store(&store).unwrap();

        assert_eq!(added, listed[0]);
        assert_eq!(
            serde_json::to_value(listed).unwrap(),
            json!([{
                "citekey": "smith2025", "title": "A useful source"
            }])
        );
    }

    fn request() -> AddReferenceRequest {
        AddReferenceRequest {
            citekey: "smith2025".to_owned(),
            title: "A useful source".to_owned(),
            author: "Ada Smith".to_owned(),
            year: 2025,
        }
    }
}
