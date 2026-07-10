use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

pub(crate) struct TestReferenceStorePath {
    directory: PathBuf,
    path: PathBuf,
}

impl TestReferenceStorePath {
    pub(crate) fn new(label: &str) -> Self {
        let directory = unique_test_directory();
        fs::create_dir_all(&directory).expect("test directory should exist");
        Self {
            path: directory.join(format!("{label}.sqlite3")),
            directory,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestReferenceStorePath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn unique_test_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("reference-store-tests")
        .join(Uuid::new_v4().to_string())
}
