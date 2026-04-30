#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod badge;
mod commands;
mod tray;
mod window;

use tauri::WindowEvent;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::update_unread_count])
        .setup(|app| {
            window::create(app)?;
            tray::setup(app)?;
            Ok(())
        })
        // Hide to tray instead of quitting when the window is closed.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    eprintln!("failed to hide window: {e}");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start application");
}
