//! Erros do núcleo do rate limiter.

use thiserror::Error;

/// Erros que podem ocorrer ao configurar/usar o limiter.
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Configuração inválida passada ao construtor (ex.: limite <= 0).
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Converte o erro do domínio em `napi::Error`, que o Node lança como uma
/// exceção JavaScript comum (com mensagem amigável).
impl From<RateLimitError> for napi::Error {
    fn from(err: RateLimitError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}
