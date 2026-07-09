use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::documents::atomic_write::write_document_atomically;

pub(crate) struct TestDocumentPath {
    path: PathBuf,
}

impl TestDocumentPath {
    pub(crate) fn new(label: &str) -> Self {
        let directory = test_directory();
        fs::create_dir_all(&directory).expect("test directory should exist");
        Self {
            path: directory.join(unique_file_name(label)),
        }
    }

    pub(crate) fn under_missing_parent(label: &str) -> Self {
        let path = test_directory()
            .join(unique_file_name(label))
            .join("document.draft");
        Self { path }
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
        let _ = fs::remove_file(&self.path);
    }
}

fn test_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("phase13-tests")
}

fn unique_file_name(label: &str) -> String {
    format!("{label}-{}.draft", Uuid::new_v4())
}
