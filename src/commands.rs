use tauri::{AppHandle, Manager};

use crate::badge;
use crate::tray::TrayState;

/// Called from JavaScript whenever the unread count changes (detected by
/// polling the favicon and sidebar DOM badges).
///
/// Regenerates the tray icon with a badge overlay and swaps it in. Skips the
/// (relatively expensive) redraw when the count hasn't actually changed.
#[tauri::command]
pub fn update_unread_count(app: AppHandle, count: u32) {
    let state = app.state::<TrayState>();

    // Skip if the count hasn't changed.
    {
        let mut last = state.last_count.lock().unwrap();
        if *last == count {
            return;
        }
        *last = count;
    }

    let icon = badge::render(count);

    if let Some(tray) = app.tray_by_id(&state.icon_id) {
        if let Err(e) = tray.set_icon(Some(icon)) {
            eprintln!("failed to update tray icon: {e}");
        }
    }
}
