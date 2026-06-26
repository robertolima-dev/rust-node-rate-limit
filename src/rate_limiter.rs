//! A classe `RateLimiter` exposta ao Node.

use dashmap::DashMap;
use napi_derive::napi;

use crate::cleanup;
use crate::entry::{now_seconds, RateLimitEntry};
use crate::errors::RateLimitError;
use crate::fixed_window;
use crate::sliding_window::{self, SlidingEntry};
use crate::stats::{CheckResult, Counters, RateLimitStats};

/// Backend de algoritmo selecionado. Cada variante é dona do seu próprio
/// `DashMap` (com o tipo de entrada adequado); os métodos do `RateLimiter`
/// despacham para o ativo. Manter mapas separados deixa o caminho do Fixed
/// Window inalterado e o do Sliding Window autocontido.
enum Backend {
    Fixed(DashMap<String, RateLimitEntry>),
    Sliding(DashMap<String, SlidingEntry>),
}

impl Backend {
    /// Núcleo de `allow()`/`check()`: aplica o algoritmo ativo de forma atômica
    /// por chave. O `RefMut` segura o lock do shard durante a modificação,
    /// garantindo atomicidade leitura-modificação-escrita para esta chave.
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

    /// Requisições restantes para `key` sem consumir. Chave inexistente/expirada
    /// devolve o limite cheio.
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

/// Opções do construtor: `new RateLimiter({ limit, windowSeconds, algorithm })`.
///
/// `#[napi(object)]` aceita um objeto JS simples. `window_seconds` vira
/// `windowSeconds` no JS automaticamente. `algorithm` é opcional e default
/// `"fixed"`.
#[napi(object)]
pub struct RateLimiterOptions {
    /// Máximo de requisições permitidas por janela.
    pub limit: i64,
    /// Duração da janela, em segundos.
    pub window_seconds: i64,
    /// Estratégia: `"fixed"` (default) ou `"sliding"` (sliding window counter,
    /// que suaviza rajadas na borda da janela).
    pub algorithm: Option<String>,
}

/// Rate limiter local (in-process) com algoritmos Fixed Window e Sliding Window.
///
/// Os métodos recebem `&self` (não `&mut self`): a mutabilidade é interna via
/// `DashMap` (por chave) e contadores atômicos. Isso torna o limiter seguro e
/// eficiente sob concorrência, sem um lock global.
#[napi]
pub struct RateLimiter {
    limit: u64,
    window_seconds: u64,
    backend: Backend,
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

    /// O algoritmo ativo: `"fixed"` ou `"sliding"`.
    #[napi(getter)]
    pub fn algorithm(&self) -> String {
        self.backend.name().to_string()
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
        self.backend
            .remaining(&key, self.limit, self.window_seconds, now_seconds()) as i64
    }

    /// Remove o estado de uma única chave (zera sua contagem).
    #[napi]
    pub fn reset(&self, key: String) {
        self.backend.reset(&key);
    }

    /// Remove o estado de todas as chaves. Não zera os contadores de `stats()`.
    #[napi]
    pub fn clear(&self) {
        self.backend.clear();
    }

    /// Remove chaves com janela expirada e retorna quantas foram removidas.
    #[napi]
    pub fn cleanup(&self) -> i64 {
        self.backend.cleanup(self.window_seconds, now_seconds()) as i64
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
            active_keys: self.backend.len() as i64,
        }
    }

    /// Núcleo compartilhado por `allow()` e `check()`: aplica o algoritmo ativo
    /// de forma atômica por chave e registra a estatística.
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
