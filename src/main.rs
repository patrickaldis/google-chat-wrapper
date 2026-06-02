#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod badge;
mod commands;
mod tray;
mod window;

use tauri::WindowEvent;

/// Parsed command-line arguments.
struct CliArgs {
    /// Optional HTML attribute name for DOM-based unread badge detection.
    badge_attr: Option<String>,
    /// When true, start with the window hidden (tray-only).
    background: bool,
}

/// Parse command-line arguments.
fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut badge_attr = None;
    let mut background = false;

    let mut i = 1; // skip the binary name
    while i < args.len() {
        match args[i].as_str() {
            "--badge-attr" => {
                if let Some(val) = args.get(i + 1) {
                    badge_attr = Some(val.clone());
                    i += 1;
                }
            }
            "--background" => {
                background = true;
            }
            _ => {}
        }
        i += 1;
    }

    CliArgs {
        badge_attr,
        background,
    }
}

fn main() {
    let cli = parse_args();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::update_unread_count, commands::send_notification])
        .setup(move |app| {
            window::create(app, cli.badge_attr.as_deref(), cli.background)?;
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
