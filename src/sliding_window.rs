//! The Sliding Window (counter) algorithm.
//!
//! Fixed Window has a known weakness: a client can send `limit` requests at the
//! end of one window and `limit` at the start of the next, getting `2 * limit`
//! through in less than a single window. The **sliding window counter** smooths
//! that boundary by keeping the count of the *current* (aligned) window plus the
//! *previous* one, and weighting the previous one by the fraction that still
//! overlaps the moving window that ends at `now`:
//!
//! ```text
//! estimate = previous * weight + current
//! weight   = (window - elapsed_in_current_window) / window
//! ```
//!
//! It is an approximation (it assumes requests were spread evenly over the
//! previous window), but it is O(1) in time and memory per key — unlike a sliding
//! *log*, which keeps every timestamp — and it removes the "double" at the boundary.

use crate::fixed_window::Decision;

/// Per-key state of the sliding window counter.
///
/// `window_index` is the number of the aligned window (`now / window`) that
/// `current` counts; `previous` is the count of `window_index - 1`.
pub struct SlidingEntry {
    pub window_index: u64,
    pub current: u64,
    pub previous: u64,
}

impl SlidingEntry {
    /// Creates a new entry already positioned at window `window_index`.
    pub fn new(window_index: u64) -> Self {
        Self {
            window_index,
            current: 0,
            previous: 0,
        }
    }

    /// Rolls `current`/`previous` to describe the window that contains `idx`. A
    /// one-step advance pushes `current` into `previous`; a larger jump makes both
    /// stale and resets everything.
    fn advance_to(&mut self, idx: u64) {
        if idx == self.window_index {
            // same window — nothing to do
        } else if idx == self.window_index + 1 {
            self.previous = self.current;
            self.current = 0;
            self.window_index = idx;
        } else {
            self.previous = 0;
            self.current = 0;
            self.window_index = idx;
        }
    }
}

/// Weight applied to the previous window: the fraction of `window` still ahead
/// of `now` within the current window.
fn weight(window: u64, elapsed: u64) -> f64 {
    window.saturating_sub(elapsed) as f64 / window as f64
}

/// Applies one request (hit) to the entry under the Sliding Window algorithm.
///
/// Takes `&mut SlidingEntry` because it may roll the window and increment the
/// count in place. The caller holds the DashMap shard lock during the call,
/// which makes the read-modify-write atomic per key.
pub fn hit(entry: &mut SlidingEntry, limit: u64, window: u64, now: u64) -> Decision {
    let idx = now / window;
    entry.advance_to(idx);

    let elapsed = now - idx * window;
    let w = weight(window, elapsed);
    let estimated = entry.previous as f64 * w + entry.current as f64;
    let reset_after = window.saturating_sub(elapsed);

    let allowed = estimated < limit as f64;
    if allowed {
        entry.current += 1;
    }

    let estimated_after = entry.previous as f64 * w + entry.current as f64;
    let remaining = (limit as f64 - estimated_after).max(0.0).floor() as u64;

    Decision {
        allowed,
        remaining,
        retry_after: if allowed { 0 } else { reset_after },
        reset_after,
    }
}

/// Requests left for the entry at `now`, without consuming one (read-only).
pub fn remaining(entry: &SlidingEntry, limit: u64, window: u64, now: u64) -> u64 {
    let idx = now / window;
    // Effective count (rolled forward) without mutating the entry.
    let (previous, current) = if idx == entry.window_index {
        (entry.previous, entry.current)
    } else if idx == entry.window_index + 1 {
        (entry.current, 0)
    } else {
        (0, 0)
    };
    let elapsed = now - idx * window;
    let estimated = previous as f64 * weight(window, elapsed) + current as f64;
    (limit as f64 - estimated).max(0.0).floor() as u64
}

/// `true` when the entry no longer counts (not seen for at least two windows:
/// neither `current` nor `previous` overlaps the moving window).
pub fn is_stale(entry: &SlidingEntry, window: u64, now: u64) -> bool {
    let idx = now / window;
    idx >= entry.window_index.saturating_add(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_up_to_limit_within_one_window() {
        let mut e = SlidingEntry::new(0);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(!hit(&mut e, 3, 60, 0).allowed);
    }

    #[test]
    fn smooths_the_boundary_burst() {
        // limit=10/60s. Fill the first window at t=30, then cross into the next one
        // halfway (t=90): the previous one weighs 0.5, so only 5 are allowed.
        let mut e = SlidingEntry::new(0);
        for _ in 0..10 {
            assert!(hit(&mut e, 10, 60, 30).allowed);
        }
        assert_eq!(remaining(&e, 10, 60, 90), 5);
        let mut admitted = 0;
        for _ in 0..10 {
            if hit(&mut e, 10, 60, 90).allowed {
                admitted += 1;
            }
        }
        assert_eq!(
            admitted, 5,
            "a burst at the boundary must be capped at 5, not 10"
        );
    }

    #[test]
    fn fully_resets_after_two_windows() {
        let mut e = SlidingEntry::new(0);
        assert!(hit(&mut e, 2, 10, 0).allowed);
        assert!(hit(&mut e, 2, 10, 0).allowed);
        assert!(!hit(&mut e, 2, 10, 0).allowed);
        assert_eq!(remaining(&e, 2, 10, 20), 2);
        assert!(hit(&mut e, 2, 10, 20).allowed);
    }

    #[test]
    fn staleness_after_two_windows() {
        let mut e = SlidingEntry::new(0);
        hit(&mut e, 1, 10, 0);
        assert!(!is_stale(&e, 10, 10)); // idx 1: the previous window still overlaps
        assert!(is_stale(&e, 10, 20)); // idx 2: stale
    }
}
