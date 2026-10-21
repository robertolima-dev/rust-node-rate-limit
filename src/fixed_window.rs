//! The Fixed Window algorithm.
//!
//! Each key has a window of `window` time units (milliseconds in `RateLimiter`;
//! the algorithm only needs `window` and `now` in the same unit). Up to `limit` requests are allowed
//! inside the window; when it expires, the window "rolls over" and the count
//! starts again from zero. Simple, predictable and cheap — ideal for a native core.

use crate::entry::RateLimitEntry;

/// Result of applying one request (hit) to an entry.
pub struct Decision {
    /// Whether the request is allowed in this window.
    pub allowed: bool,
    /// Requests left in the current window (0 when blocked).
    pub remaining: u64,
    /// Time until the client may retry, in the unit of `window` (0 when allowed).
    pub retry_after: u64,
    /// Time until the current window resets, in the unit of `window`.
    pub reset_after: u64,
}

/// Applies one request to the entry under the Fixed Window algorithm.
///
/// Takes `&mut RateLimitEntry` because it may restart the window and increment
/// the count in place. The caller holds the DashMap shard lock during the call,
/// which makes the read-modify-write atomic per key.
pub fn hit(entry: &mut RateLimitEntry, limit: u64, window: u64, now: u64) -> Decision {
    // Window expired? Reset the count and record the new start.
    if now.saturating_sub(entry.window_start) >= window {
        entry.window_start = now;
        entry.count = 0;
    }

    let elapsed = now.saturating_sub(entry.window_start);
    let reset_after = window.saturating_sub(elapsed);

    if entry.count < limit {
        entry.count += 1;
        Decision {
            allowed: true,
            remaining: limit - entry.count,
            retry_after: 0,
            reset_after,
        }
    } else {
        Decision {
            allowed: false,
            remaining: 0,
            retry_after: reset_after,
            reset_after,
        }
    }
}
