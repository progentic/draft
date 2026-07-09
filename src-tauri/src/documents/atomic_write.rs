use std::{io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AtomicDocumentWriteError {
    Open,
    Write,
    Sync,
    Commit,
}

pub(crate) fn write_document_atomically(
    target_path: &Path,
    contents: &[u8],
) -> Result<(), AtomicDocumentWriteError> {
    let mut writer = open_writer(target_path)?;
    write_contents(&mut writer, contents)?;
    sync_contents(&writer)?;
    commit_writer(writer)
}

fn open_writer(target_path: &Path) -> Result<AtomicWriteFile, AtomicDocumentWriteError> {
    AtomicWriteFile::open(target_path).map_err(|_| AtomicDocumentWriteError::Open)
}

fn write_contents(
    writer: &mut AtomicWriteFile,
    contents: &[u8],
) -> Result<(), AtomicDocumentWriteError> {
    writer
        .write_all(contents)
        .map_err(|_| AtomicDocumentWriteError::Write)
}

fn sync_contents(writer: &AtomicWriteFile) -> Result<(), AtomicDocumentWriteError> {
    writer
        .sync_all()
        .map_err(|_| AtomicDocumentWriteError::Sync)
}

fn commit_writer(writer: AtomicWriteFile) -> Result<(), AtomicDocumentWriteError> {
    writer
        .commit()
        .map_err(|_| AtomicDocumentWriteError::Commit)
}

#[cfg(test)]
mod tests {
    use std::fs;

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
    fn atomic_writer_replaces_complete_document() {
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
            Err(AtomicDocumentWriteError::Open),
        );
        assert!(!target.path().exists());
    }
}
