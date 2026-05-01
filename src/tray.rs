use std::sync::mpsc;
use std::sync::Mutex;

use ksni::blocking::{Handle, TrayMethods};
use tauri::Manager;

use crate::badge;

/// Actions sent from the ksni tray thread to the Tauri main thread.
enum TrayAction {
    Toggle,
    Quit,
}

/// The ksni tray implementation.
pub struct ChatTray {
    tx: mpsc::Sender<TrayAction>,
    pub icon: ksni::Icon,
}

impl ksni::Tray for ChatTray {
    fn id(&self) -> String {
        "google-chat-wrapper".into()
    }

    fn title(&self) -> String {
        "Google Chat".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.icon.clone()]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayAction::Toggle);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Show/Hide".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayAction::Toggle);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Holds the ksni handle so the `update_unread_count` command can update the
/// tray icon at runtime.
pub struct TrayState {
    pub handle: Handle<ChatTray>,
    /// The last badge count we rendered, used to avoid redundant redraws.
    pub last_count: Mutex<u32>,
}

/// Bring the main window to the foreground.
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

/// Toggle the main window: show it if hidden/minimized, hide it if visible.
fn toggle_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    if visible {
        if let Err(e) = window.hide() {
            eprintln!("failed to hide window: {e}");
        }
    } else {
        show_window(app);
    }
}

/// Set up the system-tray icon using ksni (D-Bus StatusNotifierItem).
///
/// - Left-clicking the tray icon toggles window visibility.
/// - Right-clicking shows a context menu with Show/Hide and Quit.
///
/// The ksni handle is stored in managed state so `update_unread_count` can
/// update the icon at runtime.
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    let initial_icon = badge::render(0).to_ksni_icon();

    let tray = ChatTray {
        tx,
        icon: initial_icon,
    };

    let handle = tray
        .spawn()
        .map_err(|e| format!("failed to spawn ksni tray: {e}"))?;

    app.manage(TrayState {
        handle: handle.clone(),
        last_count: Mutex::new(0),
    });

    // Spawn a thread to relay ksni actions to the Tauri app handle.
    let app_handle = app.handle().clone();
    std::thread::spawn(move || {
        while let Ok(action) = rx.recv() {
            match action {
                TrayAction::Toggle => toggle_window(&app_handle),
                TrayAction::Quit => {
                    app_handle.exit(0);
                    break;
                }
            }
        }
    });

    Ok(())
}
