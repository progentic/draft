use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use serde::Serialize;

use crate::documents::envelope::{DocumentEnvelope, DocumentId};

/// Process-local owner of validated live document handles.
#[derive(Default)]
pub struct DocumentRegistry {
    handles: Mutex<HashMap<DocumentId, LiveDocumentHandle>>,
    file_operations: Mutex<()>,
}

/// Bounded failures produced by live document-handle operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DocumentRegistryError {
    AlreadyOpen,
    NotOpen,
    SourcePathInUse,
    RegistryUnavailable,
}

struct LiveDocumentHandle {
    envelope: DocumentEnvelope,
    source_path: Option<PathBuf>,
}

impl DocumentRegistry {
    /// Creates an empty process-local document registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes ownership of one validated envelope while its document is open.
    pub fn open(&self, envelope: DocumentEnvelope) -> Result<(), DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        register_handle(&mut handles, envelope, None)
    }

    /// Opens a validated envelope with a Rust-selected source path.
    pub fn open_from_path(
        &self,
        envelope: DocumentEnvelope,
        source_path: PathBuf,
    ) -> Result<(), DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        register_handle(&mut handles, envelope, Some(source_path))
    }

    /// Returns the source path retained by Rust for one open document.
    pub fn source_path(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PathBuf>, DocumentRegistryError> {
        let handles = self.lock_handles()?;
        source_path_for(&handles, document_id)
    }

    /// Confirms that no other live document owns a Rust-selected source path.
    pub fn ensure_source_path_available(
        &self,
        document_id: DocumentId,
        source_path: &Path,
    ) -> Result<(), DocumentRegistryError> {
        let handles = self.lock_handles()?;
        reject_source_path_conflict(&handles, document_id, Some(source_path))
    }

    /// Replaces the validated snapshot while preserving its existing source path.
    pub fn update(&self, envelope: DocumentEnvelope) -> Result<(), DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        update_handle(&mut handles, envelope)
    }

    /// Replaces the validated snapshot and records its Rust-selected source path.
    pub fn update_source(
        &self,
        envelope: DocumentEnvelope,
        source_path: PathBuf,
    ) -> Result<(), DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        update_handle_source(&mut handles, envelope, source_path)
    }

    /// Releases one live handle and returns its validated in-memory envelope.
    pub fn close(
        &self,
        document_id: DocumentId,
    ) -> Result<DocumentEnvelope, DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        remove_handle(&mut handles, document_id)
    }

    pub(crate) fn lock_file_operations(&self) -> Result<MutexGuard<'_, ()>, DocumentRegistryError> {
        self.file_operations
            .lock()
            .map_err(|_| DocumentRegistryError::RegistryUnavailable)
    }

    fn lock_handles(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<DocumentId, LiveDocumentHandle>>, DocumentRegistryError>
    {
        self.handles
            .lock()
            .map_err(|_| DocumentRegistryError::RegistryUnavailable)
    }
}

impl fmt::Display for DocumentRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for DocumentRegistryError {}

impl DocumentRegistryError {
    fn message(self) -> &'static str {
        match self {
            Self::AlreadyOpen => "document already has a live editing handle",
            Self::NotOpen => "document does not have a live editing handle",
            Self::SourcePathInUse => "document source path already has a live editing handle",
            Self::RegistryUnavailable => "document registry is unavailable",
        }
    }
}

impl LiveDocumentHandle {
    fn new(envelope: DocumentEnvelope, source_path: Option<PathBuf>) -> Self {
        Self {
            envelope,
            source_path,
        }
    }

    fn source_path(&self) -> Option<PathBuf> {
        self.source_path.clone()
    }

    fn owns_source_path(&self, source_path: &Path) -> bool {
        self.source_path.as_deref() == Some(source_path)
    }

    fn update(&mut self, envelope: DocumentEnvelope) {
        self.envelope = envelope;
    }

    fn update_source(&mut self, envelope: DocumentEnvelope, source_path: PathBuf) {
        self.envelope = envelope;
        self.source_path = Some(source_path);
    }

    fn into_envelope(self) -> DocumentEnvelope {
        self.envelope
    }
}

fn register_handle(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    envelope: DocumentEnvelope,
    source_path: Option<PathBuf>,
) -> Result<(), DocumentRegistryError> {
    let document_id = envelope.document_id();
    reject_source_path_conflict(handles, document_id, source_path.as_deref())?;

    match handles.entry(document_id) {
        Entry::Occupied(_) => Err(DocumentRegistryError::AlreadyOpen),
        Entry::Vacant(entry) => {
            entry.insert(LiveDocumentHandle::new(envelope, source_path));
            Ok(())
        }
    }
}

fn source_path_for(
    handles: &HashMap<DocumentId, LiveDocumentHandle>,
    document_id: DocumentId,
) -> Result<Option<PathBuf>, DocumentRegistryError> {
    handles
        .get(&document_id)
        .map(LiveDocumentHandle::source_path)
        .ok_or(DocumentRegistryError::NotOpen)
}

fn update_handle(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    envelope: DocumentEnvelope,
) -> Result<(), DocumentRegistryError> {
    let handle = handle_for_update(handles, envelope.document_id())?;
    handle.update(envelope);
    Ok(())
}

fn update_handle_source(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    envelope: DocumentEnvelope,
    source_path: PathBuf,
) -> Result<(), DocumentRegistryError> {
    let document_id = envelope.document_id();
    reject_source_path_conflict(handles, document_id, Some(&source_path))?;
    let handle = handle_for_update(handles, document_id)?;
    handle.update_source(envelope, source_path);
    Ok(())
}

fn reject_source_path_conflict(
    handles: &HashMap<DocumentId, LiveDocumentHandle>,
    document_id: DocumentId,
    source_path: Option<&Path>,
) -> Result<(), DocumentRegistryError> {
    let Some(source_path) = source_path else {
        return Ok(());
    };
    let path_in_use = handles
        .iter()
        .any(|(open_id, handle)| *open_id != document_id && handle.owns_source_path(source_path));

    if path_in_use {
        Err(DocumentRegistryError::SourcePathInUse)
    } else {
        Ok(())
    }
}

fn handle_for_update(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    document_id: DocumentId,
) -> Result<&mut LiveDocumentHandle, DocumentRegistryError> {
    handles
        .get_mut(&document_id)
        .ok_or(DocumentRegistryError::NotOpen)
}

fn remove_handle(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    document_id: DocumentId,
) -> Result<DocumentEnvelope, DocumentRegistryError> {
    handles
        .remove(&document_id)
        .map(LiveDocumentHandle::into_envelope)
        .ok_or(DocumentRegistryError::NotOpen)
}

#[cfg(test)]
mod tests {
    use std::thread;

    use serde_json::json;

    use super::*;

    const FIRST_DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
    const SECOND_DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000002";

    #[test]
    fn open_document_twice_returns_already_open() {
        let registry = DocumentRegistry::new();
        let first = envelope(FIRST_DOCUMENT_ID, "First version");
        let duplicate = envelope(FIRST_DOCUMENT_ID, "Duplicate version");

        assert_eq!(registry.open(first), Ok(()));
        assert_eq!(
            registry.open(duplicate),
            Err(DocumentRegistryError::AlreadyOpen),
        );
    }

    #[test]
    fn rejected_duplicate_does_not_replace_live_document() {
        let registry = DocumentRegistry::new();
        let original = envelope(FIRST_DOCUMENT_ID, "Original");
        let document_id = original.document_id();

        registry
            .open(original.clone())
            .expect("document should open");
        let duplicate = registry.open(envelope(FIRST_DOCUMENT_ID, "Replacement"));

        assert_eq!(duplicate, Err(DocumentRegistryError::AlreadyOpen));
        assert_eq!(registry.close(document_id), Ok(original));
    }

    #[test]
    fn closing_document_releases_live_handle() {
        let registry = DocumentRegistry::new();
        let document = envelope(FIRST_DOCUMENT_ID, "Reusable");
        let document_id = document.document_id();

        registry
            .open(document.clone())
            .expect("document should open");
        assert_eq!(registry.close(document_id), Ok(document.clone()));
        assert_eq!(registry.open(document), Ok(()));
    }

    #[test]
    fn closing_unknown_document_returns_not_open() {
        let registry = DocumentRegistry::new();
        let document_id = envelope(FIRST_DOCUMENT_ID, "Unknown").document_id();

        assert_eq!(
            registry.close(document_id),
            Err(DocumentRegistryError::NotOpen),
        );
    }

    #[test]
    fn distinct_documents_open_independently() {
        let registry = DocumentRegistry::new();

        assert_eq!(registry.open(envelope(FIRST_DOCUMENT_ID, "First")), Ok(()),);
        assert_eq!(
            registry.open(envelope(SECOND_DOCUMENT_ID, "Second")),
            Ok(()),
        );
    }

    #[test]
    fn concurrent_open_allows_one_live_handle() {
        let registry = DocumentRegistry::new();
        let first = envelope(FIRST_DOCUMENT_ID, "First contender");
        let second = envelope(FIRST_DOCUMENT_ID, "Second contender");

        let outcomes = concurrent_open(&registry, first, second);

        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        assert_eq!(outcomes.iter().filter(is_already_open).count(), 1);
    }

    #[test]
    fn poisoned_registry_returns_unavailable() {
        let registry = DocumentRegistry::new();
        poison_registry(&registry);

        assert_eq!(
            registry.open(envelope(FIRST_DOCUMENT_ID, "Unavailable")),
            Err(DocumentRegistryError::RegistryUnavailable),
        );
    }

    #[test]
    fn source_path_cannot_back_two_live_handles() {
        let registry = DocumentRegistry::new();
        let source_path = PathBuf::from("shared-document.draft");
        registry
            .open_from_path(envelope(FIRST_DOCUMENT_ID, "First"), source_path.clone())
            .expect("first document should open");

        assert_eq!(
            registry.open_from_path(envelope(SECOND_DOCUMENT_ID, "Second"), source_path),
            Err(DocumentRegistryError::SourcePathInUse),
        );
    }

    #[test]
    fn source_path_conflict_preserves_unattached_handle() {
        let registry = DocumentRegistry::new();
        let source_path = PathBuf::from("owned-document.draft");
        let second = envelope(SECOND_DOCUMENT_ID, "Second");
        let second_id = second.document_id();
        registry
            .open_from_path(envelope(FIRST_DOCUMENT_ID, "First"), source_path.clone())
            .expect("first document should open");
        registry
            .open(second.clone())
            .expect("second document should open");

        assert_eq!(
            registry.update_source(envelope(SECOND_DOCUMENT_ID, "Replacement"), source_path,),
            Err(DocumentRegistryError::SourcePathInUse),
        );
        assert_eq!(registry.source_path(second_id), Ok(None));
        assert_eq!(registry.close(second_id), Ok(second));
    }

    #[test]
    fn registry_failure_shape_is_stable() {
        let errors = [
            DocumentRegistryError::AlreadyOpen,
            DocumentRegistryError::NotOpen,
            DocumentRegistryError::SourcePathInUse,
            DocumentRegistryError::RegistryUnavailable,
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "already_open" },
                { "code": "not_open" },
                { "code": "source_path_in_use" },
                { "code": "registry_unavailable" }
            ]),
        );
    }

    fn concurrent_open(
        registry: &DocumentRegistry,
        first: DocumentEnvelope,
        second: DocumentEnvelope,
    ) -> [Result<(), DocumentRegistryError>; 2] {
        thread::scope(|scope| {
            let first_open = scope.spawn(|| registry.open(first));
            let second_open = scope.spawn(|| registry.open(second));

            [join_open(first_open), join_open(second_open)]
        })
    }

    fn join_open(
        open: thread::ScopedJoinHandle<'_, Result<(), DocumentRegistryError>>,
    ) -> Result<(), DocumentRegistryError> {
        open.join()
            .expect("document-open contender should not panic")
    }

    fn is_already_open(outcome: &&Result<(), DocumentRegistryError>) -> bool {
        matches!(outcome, Err(DocumentRegistryError::AlreadyOpen))
    }

    fn poison_registry(registry: &DocumentRegistry) {
        let outcome = thread::scope(|scope| {
            scope
                .spawn(|| {
                    let _handles = registry.handles.lock().expect("registry should lock");
                    panic!("intentional registry poison");
                })
                .join()
        });

        assert!(outcome.is_err());
    }

    fn envelope(document_id: &str, title: &str) -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(json!({
            "schema_version": 1,
            "document_id": document_id,
            "title": title,
            "document": { "type": "doc", "content": [] }
        }))
        .expect("test envelope should validate")
    }
}
