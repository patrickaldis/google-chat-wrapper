use tauri::{AppHandle, Manager};

use crate::badge;
use crate::tray::TrayState;

/// Called from JavaScript whenever the unread count changes (detected by
/// polling the page title or sidebar DOM badges).
///
/// Regenerates the tray icon with a badge overlay and updates the ksni tray.
/// Skips the (relatively expensive) redraw when the count hasn't actually
/// changed.
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

    let ksni_icon = badge::render(count).to_ksni_icon();

    state.handle.update(move |tray| {
        tray.icon = ksni_icon;
    });
}
