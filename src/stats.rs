//! Contadores de estatísticas e tipos de retorno expostos ao JS.

use napi_derive::napi;
use std::sync::atomic::{AtomicU64, Ordering};

/// Contadores acumulados (thread-safe) do limiter.
///
/// Usamos atômicos `Relaxed`: queremos apenas contagens corretas no agregado,
/// sem necessidade de ordenação entre as duas variáveis.
#[derive(Default)]
pub struct Counters {
    allowed: AtomicU64,
    blocked: AtomicU64,
}

impl Counters {
    /// Registra o resultado de uma verificação.
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

/// Retorno de `check()`.
///
/// `#[napi(object)]` gera um objeto JS simples já em `camelCase`. Usamos `i64`
/// porque vira `number` no JavaScript (evitamos `BigInt`).
#[napi(object)]
pub struct CheckResult {
    /// Se a requisição é permitida.
    pub allowed: bool,
    /// Limite configurado para a janela.
    pub limit: i64,
    /// Requisições restantes na janela atual.
    pub remaining: i64,
    /// Segundos até poder tentar novamente (0 quando permitida).
    pub retry_after: i64,
    /// Segundos até a janela atual reiniciar.
    pub reset_after: i64,
}

/// Retorno de `stats()`.
#[napi(object)]
pub struct RateLimitStats {
    /// Total de requisições permitidas desde a criação do limiter.
    pub allowed: i64,
    /// Total de requisições bloqueadas desde a criação do limiter.
    pub blocked: i64,
    /// Total de verificações (allowed + blocked).
    pub checks: i64,
    /// Número de chaves atualmente rastreadas.
    pub active_keys: i64,
}
