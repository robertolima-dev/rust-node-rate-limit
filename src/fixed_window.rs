//! Algoritmo Fixed Window.
//!
//! Cada chave tem uma janela de `window` segundos. Dentro da janela são
//! permitidas até `limit` requisições; ao expirar, a janela "vira" e a contagem
//! recomeça do zero. É simples, previsível e barato — ideal para um core nativo.

use crate::entry::RateLimitEntry;

/// Resultado de aplicar uma requisição (hit) sobre uma entrada.
pub struct Decision {
    /// Se a requisição é permitida nesta janela.
    pub allowed: bool,
    /// Requisições restantes na janela atual (0 quando bloqueada).
    pub remaining: u64,
    /// Segundos até poder tentar de novo (0 quando permitida).
    pub retry_after: u64,
    /// Segundos até a janela atual reiniciar.
    pub reset_after: u64,
}

/// Aplica uma requisição à entrada, sob o algoritmo Fixed Window.
///
/// Recebe `&mut RateLimitEntry` porque pode reiniciar a janela e incrementar a
/// contagem in-place. O chamador segura o lock do shard do DashMap durante a
/// chamada, garantindo atomicidade leitura-modificação-escrita por chave.
pub fn hit(entry: &mut RateLimitEntry, limit: u64, window: u64, now: u64) -> Decision {
    // Janela expirada? Reinicia contagem e marca o novo início.
    if now.saturating_sub(entry.window_start) >= window {
        entry.window_start = now;
        entry.count = 0;
    }

    let elapsed = now.saturating_sub(entry.window_start);
    let reset_after = window.saturating_sub(elapsed);

    if entry.count < limit {
        entry.count += 1;
        Decision {
            allowed: true,
            remaining: limit - entry.count,
            retry_after: 0,
            reset_after,
        }
    } else {
        Decision {
            allowed: false,
            remaining: 0,
            retry_after: reset_after,
            reset_after,
        }
    }
}
