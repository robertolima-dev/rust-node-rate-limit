//! The `RateLimiter` class exposed to Node.

use dashmap::DashMap;
use napi_derive::napi;

use crate::cleanup;
use crate::entry::{now_seconds, RateLimitEntry};
use crate::errors::RateLimitError;
use crate::fixed_window;
use crate::sliding_window::{self, SlidingEntry};
use crate::stats::{CheckResult, Counters, RateLimitStats};

/// The selected algorithm backend. Each variant owns its own `DashMap` (with
/// the matching entry type); the `RateLimiter` methods dispatch to the active
/// one. Separate maps keep the Fixed Window path unchanged and the Sliding
/// Window path self-contained.
enum Backend {
    Fixed(DashMap<String, RateLimitEntry>),
    Sliding(DashMap<String, SlidingEntry>),
}

impl Backend {
    /// Core of `allow()`/`check()`: applies the active algorithm atomically per key.
    /// The `RefMut` holds the shard lock during the update, which makes the
    /// read-modify-write atomic for this key.
    fn apply(&self, key: String, limit: u64, window: u64, now: u64) -> fixed_window::Decision {
        match self {
            Backend::Fixed(entries) => {
                let mut entry = entries
                    .entry(key)
                    .or_insert_with(|| RateLimitEntry::new(now));
                fixed_window::hit(entry.value_mut(), limit, window, now)
            }
            Backend::Sliding(entries) => {
                let idx = now / window;
                let mut entry = entries.entry(key).or_insert_with(|| SlidingEntry::new(idx));
                sliding_window::hit(entry.value_mut(), limit, window, now)
            }
        }
    }

    /// Requests left for `key` without consuming one. A missing or expired key
    /// returns the full limit.
    fn remaining(&self, key: &str, limit: u64, window: u64, now: u64) -> u64 {
        match self {
            Backend::Fixed(entries) => match entries.get(key) {
                Some(entry) if now.saturating_sub(entry.window_start) < window => {
                    limit.saturating_sub(entry.count)
                }
                _ => limit,
            },
            Backend::Sliding(entries) => match entries.get(key) {
                Some(entry) => sliding_window::remaining(&entry, limit, window, now),
                None => limit,
            },
        }
    }

    fn reset(&self, key: &str) {
        match self {
            Backend::Fixed(entries) => {
                entries.remove(key);
            }
            Backend::Sliding(entries) => {
                entries.remove(key);
            }
        }
    }

    fn clear(&self) {
        match self {
            Backend::Fixed(entries) => entries.clear(),
            Backend::Sliding(entries) => entries.clear(),
        }
    }

    fn cleanup(&self, window: u64, now: u64) -> usize {
        match self {
            Backend::Fixed(entries) => cleanup::remove_expired(entries, window, now),
            Backend::Sliding(entries) => {
                let before = entries.len();
                entries.retain(|_, entry| !sliding_window::is_stale(entry, window, now));
                before - entries.len()
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Backend::Fixed(entries) => entries.len(),
            Backend::Sliding(entries) => entries.len(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Backend::Fixed(_) => "fixed",
            Backend::Sliding(_) => "sliding",
        }
    }
}

/// Constructor options: `new RateLimiter({ limit, windowSeconds, algorithm })`.
///
/// `#[napi(object)]` accepts a plain JS object. `window_seconds` becomes
/// `windowSeconds` in JS automatically. `algorithm` is optional and defaults to
/// `"fixed"`.
#[napi(object)]
pub struct RateLimiterOptions {
    /// Maximum number of requests allowed per window.
    pub limit: i64,
    /// Window length, in seconds.
    pub window_seconds: i64,
    /// Strategy: `"fixed"` (default) or `"sliding"` (sliding window counter,
    /// which smooths bursts at the window boundary).
    pub algorithm: Option<String>,
}

/// Local (in-process) rate limiter with Fixed Window and Sliding Window algorithms.
///
/// Methods take `&self` (not `&mut self`): mutability is internal, through
/// `DashMap` (per key) and atomic counters. That keeps the limiter safe and
/// efficient under concurrency, without a global lock.
#[napi]
pub struct RateLimiter {
    limit: u64,
    window_seconds: u64,
    backend: Backend,
    counters: Counters,
}

#[napi]
impl RateLimiter {
    /// Creates a limiter. Throws (a JS exception) if the configuration is invalid.
    #[napi(constructor)]
    pub fn new(options: RateLimiterOptions) -> napi::Result<Self> {
        if options.limit <= 0 {
            return Err(
                RateLimitError::InvalidConfig("limit must be greater than 0".into()).into(),
            );
        }
        if options.window_seconds <= 0 {
            return Err(RateLimitError::InvalidConfig(
                "windowSeconds must be greater than 0".into(),
            )
            .into());
        }

        let backend = match options.algorithm.as_deref().unwrap_or("fixed") {
            "fixed" => Backend::Fixed(DashMap::new()),
            "sliding" => Backend::Sliding(DashMap::new()),
            other => {
                return Err(RateLimitError::InvalidConfig(format!(
                    "algorithm must be \"fixed\" or \"sliding\", got {other:?}"
                ))
                .into())
            }
        };

        Ok(Self {
            limit: options.limit as u64,
            window_seconds: options.window_seconds as u64,
            backend,
            counters: Counters::default(),
        })
    }

    /// The active algorithm: `"fixed"` or `"sliding"`.
    #[napi(getter)]
    pub fn algorithm(&self) -> String {
        self.backend.name().to_string()
    }

    /// Tries to consume one request for `key`. Returns `true` if allowed.
    #[napi]
    pub fn allow(&self, key: String) -> bool {
        self.apply(key).allowed
    }

    /// Like `allow()`, but returns the detailed result of the check.
    #[napi]
    pub fn check(&self, key: String) -> CheckResult {
        self.apply(key)
    }

    /// Requests left for `key` in the current window (read-only: does not
    /// consume). If the key does not exist or its window expired, returns the limit.
    #[napi]
    pub fn remaining(&self, key: String) -> i64 {
        self.backend
            .remaining(&key, self.limit, self.window_seconds, now_seconds()) as i64
    }

    /// Removes the state of a single key (resets its count).
    #[napi]
    pub fn reset(&self, key: String) {
        self.backend.reset(&key);
    }

    /// Removes the state of every key. Does not reset the `stats()` counters.
    #[napi]
    pub fn clear(&self) {
        self.backend.clear();
    }

    /// Removes keys whose window expired and returns how many were removed.
    #[napi]
    pub fn cleanup(&self) -> i64 {
        self.backend.cleanup(self.window_seconds, now_seconds()) as i64
    }

    /// Returns the accumulated counters and the number of active keys.
    #[napi]
    pub fn stats(&self) -> RateLimitStats {
        let allowed = self.counters.allowed();
        let blocked = self.counters.blocked();
        RateLimitStats {
            allowed: allowed as i64,
            blocked: blocked as i64,
            checks: (allowed + blocked) as i64,
            active_keys: self.backend.len() as i64,
        }
    }

    /// Core shared by `allow()` and `check()`: applies the active algorithm
    /// atomically per key and records the statistic.
    fn apply(&self, key: String) -> CheckResult {
        let now = now_seconds();
        let decision = self
            .backend
            .apply(key, self.limit, self.window_seconds, now);

        self.counters.record(decision.allowed);

        CheckResult {
            allowed: decision.allowed,
            limit: self.limit as i64,
            remaining: decision.remaining as i64,
            retry_after: decision.retry_after as i64,
            reset_after: decision.reset_after as i64,
        }
    }
}
