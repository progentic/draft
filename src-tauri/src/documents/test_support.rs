use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::documents::atomic_write::write_document_atomically;

pub(crate) struct TestDocumentPath {
    directory: PathBuf,
    path: PathBuf,
}

impl TestDocumentPath {
    pub(crate) fn new(label: &str) -> Self {
        let directory = unique_test_directory();
        fs::create_dir_all(&directory).expect("test directory should exist");
        Self {
            path: directory.join(format!("{label}.draft")),
            directory,
        }
    }

    pub(crate) fn under_missing_parent(label: &str) -> Self {
        let directory = unique_test_directory();
        fs::create_dir_all(&directory).expect("test directory should exist");
        let path = directory.join(label).join("document.draft");
        Self { directory, path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn write(&self, contents: &[u8]) {
        write_document_atomically(&self.path, contents).expect("fixture should write");
    }
}

impl Drop for TestDocumentPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn unique_test_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("document-tests")
        .join(Uuid::new_v4().to_string())
}
