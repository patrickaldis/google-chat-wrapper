// Injected on every page load to:
//
// 1. Bridge browser notifications to native OS notifications via Tauri's
//    notification plugin.
// 2. Detect unread messages and forward the count to the Rust backend so the
//    tray icon badge can be updated.
//
// Google Chat does NOT put an unread count in document.title. Instead it:
//   - Swaps the favicon URL to a "notif" variant when there are unreads.
//   - Shows per-conversation unread badges in the sidebar DOM.
//
// We poll both signals every second and send the result to Rust via IPC.

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

    // Read the total unread count from the sidebar DOM.
    //
    // Google Chat renders per-conversation unread badges as:
    //   <span class="SaMfhe ...">N</span>
    // inside each conversation list item. We sum all of these.
    //
    // Conversations scrolled out of view are summarised in a separate element:
    //   <div class="i5r4Nb">N more unread</div>
    // We parse the number from that text and add it to the total.
    //
    // These class names are obfuscated and may change when Google updates Chat.
    function getUnreadCountFromDOM() {
        var count = 0;

        // Per-conversation badges
        var badges = document.body.querySelectorAll('span.SaMfhe');
        badges.forEach(function(span) {
            var n = parseInt(span.textContent, 10);
            if (n > 0) count += n;
        });

        // "N more unread" summary for off-screen conversations
        var moreUnread = document.body.querySelector('div.i5r4Nb');
        if (moreUnread) {
            var match = moreUnread.textContent.match(/(\d+)/);
            if (match) count += parseInt(match[1], 10);
        }

        return count;
    }

    // Check the favicon for a binary "has unreads" signal. Google Chat swaps
    // the favicon href to a URL containing "notif" when there are unread
    // messages.
    function faviconIndicatesUnread() {
        var link = document.querySelector(
            'link[rel="shortcut icon"],' +
            'link[rel="icon"]'
        );
        if (!link || !link.href) return false;
        return /notif/.test(link.href);
    }

    var lastCount = -1;

    function pollUnreadCount() {
        if (!window.__TAURI__ || !window.__TAURI__.core) return;

        // Use the numeric DOM count. The favicon is only a binary signal (unreads
        // exist or not) so it can't provide an actual number -- don't show a
        // misleading count when the DOM badges aren't available yet.
        var count = getUnreadCountFromDOM();

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
