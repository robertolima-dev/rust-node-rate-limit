//! A classe `RateLimiter` exposta ao Node.

use dashmap::DashMap;
use napi_derive::napi;

use crate::cleanup;
use crate::entry::{now_seconds, RateLimitEntry};
use crate::errors::RateLimitError;
use crate::fixed_window;
use crate::stats::{CheckResult, Counters, RateLimitStats};

/// Opções do construtor: `new RateLimiter({ limit, windowSeconds })`.
///
/// `#[napi(object)]` aceita um objeto JS simples. `window_seconds` vira
/// `windowSeconds` no JS automaticamente.
#[napi(object)]
pub struct RateLimiterOptions {
    /// Máximo de requisições permitidas por janela.
    pub limit: i64,
    /// Duração da janela, em segundos.
    pub window_seconds: i64,
}

/// Rate limiter local (in-process) com algoritmo Fixed Window.
///
/// Os métodos recebem `&self` (não `&mut self`): a mutabilidade é interna via
/// `DashMap` (por chave) e contadores atômicos. Isso torna o limiter seguro e
/// eficiente sob concorrência, sem um lock global.
#[napi]
pub struct RateLimiter {
    limit: u64,
    window_seconds: u64,
    entries: DashMap<String, RateLimitEntry>,
    counters: Counters,
}

#[napi]
impl RateLimiter {
    /// Cria um limiter. Lança erro (exceção JS) se a configuração for inválida.
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

        Ok(Self {
            limit: options.limit as u64,
            window_seconds: options.window_seconds as u64,
            entries: DashMap::new(),
            counters: Counters::default(),
        })
    }

    /// Tenta consumir uma requisição para `key`. Retorna `true` se permitida.
    #[napi]
    pub fn allow(&self, key: String) -> bool {
        self.apply(key).allowed
    }

    /// Como `allow()`, mas retorna o resultado detalhado da verificação.
    #[napi]
    pub fn check(&self, key: String) -> CheckResult {
        self.apply(key)
    }

    /// Requisições restantes para `key` na janela atual (somente leitura: não
    /// consome). Se a chave não existe ou a janela expirou, retorna o limite.
    #[napi]
    pub fn remaining(&self, key: String) -> i64 {
        let now = now_seconds();
        match self.entries.get(&key) {
            Some(entry) if now.saturating_sub(entry.window_start) < self.window_seconds => {
                self.limit.saturating_sub(entry.count) as i64
            }
            _ => self.limit as i64,
        }
    }

    /// Remove o estado de uma única chave (zera sua contagem).
    #[napi]
    pub fn reset(&self, key: String) {
        self.entries.remove(&key);
    }

    /// Remove o estado de todas as chaves. Não zera os contadores de `stats()`.
    #[napi]
    pub fn clear(&self) {
        self.entries.clear();
    }

    /// Remove chaves com janela expirada e retorna quantas foram removidas.
    #[napi]
    pub fn cleanup(&self) -> i64 {
        cleanup::remove_expired(&self.entries, self.window_seconds, now_seconds()) as i64
    }

    /// Retorna os contadores acumulados e o número de chaves ativas.
    #[napi]
    pub fn stats(&self) -> RateLimitStats {
        let allowed = self.counters.allowed();
        let blocked = self.counters.blocked();
        RateLimitStats {
            allowed: allowed as i64,
            blocked: blocked as i64,
            checks: (allowed + blocked) as i64,
            active_keys: self.entries.len() as i64,
        }
    }

    /// Núcleo compartilhado por `allow()` e `check()`: aplica o Fixed Window de
    /// forma atômica por chave e registra a estatística.
    fn apply(&self, key: String) -> CheckResult {
        let now = now_seconds();

        // O `RefMut` retornado segura o lock do shard durante a modificação,
        // garantindo atomicidade leitura-modificação-escrita para esta chave.
        let decision = {
            let mut entry = self
                .entries
                .entry(key)
                .or_insert_with(|| RateLimitEntry::new(now));
            fixed_window::hit(entry.value_mut(), self.limit, self.window_seconds, now)
        };

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
