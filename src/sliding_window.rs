//! Algoritmo Sliding Window (counter).
//!
//! O Fixed Window tem uma fraqueza conhecida: um cliente pode mandar `limit`
//! requisições no fim de uma janela e `limit` no começo da seguinte, passando
//! `2 * limit` num intervalo menor que uma única janela. O **sliding window
//! counter** suaviza essa borda guardando a contagem da janela *atual* (alinhada)
//! mais a da *anterior*, e ponderando a anterior pela fração que ainda se
//! sobrepõe à janela móvel que termina em `now`:
//!
//! ```text
//! estimado = anterior * peso + atual
//! peso     = (window - decorrido_na_janela_atual) / window
//! ```
//!
//! É uma aproximação (assume distribuição uniforme na janela anterior), mas é
//! O(1) em tempo e memória por chave — diferente de um sliding *log*, que guarda
//! todos os timestamps — e remove o "dobro" na borda.

use crate::fixed_window::Decision;

/// Estado por chave do sliding window counter.
///
/// `window_index` é o número da janela alinhada (`now / window`) que `current`
/// conta; `previous` é a contagem de `window_index - 1`.
pub struct SlidingEntry {
    pub window_index: u64,
    pub current: u64,
    pub previous: u64,
}

impl SlidingEntry {
    /// Cria uma entrada nova já posicionada na janela `window_index`.
    pub fn new(window_index: u64) -> Self {
        Self {
            window_index,
            current: 0,
            previous: 0,
        }
    }

    /// Rola `current`/`previous` para descrever a janela que contém `idx`. Um
    /// avanço de um passo empurra `current` para `previous`; um salto maior torna
    /// ambas obsoletas e zera tudo.
    fn advance_to(&mut self, idx: u64) {
        if idx == self.window_index {
            // mesma janela — nada a fazer
        } else if idx == self.window_index + 1 {
            self.previous = self.current;
            self.current = 0;
            self.window_index = idx;
        } else {
            self.previous = 0;
            self.current = 0;
            self.window_index = idx;
        }
    }
}

/// Peso aplicado à janela anterior: a fração de `window` ainda à frente de `now`
/// dentro da janela atual.
fn weight(window: u64, elapsed: u64) -> f64 {
    window.saturating_sub(elapsed) as f64 / window as f64
}

/// Aplica uma requisição (hit) à entrada, sob o algoritmo Sliding Window.
///
/// Recebe `&mut SlidingEntry` porque pode rolar a janela e incrementar a
/// contagem in-place. O chamador segura o lock do shard do DashMap durante a
/// chamada, garantindo atomicidade leitura-modificação-escrita por chave.
pub fn hit(entry: &mut SlidingEntry, limit: u64, window: u64, now: u64) -> Decision {
    let idx = now / window;
    entry.advance_to(idx);

    let elapsed = now - idx * window;
    let w = weight(window, elapsed);
    let estimated = entry.previous as f64 * w + entry.current as f64;
    let reset_after = window.saturating_sub(elapsed);

    let allowed = estimated < limit as f64;
    if allowed {
        entry.current += 1;
    }

    let estimated_after = entry.previous as f64 * w + entry.current as f64;
    let remaining = (limit as f64 - estimated_after).max(0.0).floor() as u64;

    Decision {
        allowed,
        remaining,
        retry_after: if allowed { 0 } else { reset_after },
        reset_after,
    }
}

/// Requisições restantes para a entrada em `now`, sem consumir (somente leitura).
pub fn remaining(entry: &SlidingEntry, limit: u64, window: u64, now: u64) -> u64 {
    let idx = now / window;
    // Conta efetiva (rolada para a frente) sem mutar a entrada.
    let (previous, current) = if idx == entry.window_index {
        (entry.previous, entry.current)
    } else if idx == entry.window_index + 1 {
        (entry.current, 0)
    } else {
        (0, 0)
    };
    let elapsed = now - idx * window;
    let estimated = previous as f64 * weight(window, elapsed) + current as f64;
    (limit as f64 - estimated).max(0.0).floor() as u64
}

/// `true` quando a entrada não contribui mais (não vista há ao menos duas
/// janelas: nem `current` nem `previous` se sobrepõem à janela móvel).
pub fn is_stale(entry: &SlidingEntry, window: u64, now: u64) -> bool {
    let idx = now / window;
    idx >= entry.window_index.saturating_add(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_up_to_limit_within_one_window() {
        let mut e = SlidingEntry::new(0);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(hit(&mut e, 3, 60, 0).allowed);
        assert!(!hit(&mut e, 3, 60, 0).allowed);
    }

    #[test]
    fn smooths_the_boundary_burst() {
        // limit=10/60s. Enche a primeira janela em t=30, depois cruza para a
        // próxima no meio (t=90): a anterior pesa 0.5, liberando só 5.
        let mut e = SlidingEntry::new(0);
        for _ in 0..10 {
            assert!(hit(&mut e, 10, 60, 30).allowed);
        }
        assert_eq!(remaining(&e, 10, 60, 90), 5);
        let mut admitted = 0;
        for _ in 0..10 {
            if hit(&mut e, 10, 60, 90).allowed {
                admitted += 1;
            }
        }
        assert_eq!(admitted, 5, "burst na borda deve ser limitado a 5, não 10");
    }

    #[test]
    fn fully_resets_after_two_windows() {
        let mut e = SlidingEntry::new(0);
        assert!(hit(&mut e, 2, 10, 0).allowed);
        assert!(hit(&mut e, 2, 10, 0).allowed);
        assert!(!hit(&mut e, 2, 10, 0).allowed);
        assert_eq!(remaining(&e, 2, 10, 20), 2);
        assert!(hit(&mut e, 2, 10, 20).allowed);
    }

    #[test]
    fn staleness_after_two_windows() {
        let mut e = SlidingEntry::new(0);
        hit(&mut e, 1, 10, 0);
        assert!(!is_stale(&e, 10, 10)); // idx 1: anterior ainda overlap
        assert!(is_stale(&e, 10, 20)); // idx 2: obsoleta
    }
}
