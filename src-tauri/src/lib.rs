pub mod analysis;
mod application;
pub mod citations;
mod commands;
mod desktop_menu;
mod diagnostics;
pub mod documents;
pub mod events;
pub mod exports;
pub mod formatting;
pub mod imports;
pub mod interoperability;
pub mod jobs;
pub mod network;
pub mod references;
pub mod research;
pub mod secrets;
mod system_browser;
pub mod workers;

#[cfg(test)]
mod critical_paths_tests;

/// Starts the DRAFT desktop runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_menu_event(desktop_menu::handle_event)
        .setup(|app| {
            desktop_menu::install(app)?;
            application::secret_store::initialize_secret_store(app);
            application::network_client::initialize_network_client(app)?;
            application::reference_store::initialize_reference_store(app)?;
            application::job_store::initialize_job_store(app)?;
            Ok(())
        })
        .manage(documents::registry::DocumentRegistry::new())
        .manage(application::open_requests::ApplicationOpenQueue::default())
        .manage(workers::cancellation::WorkerCancellationRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::application_open::open_application_document,
            commands::citation_resolution::resolve_citation,
            commands::connectivity::get_connectivity_mode,
            commands::connectivity::set_connectivity_mode,
            commands::diagnostic_snapshot::get_diagnostic_snapshot,
            commands::document_create::create_document,
            commands::document_close::close_document,
            commands::document_open::open_document,
            commands::document_save::save_document,
            commands::docx_export::export_document,
            commands::external_access::open_external_access,
            commands::external_document_save::save_external_document,
            commands::formatting_review::run_formatting_review,
            commands::reference_library::add_reference,
            commands::reference_library::list_references,
            commands::runtime_status::get_runtime_status,
            commands::native_menu::set_native_menu_state,
            commands::text_analysis::run_text_analysis,
            commands::worker_cancellation::cancel_worker
        ])
        .build(tauri::generate_context!())
        .expect("failed to build the DRAFT desktop runtime");
    app.run(application::open_requests::handle_run_event);
}
