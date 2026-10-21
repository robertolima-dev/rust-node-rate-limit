//! rust-node-rate-limit
//!
//! Native (Rust) core of the rate limiter. Exposes to Node, through N-API, the
//! `RateLimiter` class (Fixed and Sliding Window).
//!
//! How Rust exposes things to Node:
//! The `#[napi]` macro (from the `napi-derive` crate) generates, at compile time,
//! the glue code that registers functions/classes in the N-API module. When Node
//! loads the `.node` file, they show up as regular JavaScript. `snake_case` names
//! in Rust become `camelCase` in JS automatically.

// Core submodules. The split follows the BACKLOG architecture:
// each file has a single responsibility.
mod cleanup;
mod entry;
mod errors;
mod fixed_window;
mod rate_limiter;
mod sliding_window;
mod stats;

// Public re-exports (used by the Rust test layer and by type generation).
// N-API registration happens where each `#[napi]` item is defined.
pub use rate_limiter::{RateLimiter, RateLimiterOptions};
pub use stats::{CheckResult, RateLimitStats};
