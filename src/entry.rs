//! Per-key counter entry (one Fixed Window "window").

use std::time::{SystemTime, UNIX_EPOCH};

/// State of a key within the current fixed window.
///
/// `count`        -> how many requests were counted in the window so far.
/// `window_start` -> Unix timestamp (milliseconds) when the current window started.
pub struct RateLimitEntry {
    pub count: u64,
    pub window_start: u64,
}

impl RateLimitEntry {
    /// Creates a new entry, starting the window at `now`.
    pub fn new(now: u64) -> Self {
        Self {
            count: 0,
            window_start: now,
        }
    }
}

/// Current time in milliseconds since the Unix epoch.
///
/// If the clock is invalid (before the epoch), returns 0 instead of panicking —
/// keeping the limiter working is better than taking the process down.
pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
