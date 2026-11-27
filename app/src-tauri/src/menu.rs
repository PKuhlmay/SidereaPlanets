//! The native menu bar of the Mac app. The window stays the one place where actions live: a menu item only sends its id to
//! the frontend (`menu-action` event, ids in `app/src/logic/commands.ts`), and the frontend reports back which items are
//! enabled (`update_menu`). Windows and Linux keep the window without a menu bar; nothing here runs there.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::menu::{
    AboutMetadata, Menu, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu, SubmenuBuilder,
};
use tauri::{App, AppHandle, Emitter, Manager, Wry};

/// The step ids and their names, in the order of the step bar (`viewSteps` in `app/src/logic/workflow.ts`).
const STEPS: [(&str, &str); 7] = [
    ("open", "Open"),
    ("alignpoints", "Alignpoints"),
    ("align", "Align"),
    ("limit", "Limit"),
    ("stack", "Stack"),
    ("wavelets", "Wavelets"),
    ("save", "Save"),
];

/// The items the frontend switches on and off, found by their id, and the open-recent list as it is shown now.
pub struct NativeMenu {
    items: HashMap<String, MenuItem<Wry>>,
    recent_menu: Submenu<Wry>,
    recent: Mutex<Vec<String>>,
}

/// What the frontend tells the menu: which ids are enabled, the name of the pause item and the recordings opened last.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuUpdate {
    enabled: HashMap<String, bool>,
    paused: bool,
    recent: Vec<String>,
}

/// Builds the menu bar and starts forwarding its clicks to the frontend. Called on macOS only.
pub fn install(app: &App) -> tauri::Result<()> {
    let handle = app.handle();
    let mut items: HashMap<String, MenuItem<Wry>> = HashMap::new();
    let mut item =
        |id: &str, text: &str, accelerator: Option<&str>| -> tauri::Result<MenuItem<Wry>> {
            let made = MenuItem::with_id(handle, id, text, true, accelerator)?;
            items.insert(id.to_owned(), made.clone());
            Ok(made)
        };

    let about = PredefinedMenuItem::about(
        handle,
        Some("About Siderea Planets"),
        Some(AboutMetadata {
            name: Some("Siderea Planets".into()),
            version: Some(handle.package_info().version.to_string()),
            copyright: Some(
                "Siderea Planets, stacking and sharpening for planetary astrophotography.".into(),
            ),
            ..Default::default()
        }),
    )?;
    let app_menu = SubmenuBuilder::new(handle, "Siderea Planets")
        .item(&about)
        .separator()
        .item(&item("settings", "Settings…", Some("Cmd+,"))?)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let recent_menu = SubmenuBuilder::new(handle, "Open Recent").build()?;
    let file_menu = SubmenuBuilder::new(handle, "File")
        .item(&item(
            "selectRecording",
            "Select Recording…",
            Some("Cmd+O"),
        )?)
        .item(&recent_menu)
        .item(&item("openProject", "Open Project…", None)?)
        .separator()
        .item(&item("saveImage", "Save Image…", Some("Cmd+S"))?)
        .item(&item("saveProject", "Save Project…", None)?)
        .separator()
        .close_window()
        .build()?;

    let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(handle, "View")
        .item(&item("zoomIn", "Zoom In", Some("Cmd+="))?)
        .item(&item("zoomOut", "Zoom Out", Some("Cmd+-"))?)
        .item(&item("zoomFit", "Zoom to Fit", Some("Cmd+0"))?)
        .item(&item("zoomActual", "Actual Size", Some("Cmd+1"))?)
        .separator()
        .fullscreen()
        .build()?;

    let mut steps = SubmenuBuilder::new(handle, "Go to Step");
    for (number, (id, name)) in STEPS.iter().enumerate() {
        let accelerator = format!("Cmd+Alt+{}", number + 1);
        let entry = item(&format!("step:{id}"), name, Some(&accelerator))?;
        steps = steps.item(&entry);
    }
    let run_menu = SubmenuBuilder::new(handle, "Run")
        .item(&item("cancel", "Cancel", Some("Cmd+."))?)
        .item(&item("togglePause", "Pause", None)?)
        .separator()
        .item(&steps.build()?)
        .build()?;

    let window_menu = SubmenuBuilder::new(handle, "Window")
        .minimize()
        .maximize()
        .build()?;

    let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(&item("help", "Siderea Planets Help", Some("Cmd+Shift+/"))?)
        .build()?;

    let menu = Menu::with_items(
        handle,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &run_menu,
            &window_menu,
            &help_menu,
        ],
    )?;
    app.set_menu(menu)?;
    // macOS adds its window list and the help search to these two.
    #[cfg(target_os = "macos")]
    {
        window_menu.set_as_windows_menu_for_nsapp()?;
        help_menu.set_as_help_menu_for_nsapp()?;
    }

    app.on_menu_event(|app, event| {
        let _ = app.emit("menu-action", event.id().as_ref());
    });
    app.manage(NativeMenu {
        items,
        recent_menu,
        recent: Mutex::new(Vec::new()),
    });
    Ok(())
}

/// Enables and disables the items, names the pause item and rebuilds the Open Recent list when it changed. Does nothing
/// where there is no menu bar.
#[tauri::command]
pub fn update_menu(app: AppHandle, update: MenuUpdate) -> Result<(), String> {
    let Some(menu) = app.try_state::<NativeMenu>() else {
        return Ok(());
    };
    for (id, enabled) in &update.enabled {
        if let Some(item) = menu.items.get(id) {
            item.set_enabled(*enabled).map_err(|e| e.to_string())?;
        }
    }
    if let Some(pause) = menu.items.get("togglePause") {
        let text = if update.paused { "Resume" } else { "Pause" };
        pause.set_text(text).map_err(|e| e.to_string())?;
    }
    let mut shown = menu.recent.lock().map_err(|e| e.to_string())?;
    if *shown != update.recent {
        rebuild_recent(&app, &menu.recent_menu, &update.recent).map_err(|e| e.to_string())?;
        *shown = update.recent;
    }
    Ok(())
}

/// Replaces the entries of Open Recent: one per recording (`recent:<index>`), or a dimmed note when there is none.
fn rebuild_recent(app: &AppHandle, submenu: &Submenu<Wry>, labels: &[String]) -> tauri::Result<()> {
    for entry in submenu.items()? {
        match entry {
            MenuItemKind::MenuItem(item) => submenu.remove(&item)?,
            MenuItemKind::Submenu(item) => submenu.remove(&item)?,
            MenuItemKind::Predefined(item) => submenu.remove(&item)?,
            MenuItemKind::Check(item) => submenu.remove(&item)?,
            MenuItemKind::Icon(item) => submenu.remove(&item)?,
        }
    }
    if labels.is_empty() {
        submenu.append(&MenuItem::new(
            app,
            "No Recent Recordings",
            false,
            None::<&str>,
        )?)?;
    }
    for (index, label) in labels.iter().enumerate() {
        let id = format!("recent:{index}");
        submenu.append(&MenuItem::with_id(app, id, label, true, None::<&str>)?)?;
    }
    Ok(())
}
