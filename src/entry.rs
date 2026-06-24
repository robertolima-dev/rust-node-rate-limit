//! Entrada de contagem por chave (uma "janela" do Fixed Window).

use std::time::{SystemTime, UNIX_EPOCH};

/// Estado de uma chave dentro da janela fixa atual.
///
/// `count`        -> quantas requisições já foram contabilizadas na janela.
/// `window_start` -> timestamp Unix (segundos) em que a janela atual começou.
pub struct RateLimitEntry {
    pub count: u64,
    pub window_start: u64,
}

impl RateLimitEntry {
    /// Cria uma entrada nova, iniciando a janela em `now`.
    pub fn new(now: u64) -> Self {
        Self {
            count: 0,
            window_start: now,
        }
    }
}

/// Tempo atual em segundos desde a época Unix.
///
/// Em caso de relógio inválido (anterior à época), retorna 0 em vez de entrar
/// em pânico — manter o limiter funcionando é preferível a derrubar o processo.
pub fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
