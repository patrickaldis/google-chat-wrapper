use tauri::webview::{PageLoadEvent, WebviewWindowBuilder};

/// The Google Chat URL loaded into the webview.
const GOOGLE_CHAT_URL: &str = "https://chat.google.com";

/// JavaScript injected on every page load to bridge browser notifications
/// to native OS notifications. See `src/notifications.js` for details.
const NOTIFICATION_SCRIPT: &str = include_str!("notifications.js");

/// Create the main webview window pointing at Google Chat.
///
/// The notification-bridging script is injected after every page load so it
/// survives SPA navigations and redirects within chat.google.com.
pub fn create(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let url = GOOGLE_CHAT_URL
        .parse()
        .expect("hardcoded Google Chat URL is invalid");

    WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
        .title("Google Chat")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .decorations(false)
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                if let Err(e) = webview.eval(NOTIFICATION_SCRIPT) {
                    eprintln!("failed to inject notification script: {e}");
                }
            }
        })
        .build()?;

    Ok(())
}
