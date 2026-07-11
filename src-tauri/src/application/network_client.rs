use std::sync::Arc;

use tauri::{App, Manager};

use crate::network::{
    client::{NetworkClient, NetworkClientError},
    connectivity::ConnectivityPolicy,
};

/// Constructs and registers the shared Rust-owned network client.
pub(crate) fn initialize_network_client(app: &mut App) -> Result<(), NetworkClientError> {
    let connectivity = Arc::new(ConnectivityPolicy::default());
    let client = NetworkClient::new(Arc::clone(&connectivity))?;
    app.manage(connectivity);
    app.manage(client);
    Ok(())
}
