use tauri::{App, Manager};

use crate::network::client::{NetworkClient, NetworkClientError};

/// Constructs and registers the shared Rust-owned network client.
pub(crate) fn initialize_network_client(app: &mut App) -> Result<(), NetworkClientError> {
    app.manage(NetworkClient::new()?);
    Ok(())
}
