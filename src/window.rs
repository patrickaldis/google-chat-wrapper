use tauri::webview::{PageLoadEvent, WebviewWindowBuilder};

/// The Google Chat URL loaded into the webview.
const GOOGLE_CHAT_URL: &str = "https://chat.google.com";

/// JavaScript injected on every page load to bridge browser notifications
/// to native OS notifications. See `src/notifications.js` for details.
const NOTIFICATION_SCRIPT: &str = include_str!("notifications.js");

/// Create the main webview window pointing at Google Chat.
///
/// `badge_attr` is an optional HTML attribute name used to locate unread-count
/// badge spans inside conversation list items.  When `None`, badge detection
/// via DOM scraping is disabled and only the page-title signal is used.
/// Configurable via the `--badge-attr` CLI flag.
///
/// When `background` is true the window is created hidden so the app starts
/// in the system tray only.  The user can reveal it via the tray icon.
///
/// The notification-bridging script is injected after every page load so it
/// survives SPA navigations and redirects within chat.google.com.
pub fn create(app: &tauri::App, badge_attr: Option<&str>, background: bool) -> Result<(), Box<dyn std::error::Error>> {
    let url = GOOGLE_CHAT_URL
        .parse()
        .expect("hardcoded Google Chat URL is invalid");

    // Optionally inject the badge attribute as a JS global before the
    // notification script.  When absent, JS will skip DOM-based detection.
    let config_js = match badge_attr {
        Some(attr) => format!("window.__BADGE_ATTR = {};\n", serde_json_escape(attr)),
        None => String::new(),
    };
    let combined_script = format!("{config_js}{NOTIFICATION_SCRIPT}");

    WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
        .title("Google Chat")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .decorations(false)
        .visible(!background)
        .zoom_hotkeys_enabled(true)
        .on_page_load(move |webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                if let Err(e) = webview.eval(&combined_script) {
                    eprintln!("failed to inject notification script: {e}");
                }
            }
        })
        .build()?;

    Ok(())
}

/// Minimally JSON-escape a string (wraps in double quotes, escapes \, ", and
/// control characters). Avoids pulling in serde_json just for this.
fn serde_json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
