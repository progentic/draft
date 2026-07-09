use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

const DOCUMENT_FILTER_NAME: &str = "DRAFT document";
const DOCUMENT_EXTENSIONS: &[&str] = &["draft", "json"];
const DEFAULT_DOCUMENT_FILE_NAME: &str = "Untitled.draft";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnsupportedFileLocation;

pub(crate) fn select_open_document(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let selected = app_handle
        .dialog()
        .file()
        .set_title("Open DRAFT document")
        .add_filter(DOCUMENT_FILTER_NAME, DOCUMENT_EXTENSIONS)
        .blocking_pick_file();
    selected_path(selected)
}

pub(crate) fn select_save_document(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let selected = app_handle
        .dialog()
        .file()
        .set_title("Save DRAFT document")
        .set_file_name(DEFAULT_DOCUMENT_FILE_NAME)
        .add_filter(DOCUMENT_FILTER_NAME, &["draft"])
        .blocking_save_file();
    selected_path(selected)
}

fn selected_path(
    selected: Option<tauri_plugin_dialog::FilePath>,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    selected
        .map(tauri_plugin_dialog::FilePath::into_path)
        .transpose()
        .map_err(|_| UnsupportedFileLocation)
}
