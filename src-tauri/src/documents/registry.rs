use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt,
    sync::{Mutex, MutexGuard},
};

use crate::documents::envelope::{DocumentEnvelope, DocumentId};

/// Process-local owner of validated live document handles.
#[derive(Default)]
pub struct DocumentRegistry {
    handles: Mutex<HashMap<DocumentId, LiveDocumentHandle>>,
}

/// Bounded failures produced by live document-handle operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentRegistryError {
    AlreadyOpen,
    NotOpen,
    RegistryUnavailable,
}

struct LiveDocumentHandle {
    envelope: DocumentEnvelope,
}

impl DocumentRegistry {
    /// Creates an empty process-local document registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes ownership of one validated envelope while its document is open.
    pub fn open(&self, envelope: DocumentEnvelope) -> Result<(), DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        register_handle(&mut handles, envelope)
    }

    /// Releases one live handle and returns its validated in-memory envelope.
    pub fn close(
        &self,
        document_id: DocumentId,
    ) -> Result<DocumentEnvelope, DocumentRegistryError> {
        let mut handles = self.lock_handles()?;
        remove_handle(&mut handles, document_id)
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
            Self::RegistryUnavailable => "document registry is unavailable",
        }
    }
}

impl LiveDocumentHandle {
    fn new(envelope: DocumentEnvelope) -> Self {
        Self { envelope }
    }

    fn into_envelope(self) -> DocumentEnvelope {
        self.envelope
    }
}

fn register_handle(
    handles: &mut HashMap<DocumentId, LiveDocumentHandle>,
    envelope: DocumentEnvelope,
) -> Result<(), DocumentRegistryError> {
    match handles.entry(envelope.document_id()) {
        Entry::Occupied(_) => Err(DocumentRegistryError::AlreadyOpen),
        Entry::Vacant(entry) => {
            entry.insert(LiveDocumentHandle::new(envelope));
            Ok(())
        }
    }
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
