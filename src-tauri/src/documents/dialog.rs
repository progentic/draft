use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

const DOCUMENT_FILTER_NAME: &str = "DRAFT document";
const DOCUMENT_EXTENSIONS: &[&str] = &["draft", "json"];
const DEFAULT_DOCUMENT_FILE_NAME: &str = "Untitled.draft";
const DEFAULT_DOCX_FILE_NAME: &str = "Untitled.docx";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnsupportedFileLocation;

pub(crate) async fn select_open_document(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Open DRAFT document")
        .add_filter(DOCUMENT_FILTER_NAME, DOCUMENT_EXTENSIONS);
    if let Some(window) = app_handle.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let (sender, receiver) = oneshot::channel();
    dialog.pick_file(move |selected| drop(sender.send(selected)));
    let selected = receiver.await.map_err(|_| UnsupportedFileLocation)?;
    selected_path(selected)
}

pub(crate) async fn select_save_document(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Save DRAFT document")
        .set_file_name(DEFAULT_DOCUMENT_FILE_NAME)
        .add_filter(DOCUMENT_FILTER_NAME, &["draft"]);
    if let Some(window) = app_handle.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let (sender, receiver) = oneshot::channel();
    dialog.save_file(move |selected| drop(sender.send(selected)));
    let selected = receiver.await.map_err(|_| UnsupportedFileLocation)?;
    selected_path(selected)
}

pub(crate) async fn select_export_docx(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Export DOCX document")
        .set_file_name(DEFAULT_DOCX_FILE_NAME)
        .add_filter("Word document", &["docx"]);
    if let Some(window) = app_handle.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let (sender, receiver) = oneshot::channel();
    dialog.save_file(move |selected| drop(sender.send(selected)));
    let selected = receiver.await.map_err(|_| UnsupportedFileLocation)?;
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
