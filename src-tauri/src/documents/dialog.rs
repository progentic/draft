use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

use super::save_as::SaveAsFormat;

const DOCUMENT_FILTER_NAME: &str = "DRAFT document";
const DOCUMENT_EXTENSIONS: &[&str] = &["draft", "json"];
const OPEN_DOCUMENT_EXTENSIONS: &[&str] = &["draft", "json", "txt", "md", "docx"];
const TEXT_DOCUMENT_EXTENSIONS: &[&str] = &["txt", "md"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnsupportedFileLocation;

pub(crate) async fn select_open_document(
    app_handle: &AppHandle,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Open document")
        .add_filter("Supported documents", OPEN_DOCUMENT_EXTENSIONS)
        .add_filter(DOCUMENT_FILTER_NAME, DOCUMENT_EXTENSIONS)
        .add_filter("Text or Markdown", TEXT_DOCUMENT_EXTENSIONS)
        .add_filter("Word document", &["docx"]);
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
    suggested_file_name: &str,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Save DRAFT document")
        .set_file_name(suggested_file_name)
        .add_filter(DOCUMENT_FILTER_NAME, &["draft"]);
    if let Some(window) = app_handle.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let (sender, receiver) = oneshot::channel();
    dialog.save_file(move |selected| drop(sender.send(selected)));
    let selected = receiver.await.map_err(|_| UnsupportedFileLocation)?;
    selected_path(selected)
}

pub(crate) async fn select_save_as_output(
    app_handle: &AppHandle,
    format: SaveAsFormat,
    suggested_file_name: &str,
) -> Result<Option<PathBuf>, UnsupportedFileLocation> {
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title(format.dialog_title())
        .set_file_name(suggested_file_name)
        .add_filter(format.filter_name(), &[format.extension()]);
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
