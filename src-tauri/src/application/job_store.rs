use std::{error::Error, fmt, path::PathBuf};

use tauri::{App, Manager};

use crate::jobs::store::{PdfImportJobStore, PdfImportJobStoreError, job_store_path};

/// Bounded startup failures for the Rust-owned durable job state.
#[derive(Debug)]
pub(crate) enum JobStoreInitializationError {
    AppDataDirectoryUnavailable,
    Store { cause: PdfImportJobStoreError },
}

/// Resolves, opens, recovers, and registers the production job store.
pub(crate) fn initialize_job_store(app: &mut App) -> Result<(), JobStoreInitializationError> {
    app.manage(open_job_store(app)?);
    Ok(())
}

impl fmt::Display for JobStoreInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppDataDirectoryUnavailable => {
                formatter.write_str("application data directory is unavailable")
            }
            Self::Store { cause } => write!(formatter, "job store initialization failed: {cause}"),
        }
    }
}

impl Error for JobStoreInitializationError {}

fn open_job_store(app: &App) -> Result<PdfImportJobStore, JobStoreInitializationError> {
    let path = resolve_job_store_path(app)?;
    PdfImportJobStore::open(&path).map_err(|cause| JobStoreInitializationError::Store { cause })
}

fn resolve_job_store_path(app: &App) -> Result<PathBuf, JobStoreInitializationError> {
    app.path()
        .app_data_dir()
        .map(|directory| job_store_path(&directory))
        .map_err(|_| JobStoreInitializationError::AppDataDirectoryUnavailable)
}
