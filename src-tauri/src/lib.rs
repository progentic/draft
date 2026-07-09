mod application;
mod commands;

/// Starts the DRAFT desktop runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::runtime_status::get_runtime_status
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the DRAFT desktop runtime");
}
