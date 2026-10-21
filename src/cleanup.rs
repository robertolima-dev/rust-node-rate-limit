//! Cleanup of expired keys.
//!
//! Fixed Window does not remove entries automatically: a key used once stays
//! in the map. `cleanup()` lets the Node process reclaim memory from keys whose
//! windows have already expired (useful with many short-lived keys, such as
//! IPs).

use crate::entry::RateLimitEntry;
use dashmap::DashMap;

/// Removes every entry whose window has expired at `now`.
/// Returns how many keys were removed.
pub fn remove_expired(entries: &DashMap<String, RateLimitEntry>, window: u64, now: u64) -> usize {
    let before = entries.len();
    // `retain` keeps only the entries still inside their window.
    entries.retain(|_, entry| now.saturating_sub(entry.window_start) < window);
    before - entries.len()
}
