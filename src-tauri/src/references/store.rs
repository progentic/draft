use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use rusqlite::{Connection, OptionalExtension, Row, Transaction, TransactionBehavior, params};
use serde::Serialize;
use serde_json::Value;

use crate::references::record::{
    REFERENCE_RECORD_SCHEMA_VERSION, ReferenceId, ReferenceRecord, ReferenceRecordError,
};

const CREATE_SCHEMA_V1: &str = r#"
    CREATE TABLE reference_records (
        reference_id TEXT PRIMARY KEY NOT NULL,
        citekey TEXT NOT NULL UNIQUE COLLATE BINARY,
        schema_version INTEGER NOT NULL CHECK (schema_version = 1),
        payload_json TEXT NOT NULL
    ) STRICT;
    PRAGMA user_version = 1;
"#;

/// Current SQLite schema version for the local reference store.
pub const REFERENCE_STORE_SCHEMA_VERSION: u64 = 1;

/// Stable database filename joined to the Rust-resolved application data directory.
pub const REFERENCE_STORE_FILENAME: &str = "references.sqlite3";

/// Maximum time a store connection waits for another SQLite writer.
const REFERENCE_STORE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Rust-owned transactional persistence for validated reference records.
pub struct ReferenceStore {
    connection: Mutex<Connection>,
}

/// Bounded failures produced by reference-store operations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ReferenceStoreError {
    StorageLocationUnavailable,
    OpenFailed,
    SchemaReadFailed,
    SchemaMigrationFailed,
    UnsupportedStoreSchema { found: u64 },
    InvalidStoreSchema,
    StoreUnavailable,
    SerializationFailed,
    WriteFailed,
    ReadFailed,
    DuplicateReferenceId,
    DuplicateCitekey,
    ReferenceNotFound,
    MalformedStoredJson,
    InvalidStoredRecord { cause: ReferenceRecordError },
    StoredSchemaMismatch,
    StoredIdentityMismatch,
}

struct StoredReference {
    reference_id: String,
    citekey: String,
    schema_version: i64,
    payload_json: String,
}

/// Returns the production reference-store path for a Rust-resolved app-data directory.
pub fn reference_store_path(app_data_directory: &Path) -> PathBuf {
    app_data_directory.join(REFERENCE_STORE_FILENAME)
}

impl ReferenceStore {
    /// Opens or initializes a local SQLite reference store at a Rust-selected path.
    pub fn open(path: &Path) -> Result<Self, ReferenceStoreError> {
        ensure_parent_directory(path)?;
        let mut connection = open_connection(path)?;
        configure_connection(&connection)?;
        migrate_store(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Creates one validated record and rejects duplicate identity or citekey values.
    pub fn create(&self, record: &ReferenceRecord) -> Result<(), ReferenceStoreError> {
        let payload = serialize_record(record)?;
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        reject_create_conflict(&transaction, record)?;
        insert_record(&transaction, record, &payload)?;
        commit_write(transaction)
    }

    /// Returns one validated record by stable identity.
    pub fn get(
        &self,
        reference_id: ReferenceId,
    ) -> Result<Option<ReferenceRecord>, ReferenceStoreError> {
        let connection = self.lock_connection()?;
        let stored = load_by_id(&connection, reference_id)?;
        stored.map(decode_stored_reference).transpose()
    }

    /// Returns one validated record by its case-sensitive citekey.
    pub fn get_by_citekey(
        &self,
        citekey: &str,
    ) -> Result<Option<ReferenceRecord>, ReferenceStoreError> {
        let connection = self.lock_connection()?;
        let stored = load_by_citekey(&connection, citekey)?;
        stored.map(decode_stored_reference).transpose()
    }

    /// Returns all validated records in deterministic citekey and identity order.
    pub fn list(&self) -> Result<Vec<ReferenceRecord>, ReferenceStoreError> {
        let connection = self.lock_connection()?;
        load_all(&connection)?
            .into_iter()
            .map(decode_stored_reference)
            .collect()
    }

    /// Replaces one existing record without changing its stable identity.
    pub fn update(&self, record: &ReferenceRecord) -> Result<(), ReferenceStoreError> {
        let payload = serialize_record(record)?;
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        require_existing_record(&transaction, record.reference_id())?;
        reject_update_citekey_conflict(&transaction, record)?;
        update_record(&transaction, record, &payload)?;
        commit_write(transaction)
    }

    /// Deletes and returns one validated record, or reports that it is absent.
    pub fn delete(
        &self,
        reference_id: ReferenceId,
    ) -> Result<ReferenceRecord, ReferenceStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let stored = load_by_id(&transaction, reference_id)?
            .ok_or(ReferenceStoreError::ReferenceNotFound)?;
        let record = decode_stored_reference(stored)?;
        delete_record(&transaction, reference_id)?;
        commit_write(transaction)?;
        Ok(record)
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, ReferenceStoreError> {
        self.connection
            .lock()
            .map_err(|_| ReferenceStoreError::StoreUnavailable)
    }

    #[cfg(test)]
    pub(crate) fn replace_payload_for_test(&self, citekey: &str, payload: &str) {
        self.connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE reference_records SET payload_json = ?2 WHERE citekey = ?1",
                params![citekey, payload],
            )
            .unwrap();
    }
}

impl fmt::Display for ReferenceStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for ReferenceStoreError {}

impl ReferenceStoreError {
    fn message(&self) -> &'static str {
        match self {
            Self::StorageLocationUnavailable => "reference storage location is unavailable",
            Self::OpenFailed => "reference store could not be opened",
            Self::SchemaReadFailed => "reference store schema could not be read",
            Self::SchemaMigrationFailed => "reference store schema migration failed",
            Self::UnsupportedStoreSchema { .. } => "reference store schema is not supported",
            Self::InvalidStoreSchema => "reference store schema is incomplete or invalid",
            Self::StoreUnavailable => "reference store state is unavailable",
            Self::SerializationFailed => "reference record could not be serialized",
            Self::WriteFailed => "reference store write failed",
            Self::ReadFailed => "reference store read failed",
            Self::DuplicateReferenceId => "reference identity already exists",
            Self::DuplicateCitekey => "reference citekey already exists",
            Self::ReferenceNotFound => "reference record was not found",
            Self::MalformedStoredJson => "stored reference JSON is malformed",
            Self::InvalidStoredRecord { .. } => "stored reference record is invalid",
            Self::StoredSchemaMismatch => "stored reference schema index does not match payload",
            Self::StoredIdentityMismatch => {
                "stored reference identity index does not match payload"
            }
        }
    }
}

fn ensure_parent_directory(path: &Path) -> Result<(), ReferenceStoreError> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|_| ReferenceStoreError::StorageLocationUnavailable)
}

fn open_connection(path: &Path) -> Result<Connection, ReferenceStoreError> {
    Connection::open(path).map_err(|_| ReferenceStoreError::OpenFailed)
}

fn configure_connection(connection: &Connection) -> Result<(), ReferenceStoreError> {
    connection
        .busy_timeout(REFERENCE_STORE_BUSY_TIMEOUT)
        .map_err(|_| ReferenceStoreError::OpenFailed)
}

fn migrate_store(connection: &mut Connection) -> Result<(), ReferenceStoreError> {
    match read_store_schema(connection)? {
        0 => migrate_zero_to_one(connection),
        REFERENCE_STORE_SCHEMA_VERSION => Ok(()),
        found => Err(ReferenceStoreError::UnsupportedStoreSchema { found }),
    }?;
    verify_current_schema(connection)
}

fn read_store_schema(connection: &Connection) -> Result<u64, ReferenceStoreError> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| ReferenceStoreError::SchemaReadFailed)?;
    u64::try_from(version).map_err(|_| ReferenceStoreError::SchemaReadFailed)
}

fn migrate_zero_to_one(connection: &mut Connection) -> Result<(), ReferenceStoreError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| ReferenceStoreError::SchemaMigrationFailed)?;
    transaction
        .execute_batch(CREATE_SCHEMA_V1)
        .map_err(|_| ReferenceStoreError::SchemaMigrationFailed)?;
    transaction
        .commit()
        .map_err(|_| ReferenceStoreError::SchemaMigrationFailed)
}

fn verify_current_schema(connection: &Connection) -> Result<(), ReferenceStoreError> {
    verify_schema_definition(connection)?;
    connection
        .prepare(
            "SELECT reference_id, citekey, schema_version, payload_json
             FROM reference_records LIMIT 0",
        )
        .map(|_| ())
        .map_err(|_| ReferenceStoreError::InvalidStoreSchema)
}

fn verify_schema_definition(connection: &Connection) -> Result<(), ReferenceStoreError> {
    let definition = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type = 'table' AND name = 'reference_records'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| ReferenceStoreError::InvalidStoreSchema)?
        .ok_or(ReferenceStoreError::InvalidStoreSchema)?;
    let required_fragments = [
        "reference_id TEXT PRIMARY KEY NOT NULL",
        "citekey TEXT NOT NULL UNIQUE COLLATE BINARY",
        "schema_version INTEGER NOT NULL CHECK (schema_version = 1)",
        "payload_json TEXT NOT NULL",
        ") STRICT",
    ];
    if required_fragments
        .iter()
        .all(|fragment| definition.contains(fragment))
    {
        Ok(())
    } else {
        Err(ReferenceStoreError::InvalidStoreSchema)
    }
}

fn serialize_record(record: &ReferenceRecord) -> Result<String, ReferenceStoreError> {
    serde_json::to_string(record).map_err(|_| ReferenceStoreError::SerializationFailed)
}

fn begin_write(connection: &mut Connection) -> Result<Transaction<'_>, ReferenceStoreError> {
    connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| ReferenceStoreError::WriteFailed)
}

fn commit_write(transaction: Transaction<'_>) -> Result<(), ReferenceStoreError> {
    transaction
        .commit()
        .map_err(|_| ReferenceStoreError::WriteFailed)
}

fn reject_create_conflict(
    connection: &Connection,
    record: &ReferenceRecord,
) -> Result<(), ReferenceStoreError> {
    if reference_id_exists(connection, record.reference_id())? {
        return Err(ReferenceStoreError::DuplicateReferenceId);
    }
    if citekey_owner(connection, record.citekey())?.is_some() {
        return Err(ReferenceStoreError::DuplicateCitekey);
    }
    Ok(())
}

fn reject_update_citekey_conflict(
    connection: &Connection,
    record: &ReferenceRecord,
) -> Result<(), ReferenceStoreError> {
    let owner = citekey_owner(connection, record.citekey())?;
    if owner
        .as_deref()
        .is_some_and(|owner| owner != record.reference_id().to_string())
    {
        Err(ReferenceStoreError::DuplicateCitekey)
    } else {
        Ok(())
    }
}

fn require_existing_record(
    connection: &Connection,
    reference_id: ReferenceId,
) -> Result<(), ReferenceStoreError> {
    if reference_id_exists(connection, reference_id)? {
        Ok(())
    } else {
        Err(ReferenceStoreError::ReferenceNotFound)
    }
}

fn reference_id_exists(
    connection: &Connection,
    reference_id: ReferenceId,
) -> Result<bool, ReferenceStoreError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM reference_records WHERE reference_id = ?1)",
            [reference_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|_| ReferenceStoreError::ReadFailed)
}

fn citekey_owner(
    connection: &Connection,
    citekey: &str,
) -> Result<Option<String>, ReferenceStoreError> {
    connection
        .query_row(
            "SELECT reference_id FROM reference_records WHERE citekey = ?1",
            [citekey],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ReferenceStoreError::ReadFailed)
}

fn insert_record(
    connection: &Connection,
    record: &ReferenceRecord,
    payload: &str,
) -> Result<(), ReferenceStoreError> {
    connection
        .execute(
            "INSERT INTO reference_records
             (reference_id, citekey, schema_version, payload_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                record.reference_id().to_string(),
                record.citekey(),
                record.schema_version() as i64,
                payload
            ],
        )
        .map(|_| ())
        .map_err(|_| ReferenceStoreError::WriteFailed)
}

fn update_record(
    connection: &Connection,
    record: &ReferenceRecord,
    payload: &str,
) -> Result<(), ReferenceStoreError> {
    connection
        .execute(
            "UPDATE reference_records
             SET citekey = ?2, schema_version = ?3, payload_json = ?4
             WHERE reference_id = ?1",
            params![
                record.reference_id().to_string(),
                record.citekey(),
                record.schema_version() as i64,
                payload
            ],
        )
        .map(|_| ())
        .map_err(|_| ReferenceStoreError::WriteFailed)
}

fn delete_record(
    connection: &Connection,
    reference_id: ReferenceId,
) -> Result<(), ReferenceStoreError> {
    connection
        .execute(
            "DELETE FROM reference_records WHERE reference_id = ?1",
            [reference_id.to_string()],
        )
        .map(|_| ())
        .map_err(|_| ReferenceStoreError::WriteFailed)
}

fn load_by_id(
    connection: &Connection,
    reference_id: ReferenceId,
) -> Result<Option<StoredReference>, ReferenceStoreError> {
    connection
        .query_row(
            "SELECT reference_id, citekey, schema_version, payload_json
             FROM reference_records WHERE reference_id = ?1",
            [reference_id.to_string()],
            map_stored_reference,
        )
        .optional()
        .map_err(|_| ReferenceStoreError::ReadFailed)
}

fn load_by_citekey(
    connection: &Connection,
    citekey: &str,
) -> Result<Option<StoredReference>, ReferenceStoreError> {
    connection
        .query_row(
            "SELECT reference_id, citekey, schema_version, payload_json
             FROM reference_records WHERE citekey = ?1",
            [citekey],
            map_stored_reference,
        )
        .optional()
        .map_err(|_| ReferenceStoreError::ReadFailed)
}

fn load_all(connection: &Connection) -> Result<Vec<StoredReference>, ReferenceStoreError> {
    let mut statement = connection
        .prepare(
            "SELECT reference_id, citekey, schema_version, payload_json
             FROM reference_records ORDER BY citekey COLLATE BINARY, reference_id",
        )
        .map_err(|_| ReferenceStoreError::ReadFailed)?;
    let rows = statement
        .query_map([], map_stored_reference)
        .map_err(|_| ReferenceStoreError::ReadFailed)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| ReferenceStoreError::ReadFailed)
}

fn map_stored_reference(row: &Row<'_>) -> rusqlite::Result<StoredReference> {
    Ok(StoredReference {
        reference_id: row.get(0)?,
        citekey: row.get(1)?,
        schema_version: row.get(2)?,
        payload_json: row.get(3)?,
    })
}

fn decode_stored_reference(
    stored: StoredReference,
) -> Result<ReferenceRecord, ReferenceStoreError> {
    let value = serde_json::from_str::<Value>(&stored.payload_json)
        .map_err(|_| ReferenceStoreError::MalformedStoredJson)?;
    let record = ReferenceRecord::from_json_value(value)
        .map_err(|cause| ReferenceStoreError::InvalidStoredRecord { cause })?;
    validate_stored_indexes(&stored, &record)?;
    Ok(record)
}

fn validate_stored_indexes(
    stored: &StoredReference,
    record: &ReferenceRecord,
) -> Result<(), ReferenceStoreError> {
    if stored.schema_version != record.schema_version() as i64
        || stored.schema_version != REFERENCE_RECORD_SCHEMA_VERSION as i64
    {
        return Err(ReferenceStoreError::StoredSchemaMismatch);
    }
    if stored.reference_id != record.reference_id().to_string()
        || stored.citekey != record.citekey()
    {
        return Err(ReferenceStoreError::StoredIdentityMismatch);
    }
    Ok(())
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
