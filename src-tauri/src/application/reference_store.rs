use std::{error::Error, fmt, path::PathBuf};

use tauri::{App, Manager};

use crate::references::store::{ReferenceStore, ReferenceStoreError, reference_store_path};

/// Bounded startup failures for the Rust-owned reference-store state.
#[derive(Debug)]
pub(crate) enum ReferenceStoreInitializationError {
    AppDataDirectoryUnavailable,
    Store { cause: ReferenceStoreError },
}

/// Resolves, opens, and registers the production reference store.
pub(crate) fn initialize_reference_store(
    app: &mut App,
) -> Result<(), ReferenceStoreInitializationError> {
    app.manage(open_reference_store(app)?);
    Ok(())
}

impl fmt::Display for ReferenceStoreInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppDataDirectoryUnavailable => {
                formatter.write_str("application data directory is unavailable")
            }
            Self::Store { cause } => {
                write!(formatter, "reference store initialization failed: {cause}")
            }
        }
    }
}

impl Error for ReferenceStoreInitializationError {}

fn open_reference_store(app: &App) -> Result<ReferenceStore, ReferenceStoreInitializationError> {
    let path = resolve_reference_store_path(app)?;
    ReferenceStore::open(&path).map_err(|cause| ReferenceStoreInitializationError::Store { cause })
}

fn resolve_reference_store_path(app: &App) -> Result<PathBuf, ReferenceStoreInitializationError> {
    app.path()
        .app_data_dir()
        .map(|directory| reference_store_path(&directory))
        .map_err(|_| ReferenceStoreInitializationError::AppDataDirectoryUnavailable)
}
