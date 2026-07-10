use std::{fs, sync::Arc, thread};

use rusqlite::Connection;
use serde_json::{Value, json};

use super::*;
use crate::references::test_support::TestReferenceStorePath;

const FIRST_REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000001";
const SECOND_REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000002";

#[test]
fn reference_store_path_uses_app_data_directory() {
    let app_data = Path::new("app-data");
    assert_eq!(
        reference_store_path(app_data),
        app_data.join(REFERENCE_STORE_FILENAME)
    );
}

#[test]
fn new_store_initializes_schema_and_table() {
    let target = TestReferenceStorePath::new("initialize");
    let store = ReferenceStore::open(target.path()).expect("store should open");

    assert_eq!(read_schema(&store), REFERENCE_STORE_SCHEMA_VERSION);
    assert!(reference_table_exists(&store));
}

#[test]
fn schema_initialization_is_idempotent() {
    let target = TestReferenceStorePath::new("idempotent");
    ReferenceStore::open(target.path()).expect("first open should initialize");

    let reopened = ReferenceStore::open(target.path()).expect("second open should reuse schema");

    assert_eq!(read_schema(&reopened), REFERENCE_STORE_SCHEMA_VERSION);
}

#[test]
fn unsupported_store_schema_fails_explicitly() {
    let target = TestReferenceStorePath::new("unsupported-schema");
    let connection = Connection::open(target.path()).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::UnsupportedStoreSchema { found: 2 })
    );
}

#[test]
fn claimed_current_schema_requires_expected_table() {
    let target = TestReferenceStorePath::new("invalid-current-schema");
    let connection = Connection::open(target.path()).unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    drop(connection);

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::InvalidStoreSchema)
    );
}

#[test]
fn claimed_current_schema_requires_expected_constraints() {
    let target = TestReferenceStorePath::new("invalid-current-constraints");
    let connection = Connection::open(target.path()).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE reference_records (
                reference_id TEXT,
                citekey TEXT,
                schema_version INTEGER,
                payload_json TEXT
            );
            PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(connection);

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::InvalidStoreSchema)
    );
}

#[test]
fn store_creates_missing_parent_directory() {
    let target = TestReferenceStorePath::new("parent");
    let nested = target
        .path()
        .parent()
        .unwrap()
        .join("nested")
        .join("references.sqlite3");

    ReferenceStore::open(&nested).expect("store should create parent");

    assert!(nested.is_file());
}

#[test]
fn unavailable_parent_returns_storage_error() {
    let target = TestReferenceStorePath::new("unavailable-parent");
    fs::write(target.path(), b"not a directory").unwrap();
    let nested = target.path().join("references.sqlite3");

    assert_eq!(
        ReferenceStore::open(&nested).err(),
        Some(ReferenceStoreError::StorageLocationUnavailable)
    );
}

#[test]
fn directory_database_path_returns_open_error() {
    let target = TestReferenceStorePath::new("directory-path");
    fs::create_dir(target.path()).unwrap();

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::OpenFailed)
    );
}

#[test]
fn malformed_database_returns_schema_read_error() {
    let target = TestReferenceStorePath::new("malformed-database");
    fs::write(target.path(), b"not a sqlite database").unwrap();

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::SchemaReadFailed)
    );
}

#[test]
fn conflicting_zero_version_schema_returns_migration_error() {
    let target = TestReferenceStorePath::new("migration-failure");
    let connection = Connection::open(target.path()).unwrap();
    connection
        .execute("CREATE TABLE reference_records (wrong TEXT)", [])
        .unwrap();
    drop(connection);

    assert_eq!(
        ReferenceStore::open(target.path()).err(),
        Some(ReferenceStoreError::SchemaMigrationFailed)
    );
}

#[test]
fn create_read_and_reopen_preserve_record() {
    let target = TestReferenceStorePath::new("reopen");
    let expected = reference(FIRST_REFERENCE_ID, "smith2025", "Original");
    let reference_id = expected.reference_id();
    let store = ReferenceStore::open(target.path()).unwrap();
    store.create(&expected).unwrap();
    drop(store);

    let reopened = ReferenceStore::open(target.path()).unwrap();
    let actual = reopened.get(reference_id).unwrap().unwrap();

    assert_eq!(reference_value(&actual), reference_value(&expected));
}

#[test]
fn duplicate_identity_and_citekey_fail_explicitly() {
    let target = TestReferenceStorePath::new("duplicates");
    let store = ReferenceStore::open(target.path()).unwrap();
    let first = reference(FIRST_REFERENCE_ID, "smith2025", "First");
    store.create(&first).unwrap();

    assert_eq!(
        store.create(&reference(FIRST_REFERENCE_ID, "other2025", "Duplicate ID")),
        Err(ReferenceStoreError::DuplicateReferenceId)
    );
    assert_eq!(
        store.create(&reference(
            SECOND_REFERENCE_ID,
            "smith2025",
            "Duplicate key"
        )),
        Err(ReferenceStoreError::DuplicateCitekey)
    );
}

#[test]
fn citekey_uniqueness_is_case_sensitive() {
    let target = TestReferenceStorePath::new("citekey-case");
    let store = ReferenceStore::open(target.path()).unwrap();

    store
        .create(&reference(FIRST_REFERENCE_ID, "smith2025", "Lower"))
        .unwrap();
    store
        .create(&reference(SECOND_REFERENCE_ID, "Smith2025", "Upper"))
        .unwrap();

    assert!(store.get_by_citekey("smith2025").unwrap().is_some());
    assert!(store.get_by_citekey("Smith2025").unwrap().is_some());
}

#[test]
fn update_replaces_payload_and_citekey() {
    let target = TestReferenceStorePath::new("update");
    let store = ReferenceStore::open(target.path()).unwrap();
    let original = reference(FIRST_REFERENCE_ID, "smith2025", "Original");
    let updated = reference(FIRST_REFERENCE_ID, "smith2026", "Updated");
    store.create(&original).unwrap();

    store.update(&updated).unwrap();

    assert!(store.get_by_citekey("smith2025").unwrap().is_none());
    assert_eq!(
        reference_value(&store.get(updated.reference_id()).unwrap().unwrap()),
        reference_value(&updated)
    );
}

#[test]
fn conflicting_update_preserves_both_records() {
    let target = TestReferenceStorePath::new("update-conflict");
    let store = ReferenceStore::open(target.path()).unwrap();
    let first = reference(FIRST_REFERENCE_ID, "first2025", "First");
    let second = reference(SECOND_REFERENCE_ID, "second2025", "Second");
    store.create(&first).unwrap();
    store.create(&second).unwrap();
    let conflicting = reference(SECOND_REFERENCE_ID, "first2025", "Changed");

    assert_eq!(
        store.update(&conflicting),
        Err(ReferenceStoreError::DuplicateCitekey)
    );
    assert_eq!(stored_title(&store, first.reference_id()), "First");
    assert_eq!(stored_title(&store, second.reference_id()), "Second");
}

#[test]
fn delete_returns_record_and_removes_it() {
    let target = TestReferenceStorePath::new("delete");
    let store = ReferenceStore::open(target.path()).unwrap();
    let expected = reference(FIRST_REFERENCE_ID, "smith2025", "Delete me");
    let reference_id = expected.reference_id();
    store.create(&expected).unwrap();

    let deleted = store.delete(reference_id).unwrap();

    assert_eq!(reference_value(&deleted), reference_value(&expected));
    assert!(store.get(reference_id).unwrap().is_none());
}

#[test]
fn list_is_deterministic_by_citekey() {
    let target = TestReferenceStorePath::new("list");
    let store = ReferenceStore::open(target.path()).unwrap();
    store
        .create(&reference(FIRST_REFERENCE_ID, "zeta2025", "Zeta"))
        .unwrap();
    store
        .create(&reference(SECOND_REFERENCE_ID, "alpha2025", "Alpha"))
        .unwrap();

    let citekeys = store
        .list()
        .unwrap()
        .into_iter()
        .map(|record| record.citekey().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(citekeys, ["alpha2025", "zeta2025"]);
}

#[test]
fn missing_update_and_delete_fail_explicitly() {
    let target = TestReferenceStorePath::new("missing");
    let store = ReferenceStore::open(target.path()).unwrap();
    let missing = reference(FIRST_REFERENCE_ID, "missing2025", "Missing");

    assert_eq!(
        store.update(&missing),
        Err(ReferenceStoreError::ReferenceNotFound)
    );
    assert_eq!(
        store.delete(missing.reference_id()),
        Err(ReferenceStoreError::ReferenceNotFound)
    );
}

#[test]
fn malformed_stored_json_fails_without_deleting_row() {
    let target = TestReferenceStorePath::new("malformed-json");
    let store = ReferenceStore::open(target.path()).unwrap();
    insert_raw_record(&store, FIRST_REFERENCE_ID, "broken2025", "{", 1);
    let reference_id = reference(FIRST_REFERENCE_ID, "broken2025", "Broken").reference_id();

    assert_eq!(
        store.get(reference_id),
        Err(ReferenceStoreError::MalformedStoredJson)
    );
    assert_eq!(
        store.delete(reference_id),
        Err(ReferenceStoreError::MalformedStoredJson)
    );
    assert_eq!(raw_record_count(&store), 1);
}

#[test]
fn missing_live_table_returns_read_error() {
    let target = TestReferenceStorePath::new("missing-live-table");
    let store = ReferenceStore::open(target.path()).unwrap();
    store
        .connection
        .lock()
        .unwrap()
        .execute("DROP TABLE reference_records", [])
        .unwrap();

    assert_eq!(store.list(), Err(ReferenceStoreError::ReadFailed));
}

#[test]
fn invalid_stored_record_returns_typed_cause() {
    let target = TestReferenceStorePath::new("invalid-record");
    let store = ReferenceStore::open(target.path()).unwrap();
    let mut invalid = reference_fixture(FIRST_REFERENCE_ID, "invalid2025", "Invalid");
    invalid["kind"] = json!("unknown");
    insert_raw_record(
        &store,
        FIRST_REFERENCE_ID,
        "invalid2025",
        &invalid.to_string(),
        1,
    );

    assert_eq!(
        store.list(),
        Err(ReferenceStoreError::InvalidStoredRecord {
            cause: ReferenceRecordError::UnsupportedReferenceKind
        })
    );
}

#[test]
fn mismatched_stored_indexes_fail_closed() {
    let target = TestReferenceStorePath::new("mismatched-index");
    let store = ReferenceStore::open(target.path()).unwrap();
    let payload = reference_fixture(SECOND_REFERENCE_ID, "payload2025", "Payload");
    insert_raw_record(
        &store,
        FIRST_REFERENCE_ID,
        "payload2025",
        &payload.to_string(),
        1,
    );

    assert_eq!(
        store.list(),
        Err(ReferenceStoreError::StoredIdentityMismatch)
    );
}

#[test]
fn mismatched_stored_schema_fails_closed() {
    let target = TestReferenceStorePath::new("mismatched-schema");
    let store = ReferenceStore::open(target.path()).unwrap();
    let payload = reference_fixture(FIRST_REFERENCE_ID, "schema2025", "Schema");
    let connection = store.connection.lock().unwrap();
    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    connection
        .execute(
            "INSERT INTO reference_records
             (reference_id, citekey, schema_version, payload_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![FIRST_REFERENCE_ID, "schema2025", 2, payload.to_string()],
        )
        .unwrap();
    drop(connection);

    assert_eq!(store.list(), Err(ReferenceStoreError::StoredSchemaMismatch));
}

#[test]
fn concurrent_create_allows_one_record() {
    let target = TestReferenceStorePath::new("concurrent-create");
    let store = Arc::new(ReferenceStore::open(target.path()).unwrap());
    let outcomes = thread::scope(|scope| {
        let first_store = Arc::clone(&store);
        let first = scope
            .spawn(move || first_store.create(&reference(FIRST_REFERENCE_ID, "same2025", "First")));
        let second_store = Arc::clone(&store);
        let second = scope.spawn(move || {
            second_store.create(&reference(FIRST_REFERENCE_ID, "same2025", "Second"))
        });
        [first.join().unwrap(), second.join().unwrap()]
    });

    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| **outcome == Err(ReferenceStoreError::DuplicateReferenceId))
            .count(),
        1
    );
}

#[test]
fn poisoned_store_returns_unavailable() {
    let target = TestReferenceStorePath::new("poisoned");
    let store = Arc::new(ReferenceStore::open(target.path()).unwrap());
    let poisoned = Arc::clone(&store);
    let _ = thread::spawn(move || {
        let _connection = poisoned.connection.lock().unwrap();
        panic!("poison store for test");
    })
    .join();

    assert_eq!(store.list(), Err(ReferenceStoreError::StoreUnavailable));
}

#[test]
fn store_failure_shape_is_stable() {
    let failures = [
        ReferenceStoreError::StorageLocationUnavailable,
        ReferenceStoreError::OpenFailed,
        ReferenceStoreError::SchemaReadFailed,
        ReferenceStoreError::SchemaMigrationFailed,
        ReferenceStoreError::UnsupportedStoreSchema { found: 2 },
        ReferenceStoreError::InvalidStoreSchema,
        ReferenceStoreError::StoreUnavailable,
        ReferenceStoreError::SerializationFailed,
        ReferenceStoreError::WriteFailed,
        ReferenceStoreError::ReadFailed,
        ReferenceStoreError::DuplicateReferenceId,
        ReferenceStoreError::DuplicateCitekey,
        ReferenceStoreError::ReferenceNotFound,
        ReferenceStoreError::MalformedStoredJson,
        ReferenceStoreError::InvalidStoredRecord {
            cause: ReferenceRecordError::InvalidCitekey,
        },
        ReferenceStoreError::StoredSchemaMismatch,
        ReferenceStoreError::StoredIdentityMismatch,
    ];
    let values = serde_json::to_value(failures).unwrap();

    let codes = [
        "storage_location_unavailable",
        "open_failed",
        "schema_read_failed",
        "schema_migration_failed",
        "unsupported_store_schema",
        "invalid_store_schema",
        "store_unavailable",
        "serialization_failed",
        "write_failed",
        "read_failed",
        "duplicate_reference_id",
        "duplicate_citekey",
        "reference_not_found",
        "malformed_stored_json",
        "invalid_stored_record",
        "stored_schema_mismatch",
        "stored_identity_mismatch",
    ];
    for (value, code) in values.as_array().unwrap().iter().zip(codes) {
        assert_eq!(value["code"], code);
    }
    assert_eq!(
        values[4],
        json!({ "code": "unsupported_store_schema", "found": 2 })
    );
    assert_eq!(
        values[14],
        json!({
            "code": "invalid_stored_record",
            "cause": { "code": "invalid_citekey" }
        })
    );
}

fn read_schema(store: &ReferenceStore) -> u64 {
    let connection = store.connection.lock().unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    version as u64
}

fn reference_table_exists(store: &ReferenceStore) -> bool {
    let connection = store.connection.lock().unwrap();
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = 'reference_records'
            )",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

fn insert_raw_record(
    store: &ReferenceStore,
    reference_id: &str,
    citekey: &str,
    payload: &str,
    schema_version: i64,
) {
    let connection = store.connection.lock().unwrap();
    connection
        .execute(
            "INSERT INTO reference_records
             (reference_id, citekey, schema_version, payload_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![reference_id, citekey, schema_version, payload],
        )
        .unwrap();
}

fn raw_record_count(store: &ReferenceStore) -> u64 {
    let connection = store.connection.lock().unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM reference_records", [], |row| {
            row.get(0)
        })
        .unwrap();
    count as u64
}

fn stored_title(store: &ReferenceStore, reference_id: ReferenceId) -> String {
    let record = store.get(reference_id).unwrap().unwrap();
    reference_value(&record)["title"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn reference(reference_id: &str, citekey: &str, title: &str) -> ReferenceRecord {
    ReferenceRecord::from_json_value(reference_fixture(reference_id, citekey, title)).unwrap()
}

fn reference_value(record: &ReferenceRecord) -> Value {
    serde_json::to_value(record).unwrap()
}

fn reference_fixture(reference_id: &str, citekey: &str, title: &str) -> Value {
    json!({
        "schema_version": 1,
        "reference_id": reference_id,
        "citekey": citekey,
        "kind": "article",
        "title": title,
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
