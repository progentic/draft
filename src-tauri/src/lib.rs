mod application;
pub mod citations;
mod commands;
pub mod documents;
mod events;
pub mod references;
pub mod workers;

/// Starts the DRAFT desktop runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            application::reference_store::initialize_reference_store(app)?;
            Ok(())
        })
        .manage(documents::registry::DocumentRegistry::new())
        .manage(workers::cancellation::WorkerCancellationRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::citation_resolution::resolve_citation,
            commands::document_open::open_document,
            commands::document_save::save_document,
            commands::runtime_status::get_runtime_status,
            commands::worker_cancellation::cancel_worker
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the DRAFT desktop runtime");
}
