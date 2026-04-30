use std::sync::Mutex;

use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent, TrayIconId};

use crate::badge;

/// Holds the tray icon ID so the `update_unread_count` command can look it up.
pub struct TrayState {
    pub icon_id: TrayIconId,
    /// The last badge count we rendered, used to avoid redundant redraws.
    pub last_count: Mutex<u32>,
}

/// Bring the main window to the foreground.
///
/// Silently does nothing if the window has already been dropped (shouldn't
/// happen in normal operation, but avoids a panic).
fn show_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(e) = window.show() {
        eprintln!("failed to show window: {e}");
    }
    if let Err(e) = window.unminimize() {
        eprintln!("failed to unminimize window: {e}");
    }
    if let Err(e) = window.set_focus() {
        eprintln!("failed to focus window: {e}");
    }
}

/// Set up the system-tray icon with a Show/Quit context menu.
///
/// - Left-clicking the tray icon brings the window to the foreground.
/// - The "Show" menu item does the same.
/// - The "Quit" menu item exits the application.
///
/// The tray icon ID is stored in managed state so `update_unread_count` can
/// update the icon at runtime.
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    let icon = badge::render(0);

    let tray = TrayIconBuilder::new()
        .tooltip("Google Chat")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayState {
        icon_id: tray.id().clone(),
        last_count: Mutex::new(0),
    });

    Ok(())
}
