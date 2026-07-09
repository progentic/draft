mod application;
mod commands;
mod events;
pub mod workers;

/// Starts the DRAFT desktop runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(workers::cancellation::WorkerCancellationRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::runtime_status::get_runtime_status,
            commands::worker_cancellation::cancel_worker
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the DRAFT desktop runtime");
}
