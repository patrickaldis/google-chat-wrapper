// Injected on every page load to:
//
// 1. Bridge browser notifications to native OS notifications via Tauri's
//    notification plugin.
// 2. Detect unread messages and forward the count to the Rust backend so the
//    tray icon badge can be updated.
//
// Unread detection first checks the page title for a "Google Chat (N)"
// pattern. If --badge-attr is provided via CLI, it also walks role="listitem"
// elements for numeric badge spans (identified by the given attribute),
// skipping Spaces nav duplicates (identified by role="link" descendants).
//
// The badge span attribute is set via window.__BADGE_ATTR (injected by Rust
// from the --badge-attr CLI flag). When not provided, only the page title
// signal is used.
//
// We poll every second and send the result to Rust via IPC.

(function() {
    if (window.__notificationOverrideInstalled) return;
    window.__notificationOverrideInstalled = true;

    // --- Notification bridge ---------------------------------------------------

    var OriginalNotification = window.Notification;

    window.Notification = function(title, options) {
        if (window.__TAURI__ && window.__TAURI__.notification) {
            window.__TAURI__.notification.sendNotification({
                title: title,
                body: options?.body || '',
            });
        }
        return new OriginalNotification(title, options);
    };

    window.Notification.requestPermission = function() {
        return Promise.resolve('granted');
    };

    Object.defineProperty(window.Notification, 'permission', {
        get: function() { return 'granted'; }
    });

    // --- Unread count detection ------------------------------------------------

    // The HTML attribute used to locate badge spans, if provided via
    // --badge-attr CLI flag.  When absent, only the page title is used.
    var BADGE_ATTR = window.__BADGE_ATTR || null;

    function getUnreadCount() {
        // Try the page title first — most stable signal.
        var titleMatch = document.title.match(/\((\d+)\)/);
        if (titleMatch) return parseInt(titleMatch[1], 10);

        // If no badge attribute was configured, we can't scrape the DOM.
        if (!BADGE_ATTR) return 0;

        // Walk all role="listitem" elements looking for numeric badge spans.
        var count = 0;
        var listItems = document.body.querySelectorAll('[role="listitem"]');

        listItems.forEach(function(item) {
            // Skip Spaces nav entries — they contain a role="link" wrapper
            // and duplicate the conversation list badges.
            if (item.querySelector('[role="link"]')) return;

            // Look for spans with the configured attribute that contain a
            // bare integer (the unread badge).
            var spans = item.querySelectorAll('span[' + BADGE_ATTR + '="true"]');
            for (var i = 0; i < spans.length; i++) {
                var text = spans[i].textContent.trim();
                if (/^\d{1,3}$/.test(text)) {
                    count += parseInt(text, 10);
                    break; // one badge per listitem
                }
            }
        });

        return count;
    }

    var lastCount = -1;

    function pollUnreadCount() {
        if (!window.__TAURI__ || !window.__TAURI__.core) return;

        var count = getUnreadCount();

        // Only invoke the Rust command when the count actually changes.
        if (count !== lastCount) {
            lastCount = count;
            window.__TAURI__.core.invoke('update_unread_count', { count: count });
        }
    }

    // Poll every second. MutationObserver doesn't work reliably here because
    // Google Chat replaces the favicon element entirely rather than mutating it.
    setInterval(pollUnreadCount, 1000);

    // Also run once immediately in case the page already has unreads.
    pollUnreadCount();
})();
