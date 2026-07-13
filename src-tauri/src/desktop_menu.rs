use tauri::{
    App, AppHandle, Manager, Runtime, Wry,
    menu::{Menu, MenuBuilder, MenuEvent, MenuItem, MenuItemBuilder, SubmenuBuilder},
};

use crate::events::native_menu::{NativeMenuEvent, emit_native_menu_action};

#[derive(Clone, Copy)]
struct FileMenuSpec {
    action: NativeMenuEvent,
    label: &'static str,
    accelerator: Option<&'static str>,
    starts_group: bool,
}

const FILE_MENU_SPECS: [FileMenuSpec; 7] = [
    file_item(
        NativeMenuEvent::NewDocument,
        "New Document",
        "CmdOrCtrl+N",
        false,
    ),
    file_item(NativeMenuEvent::OpenDocument, "Open…", "CmdOrCtrl+O", false),
    file_item(
        NativeMenuEvent::CloseDocument,
        "Close",
        "CmdOrCtrl+W",
        false,
    ),
    file_item(NativeMenuEvent::SaveDocument, "Save", "CmdOrCtrl+S", true),
    file_item(
        NativeMenuEvent::SaveDocumentAs,
        "Save As…",
        "CmdOrCtrl+Shift+S",
        false,
    ),
    file_item_without_shortcut(
        NativeMenuEvent::SaveBackToSource,
        "Save Back to Source",
        false,
    ),
    file_item(
        NativeMenuEvent::ExportDocx,
        "Export DOCX…",
        "CmdOrCtrl+Shift+E",
        true,
    ),
];

const fn file_item(
    action: NativeMenuEvent,
    label: &'static str,
    accelerator: &'static str,
    starts_group: bool,
) -> FileMenuSpec {
    FileMenuSpec {
        action,
        label,
        accelerator: Some(accelerator),
        starts_group,
    }
}

const fn file_item_without_shortcut(
    action: NativeMenuEvent,
    label: &'static str,
    starts_group: bool,
) -> FileMenuSpec {
    FileMenuSpec {
        action,
        label,
        accelerator: None,
        starts_group,
    }
}

pub(crate) struct NativeMenuItems {
    entries: Vec<(NativeMenuEvent, MenuItem<Wry>)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeMenuAvailability {
    pub(crate) can_new: bool,
    pub(crate) can_open: bool,
    pub(crate) can_close: bool,
    pub(crate) can_save: bool,
    pub(crate) can_save_as: bool,
    pub(crate) can_save_back: bool,
    pub(crate) can_export: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeMenuUpdateError;

pub(crate) fn install(app: &mut App) -> tauri::Result<()> {
    let items = NativeMenuItems::build(app.handle())?;
    let menu = build_application_menu(app.handle(), &items)?;
    app.set_menu(menu)?;
    app.manage(items);
    Ok(())
}

pub(crate) fn handle_event(app: &AppHandle, event: MenuEvent) {
    let Some(action) = action_for_id(event.id().as_ref()) else {
        return;
    };
    emit_native_menu_action(app, action);
}

impl NativeMenuItems {
    fn build<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<Self>
    where
        MenuItem<R>: Into<MenuItem<Wry>>,
    {
        let mut entries = Vec::with_capacity(FILE_MENU_SPECS.len());
        for spec in FILE_MENU_SPECS {
            let mut builder = MenuItemBuilder::with_id(action_id(spec.action), spec.label);
            builder = builder.enabled(initially_enabled(spec.action));
            if let Some(accelerator) = spec.accelerator {
                builder = builder.accelerator(accelerator);
            }
            entries.push((spec.action, builder.build(manager)?.into()));
        }
        Ok(Self { entries })
    }

    fn item(&self, action: NativeMenuEvent) -> &MenuItem<Wry> {
        self.entries
            .iter()
            .find_map(|(candidate, item)| (*candidate == action).then_some(item))
            .expect("every native action has one menu item")
    }

    pub(crate) fn apply(
        &self,
        availability: NativeMenuAvailability,
    ) -> Result<(), NativeMenuUpdateError> {
        for (action, item) in &self.entries {
            item.set_enabled(availability.enabled(*action))
                .map_err(|_| NativeMenuUpdateError)?;
        }
        Ok(())
    }
}

impl NativeMenuAvailability {
    fn enabled(self, action: NativeMenuEvent) -> bool {
        match action {
            NativeMenuEvent::NewDocument => self.can_new,
            NativeMenuEvent::OpenDocument => self.can_open,
            NativeMenuEvent::CloseDocument => self.can_close,
            NativeMenuEvent::SaveDocument => self.can_save,
            NativeMenuEvent::SaveDocumentAs => self.can_save_as,
            NativeMenuEvent::SaveBackToSource => self.can_save_back,
            NativeMenuEvent::ExportDocx => self.can_export,
        }
    }
}

fn build_application_menu(app: &AppHandle, items: &NativeMenuItems) -> tauri::Result<Menu<Wry>> {
    let file = build_file_menu(app, items)?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .fullscreen()
        .build()?;
    #[cfg(target_os = "macos")]
    let builder = {
        let application = SubmenuBuilder::new(app, "DRAFT")
            .about(None)
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?;
        MenuBuilder::new(app).item(&application)
    };
    #[cfg(not(target_os = "macos"))]
    let builder = MenuBuilder::new(app);
    builder.item(&file).item(&edit).item(&window).build()
}

fn build_file_menu(
    app: &AppHandle,
    items: &NativeMenuItems,
) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let mut builder = SubmenuBuilder::new(app, "File");
    for spec in FILE_MENU_SPECS {
        if spec.starts_group {
            builder = builder.separator();
        }
        builder = builder.item(items.item(spec.action));
    }
    builder.build()
}

fn action_id(action: NativeMenuEvent) -> &'static str {
    match action {
        NativeMenuEvent::NewDocument => "file.new_document",
        NativeMenuEvent::OpenDocument => "file.open_document",
        NativeMenuEvent::CloseDocument => "file.close_document",
        NativeMenuEvent::SaveDocument => "file.save_document",
        NativeMenuEvent::SaveDocumentAs => "file.save_document_as",
        NativeMenuEvent::SaveBackToSource => "file.save_back_to_source",
        NativeMenuEvent::ExportDocx => "file.export_docx",
    }
}

fn action_for_id(id: &str) -> Option<NativeMenuEvent> {
    FILE_MENU_SPECS
        .iter()
        .find_map(|spec| (action_id(spec.action) == id).then_some(spec.action))
}

fn initially_enabled(action: NativeMenuEvent) -> bool {
    matches!(
        action,
        NativeMenuEvent::NewDocument | NativeMenuEvent::OpenDocument
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_menu_contract_has_stable_order_labels_and_shortcuts() {
        let actual = FILE_MENU_SPECS.map(|spec| {
            (
                action_id(spec.action),
                spec.label,
                spec.accelerator,
                spec.starts_group,
            )
        });

        assert_eq!(
            actual,
            [
                (
                    "file.new_document",
                    "New Document",
                    Some("CmdOrCtrl+N"),
                    false
                ),
                ("file.open_document", "Open…", Some("CmdOrCtrl+O"), false),
                ("file.close_document", "Close", Some("CmdOrCtrl+W"), false),
                ("file.save_document", "Save", Some("CmdOrCtrl+S"), true),
                (
                    "file.save_document_as",
                    "Save As…",
                    Some("CmdOrCtrl+Shift+S"),
                    false
                ),
                (
                    "file.save_back_to_source",
                    "Save Back to Source",
                    None,
                    false
                ),
                (
                    "file.export_docx",
                    "Export DOCX…",
                    Some("CmdOrCtrl+Shift+E"),
                    true
                ),
            ]
        );
    }

    #[test]
    fn every_menu_id_maps_to_one_action() {
        for spec in FILE_MENU_SPECS {
            assert_eq!(action_for_id(action_id(spec.action)), Some(spec.action));
        }
        assert_eq!(action_for_id("file.unknown"), None);
    }

    #[test]
    fn initial_state_allows_only_document_creation_or_opening() {
        for spec in FILE_MENU_SPECS {
            assert_eq!(
                initially_enabled(spec.action),
                matches!(
                    spec.action,
                    NativeMenuEvent::NewDocument | NativeMenuEvent::OpenDocument
                )
            );
        }
    }

    #[test]
    fn availability_is_action_specific() {
        let request = NativeMenuAvailability {
            can_new: true,
            can_open: false,
            can_close: true,
            can_save: false,
            can_save_as: true,
            can_save_back: false,
            can_export: false,
        };

        assert!(request.enabled(NativeMenuEvent::NewDocument));
        assert!(!request.enabled(NativeMenuEvent::OpenDocument));
        assert!(request.enabled(NativeMenuEvent::CloseDocument));
        assert!(!request.enabled(NativeMenuEvent::SaveDocument));
        assert!(request.enabled(NativeMenuEvent::SaveDocumentAs));
        assert!(!request.enabled(NativeMenuEvent::SaveBackToSource));
        assert!(!request.enabled(NativeMenuEvent::ExportDocx));
    }
}
