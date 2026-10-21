//! Errors of the rate limiter core.

use thiserror::Error;

/// Errors that can happen when configuring or using the limiter.
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Invalid configuration passed to the constructor (e.g. limit <= 0).
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Converts the domain error into a `napi::Error`, which Node throws as a
/// regular JavaScript exception (with a readable message).
impl From<RateLimitError> for napi::Error {
    fn from(err: RateLimitError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}
