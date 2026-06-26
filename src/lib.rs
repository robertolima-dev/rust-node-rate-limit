//! rust-node-rate-limit
//!
//! Núcleo nativo (Rust) do rate limiter. Expõe ao Node, via N-API:
//! - `hello()`        -> string de sanity-check.
//! - `RateLimiter`    -> classe de rate limiting (algoritmo Fixed Window).
//!
//! Como o Rust expõe coisas para o Node:
//! O macro `#[napi]` (do crate `napi-derive`) gera, em tempo de compilação, o
//! código de "cola" que registra funções/classes no módulo N-API. Quando o Node
//! carrega o arquivo `.node`, elas aparecem como JavaScript normal. Nomes em
//! `snake_case` no Rust viram `camelCase` no JS automaticamente.

use napi_derive::napi;

// Submódulos do núcleo. A separação segue a arquitetura do BACKLOG:
// cada arquivo tem uma responsabilidade única.
mod cleanup;
mod entry;
mod errors;
mod fixed_window;
mod rate_limiter;
mod sliding_window;
mod stats;

// Re-exports públicos (consumidos pela camada de testes Rust e pela geração de
// tipos). O registro N-API acontece no ponto de definição de cada `#[napi]`.
pub use rate_limiter::{RateLimiter, RateLimiterOptions};
pub use stats::{CheckResult, RateLimitStats};

/// Sanity-check: confirma que o addon nativo carregou corretamente.
#[napi]
pub fn hello() -> String {
    "Hello from Rust".to_string()
}
