//! Limpeza de chaves expiradas.
//!
//! O Fixed Window não remove entradas automaticamente: uma chave usada uma vez
//! permanece no mapa. `cleanup()` permite ao processo Node recuperar memória de
//! chaves cujas janelas já expiraram (úteis em cenários com muitas chaves
//! efêmeras, como IPs).

use crate::entry::RateLimitEntry;
use dashmap::DashMap;

/// Remove todas as entradas cuja janela já expirou em `now`.
/// Retorna quantas chaves foram removidas.
pub fn remove_expired(entries: &DashMap<String, RateLimitEntry>, window: u64, now: u64) -> usize {
    let before = entries.len();
    // `retain` mantém apenas as entradas ainda dentro da janela.
    entries.retain(|_, entry| now.saturating_sub(entry.window_start) < window);
    before - entries.len()
}
