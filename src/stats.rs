//! Statistics counters and the return types exposed to JS.

use napi_derive::napi;
use std::sync::atomic::{AtomicU64, Ordering};

/// Accumulated (thread-safe) counters of the limiter.
///
/// `Relaxed` atomics: we only need correct totals, with no ordering between
/// the two variables.
#[derive(Default)]
pub struct Counters {
    allowed: AtomicU64,
    blocked: AtomicU64,
}

impl Counters {
    /// Records the result of one check.
    pub fn record(&self, allowed: bool) {
        if allowed {
            self.allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.blocked.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn allowed(&self) -> u64 {
        self.allowed.load(Ordering::Relaxed)
    }

    pub fn blocked(&self) -> u64 {
        self.blocked.load(Ordering::Relaxed)
    }
}

/// Return value of `check()`.
///
/// `#[napi(object)]` generates a plain JS object already in `camelCase`. `i64`
/// becomes `number` in JavaScript (avoiding `BigInt`).
#[napi(object)]
pub struct CheckResult {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Limit configured for the window.
    pub limit: i64,
    /// Requests left in the current window.
    pub remaining: i64,
    /// Whole seconds until the client may retry, rounded up so a blocked client
    /// never sees 0 (0 when allowed). Fits the HTTP `Retry-After` header.
    pub retry_after: i64,
    /// Whole seconds until the current window resets, rounded up.
    pub reset_after: i64,
    /// Milliseconds until the client may retry (0 when allowed).
    pub retry_after_ms: i64,
    /// Milliseconds until the current window resets.
    pub reset_after_ms: i64,
}

/// Return value of `stats()`.
#[napi(object)]
pub struct RateLimitStats {
    /// Total requests allowed since the limiter was created.
    pub allowed: i64,
    /// Total requests blocked since the limiter was created.
    pub blocked: i64,
    /// Total checks (allowed + blocked).
    pub checks: i64,
    /// Number of keys currently tracked.
    pub active_keys: i64,
}
