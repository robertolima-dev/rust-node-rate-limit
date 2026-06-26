import { describe, it, expect } from "vitest";
import { RateLimiter } from "../js/index";

// A matemática de suavização na borda é verificada de forma determinística nos
// testes Rust (src/sliding_window.rs), onde o relógio é injetado. Aqui cobrimos
// a API pública: seleção de algoritmo, o getter `algorithm`, validação e o
// comportamento básico de admitir/bloquear.

describe("RateLimiter — algorithm", () => {
  it("usa Fixed Window por padrão", () => {
    const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });
    expect(limiter.algorithm).toBe("fixed");
  });

  it("expõe o algoritmo sliding via getter", () => {
    const limiter = new RateLimiter({
      limit: 5,
      windowSeconds: 60,
      algorithm: "sliding",
    });
    expect(limiter.algorithm).toBe("sliding");
  });

  it("rejeita um algoritmo desconhecido", () => {
    expect(
      () =>
        new RateLimiter({ limit: 5, windowSeconds: 60, algorithm: "leaky" }),
    ).toThrow();
  });
});

describe("RateLimiter — Sliding Window", () => {
  const sliding = () =>
    new RateLimiter({ limit: 3, windowSeconds: 60, algorithm: "sliding" });

  it("permite requisições abaixo do limite", () => {
    const limiter = sliding();
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
  });

  it("bloqueia depois de atingir o limite", () => {
    const limiter = new RateLimiter({
      limit: 2,
      windowSeconds: 60,
      algorithm: "sliding",
    });
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(false);
  });

  it("remaining() reflete o consumo sem consumir", () => {
    const limiter = sliding();
    expect(limiter.remaining("a")).toBe(3);
    limiter.allow("a");
    expect(limiter.remaining("a")).toBe(2);
    expect(limiter.remaining("a")).toBe(2);
  });

  it("check() bloqueado reporta retryAfter", () => {
    const limiter = new RateLimiter({
      limit: 1,
      windowSeconds: 60,
      algorithm: "sliding",
    });
    expect(limiter.check("a").allowed).toBe(true);
    const blocked = limiter.check("a");
    expect(blocked.allowed).toBe(false);
    expect(blocked.remaining).toBe(0);
    expect(blocked.retryAfter).toBeGreaterThan(0);
  });

  it("mantém chaves independentes", () => {
    const limiter = new RateLimiter({
      limit: 1,
      windowSeconds: 60,
      algorithm: "sliding",
    });
    expect(limiter.allow("user:1")).toBe(true);
    expect(limiter.allow("user:2")).toBe(true);
    expect(limiter.allow("user:1")).toBe(false);
  });

  it("stats() conta allowed/blocked e activeKeys", () => {
    const limiter = new RateLimiter({
      limit: 1,
      windowSeconds: 60,
      algorithm: "sliding",
    });
    limiter.allow("a");
    limiter.allow("a"); // bloqueado
    const stats = limiter.stats();
    expect(stats.allowed).toBe(1);
    expect(stats.blocked).toBe(1);
    expect(stats.activeKeys).toBe(1);
  });
});
