use tauri::{App, Manager};

use crate::secrets::store::SecretStore;

/// Registers the lazy Rust-owned native credential store.
pub(crate) fn initialize_secret_store(app: &mut App) {
    app.manage(SecretStore::native());
}
