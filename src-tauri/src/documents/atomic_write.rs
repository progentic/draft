use std::{
    io::{self, Write},
    path::Path,
};

use serde::Serialize;
use tempfile::{Builder, NamedTempFile};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum AtomicDocumentWriteError {
    OpenTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    ReplaceTarget,
    CleanupTemporaryFile,
    SyncParentDirectory,
}

pub(crate) fn write_document_atomically(
    target_path: &Path,
    contents: &[u8],
) -> Result<(), AtomicDocumentWriteError> {
    write_document_with_faults(target_path, contents, &NoWriteFaults)
}

impl AtomicDocumentWriteError {
    pub(crate) fn target_was_replaced(self) -> bool {
        self == Self::SyncParentDirectory
    }
}

fn write_document_with_faults<Faults>(
    target_path: &Path,
    contents: &[u8],
    faults: &Faults,
) -> Result<(), AtomicDocumentWriteError>
where
    Faults: WriteFaults,
{
    let parent = parent_directory(target_path);
    let mut temporary_file = create_temporary_file(parent)?;
    if let Err(cause) = prepare_temporary_file(&mut temporary_file, contents, faults) {
        return Err(clean_up_failed_write(temporary_file, cause));
    }
    persist_temporary_file(temporary_file, target_path)?;
    synchronize_parent_directory(parent, faults)
}

fn parent_directory(target_path: &Path) -> &Path {
    target_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn create_temporary_file(parent: &Path) -> Result<NamedTempFile, AtomicDocumentWriteError> {
    Builder::new()
        .prefix(".draft-save-")
        .tempfile_in(parent)
        .map_err(|_| AtomicDocumentWriteError::OpenTemporaryFile)
}

fn prepare_temporary_file<Faults>(
    temporary_file: &mut NamedTempFile,
    contents: &[u8],
    faults: &Faults,
) -> Result<(), AtomicDocumentWriteError>
where
    Faults: WriteFaults,
{
    faults.check(WriteCheckpoint::BeforeWrite)?;
    write_contents(temporary_file, contents, faults)?;
    faults.check(WriteCheckpoint::BeforeContentSync)?;
    synchronize_contents(temporary_file)?;
    faults.check(WriteCheckpoint::BeforeReplace)
}

fn write_contents<Faults>(
    temporary_file: &mut NamedTempFile,
    contents: &[u8],
    faults: &Faults,
) -> Result<(), AtomicDocumentWriteError>
where
    Faults: WriteFaults,
{
    let (first, second) = contents.split_at(contents.len() / 2);
    write_chunk(temporary_file, first)?;
    faults.check(WriteCheckpoint::DuringWrite)?;
    write_chunk(temporary_file, second)
}

fn write_chunk(
    temporary_file: &mut NamedTempFile,
    contents: &[u8],
) -> Result<(), AtomicDocumentWriteError> {
    temporary_file
        .write_all(contents)
        .map_err(|_| AtomicDocumentWriteError::WriteTemporaryFile)
}

fn synchronize_contents(temporary_file: &NamedTempFile) -> Result<(), AtomicDocumentWriteError> {
    temporary_file
        .as_file()
        .sync_all()
        .map_err(|_| AtomicDocumentWriteError::SyncTemporaryFile)
}

fn persist_temporary_file(
    temporary_file: NamedTempFile,
    target_path: &Path,
) -> Result<(), AtomicDocumentWriteError> {
    temporary_file
        .persist(target_path)
        .map(|_| ())
        .map_err(|error| clean_up_failed_write(error.file, AtomicDocumentWriteError::ReplaceTarget))
}

fn clean_up_failed_write(
    temporary_file: NamedTempFile,
    cause: AtomicDocumentWriteError,
) -> AtomicDocumentWriteError {
    match temporary_file.close() {
        Ok(()) => cause,
        Err(_) => AtomicDocumentWriteError::CleanupTemporaryFile,
    }
}

fn synchronize_parent_directory<Faults>(
    parent: &Path,
    faults: &Faults,
) -> Result<(), AtomicDocumentWriteError>
where
    Faults: WriteFaults,
{
    faults.check(WriteCheckpoint::BeforeParentSync)?;
    sync_directory(parent).map_err(|_| AtomicDocumentWriteError::SyncParentDirectory)
}

#[cfg(unix)]
fn sync_directory(parent: &Path) -> io::Result<()> {
    std::fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriteCheckpoint {
    BeforeWrite,
    DuringWrite,
    BeforeContentSync,
    BeforeReplace,
    BeforeParentSync,
}

trait WriteFaults {
    fn check(&self, checkpoint: WriteCheckpoint) -> Result<(), AtomicDocumentWriteError>;
}

struct NoWriteFaults;

impl WriteFaults for NoWriteFaults {
    fn check(&self, _checkpoint: WriteCheckpoint) -> Result<(), AtomicDocumentWriteError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, ffi::OsString, fs};

    use super::*;
    use crate::documents::test_support::TestDocumentPath;

    #[test]
    fn atomic_writer_creates_complete_document() {
        let target = TestDocumentPath::new("atomic-create");

        write_document_atomically(target.path(), b"complete document")
            .expect("atomic write should succeed");

        assert_eq!(
            fs::read(target.path()).expect("document should read"),
            b"complete document"
        );
    }

    #[test]
    fn platform_replacement_preserves_complete_document() {
        let target = TestDocumentPath::new("atomic-replace");
        write_document_atomically(target.path(), b"old complete")
            .expect("initial atomic write should succeed");

        write_document_atomically(target.path(), b"new complete")
            .expect("replacement atomic write should succeed");

        assert_eq!(
            fs::read(target.path()).expect("document should read"),
            b"new complete"
        );
    }

    #[test]
    fn atomic_writer_rejects_missing_parent() {
        let target = TestDocumentPath::under_missing_parent("atomic-missing-parent");

        assert_eq!(
            write_document_atomically(target.path(), b"document"),
            Err(AtomicDocumentWriteError::OpenTemporaryFile),
        );
        assert!(!target.path().exists());
    }

    #[test]
    fn interrupted_save_preserves_complete_source() {
        for checkpoint in pre_replacement_checkpoints() {
            let target = TestDocumentPath::new("atomic-interruption");
            target.write(b"prior complete document");

            let result = write_document_with_faults(
                target.path(),
                b"replacement document",
                &FailAt(checkpoint),
            );

            assert!(result.is_err());
            assert_eq!(fs::read(target.path()).unwrap(), b"prior complete document");
        }
    }

    #[test]
    fn interrupted_save_cleans_temporary_file() {
        for checkpoint in pre_replacement_checkpoints() {
            let target = TestDocumentPath::new("atomic-cleanup");
            target.write(b"prior complete document");
            let before = directory_entries(target.path());

            let _ = write_document_with_faults(
                target.path(),
                b"replacement document",
                &FailAt(checkpoint),
            );

            assert_eq!(directory_entries(target.path()), before);
        }
    }

    #[test]
    fn failed_replacement_cleans_temporary_file() {
        let target = TestDocumentPath::new("atomic-replacement-failure");
        fs::create_dir(target.path()).expect("target directory should exist");
        let before = directory_entries(target.path());

        let result = write_document_atomically(target.path(), b"document");

        assert_eq!(result, Err(AtomicDocumentWriteError::ReplaceTarget));
        assert!(target.path().is_dir());
        assert_eq!(directory_entries(target.path()), before);
        fs::remove_dir(target.path()).expect("target directory should be removed");
    }

    #[test]
    fn parent_sync_failure_leaves_new_complete_source() {
        let target = TestDocumentPath::new("atomic-parent-sync");
        target.write(b"prior complete document");
        let before = directory_entries(target.path());

        let result = write_document_with_faults(
            target.path(),
            b"new complete document",
            &FailAt(WriteCheckpoint::BeforeParentSync),
        );

        assert_eq!(result, Err(AtomicDocumentWriteError::SyncParentDirectory));
        assert_eq!(fs::read(target.path()).unwrap(), b"new complete document");
        assert_eq!(directory_entries(target.path()), before);
    }

    #[test]
    fn atomic_write_failure_shape_is_stable() {
        let failures = [
            AtomicDocumentWriteError::OpenTemporaryFile,
            AtomicDocumentWriteError::WriteTemporaryFile,
            AtomicDocumentWriteError::SyncTemporaryFile,
            AtomicDocumentWriteError::ReplaceTarget,
            AtomicDocumentWriteError::CleanupTemporaryFile,
            AtomicDocumentWriteError::SyncParentDirectory,
        ];

        assert_eq!(
            serde_json::to_value(failures).unwrap(),
            serde_json::json!([
                { "code": "open_temporary_file" },
                { "code": "write_temporary_file" },
                { "code": "sync_temporary_file" },
                { "code": "replace_target" },
                { "code": "cleanup_temporary_file" },
                { "code": "sync_parent_directory" }
            ])
        );
    }

    fn pre_replacement_checkpoints() -> [WriteCheckpoint; 4] {
        [
            WriteCheckpoint::BeforeWrite,
            WriteCheckpoint::DuringWrite,
            WriteCheckpoint::BeforeContentSync,
            WriteCheckpoint::BeforeReplace,
        ]
    }

    fn directory_entries(target_path: &Path) -> BTreeSet<OsString> {
        fs::read_dir(target_path.parent().unwrap())
            .expect("test directory should read")
            .map(|entry| entry.expect("entry should read").file_name())
            .collect()
    }

    struct FailAt(WriteCheckpoint);

    impl WriteFaults for FailAt {
        fn check(&self, checkpoint: WriteCheckpoint) -> Result<(), AtomicDocumentWriteError> {
            if self.0 == checkpoint {
                return Err(error_for(checkpoint));
            }
            Ok(())
        }
    }

    fn error_for(checkpoint: WriteCheckpoint) -> AtomicDocumentWriteError {
        match checkpoint {
            WriteCheckpoint::BeforeWrite | WriteCheckpoint::DuringWrite => {
                AtomicDocumentWriteError::WriteTemporaryFile
            }
            WriteCheckpoint::BeforeContentSync => AtomicDocumentWriteError::SyncTemporaryFile,
            WriteCheckpoint::BeforeReplace => AtomicDocumentWriteError::ReplaceTarget,
            WriteCheckpoint::BeforeParentSync => AtomicDocumentWriteError::SyncParentDirectory,
        }
    }
}
