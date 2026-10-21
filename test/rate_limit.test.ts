import { describe, it, expect } from "vitest";
import { RateLimiter } from "../js/index";

describe("RateLimiter — allow()/check()", () => {
  it("permite requisições abaixo do limite", () => {
    const limiter = new RateLimiter({ limit: 3, windowSeconds: 60 });
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
  });

  it("bloqueia depois de atingir o limite", () => {
    const limiter = new RateLimiter({ limit: 2, windowSeconds: 60 });
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(false);
    expect(limiter.allow("a")).toBe(false);
  });

  it("check() retorna o resultado detalhado", () => {
    const limiter = new RateLimiter({ limit: 2, windowSeconds: 60 });

    const first = limiter.check("a");
    expect(first.allowed).toBe(true);
    expect(first.limit).toBe(2);
    expect(first.remaining).toBe(1);
    expect(first.retryAfter).toBe(0);
    expect(first.resetAfter).toBeGreaterThan(0);
    expect(first.resetAfter).toBeLessThanOrEqual(60);

    limiter.check("a"); // consome o segundo

    const blocked = limiter.check("a");
    expect(blocked.allowed).toBe(false);
    expect(blocked.remaining).toBe(0);
    expect(blocked.retryAfter).toBeGreaterThan(0);
  });

  it("reinicia após a expiração da janela", async () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 1 });
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("a")).toBe(false);

    // Aguarda a janela de 1s expirar.
    await new Promise((r) => setTimeout(r, 1100));

    expect(limiter.allow("a")).toBe(true);
  });

  it("mantém chaves independentes", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
    expect(limiter.allow("user:1")).toBe(true);
    expect(limiter.allow("user:2")).toBe(true);
    expect(limiter.allow("user:1")).toBe(false);
    expect(limiter.allow("user:2")).toBe(false);
  });
});

describe("RateLimiter — check() com buffer compartilhado", () => {
  it("cada chamada devolve um objeto novo e limiters não se misturam", () => {
    const a = new RateLimiter({ limit: 1, windowSeconds: 60 });
    const b = new RateLimiter({ limit: 5, windowSeconds: 30, algorithm: "sliding" });
    const a1 = a.check("k");
    const b1 = b.check("k");
    const a2 = a.check("k");
    expect(a1).toMatchObject({ allowed: true, limit: 1, remaining: 0 });
    expect(b1).toMatchObject({ allowed: true, limit: 5, remaining: 4 });
    expect(a2).toMatchObject({ allowed: false, limit: 1, remaining: 0 });
    expect(a2.retryAfter).toBeGreaterThanOrEqual(1);
    expect(a1).not.toBe(a2);
    expect(a1.allowed).toBe(true); // not overwritten by the later calls
    expect(Object.keys(a1).sort()).toEqual(
      ["allowed", "limit", "remaining", "resetAfter", "resetAfterMs", "retryAfter", "retryAfterMs"],
    );
  });
});

describe("RateLimiter — remaining()/reset()/clear()", () => {
  it("remaining() reflete o consumo sem consumir", () => {
    const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });
    expect(limiter.remaining("a")).toBe(5);
    limiter.allow("a");
    limiter.allow("a");
    expect(limiter.remaining("a")).toBe(3);
    // Chamar remaining() não consome.
    expect(limiter.remaining("a")).toBe(3);
  });

  it("reset() zera uma única chave", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
    limiter.allow("a");
    expect(limiter.allow("a")).toBe(false);
    limiter.reset("a");
    expect(limiter.allow("a")).toBe(true);
  });

  it("clear() zera todas as chaves", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
    limiter.allow("a");
    limiter.allow("b");
    limiter.clear();
    expect(limiter.allow("a")).toBe(true);
    expect(limiter.allow("b")).toBe(true);
  });

  it("cleanup() remove chaves expiradas", async () => {
    const limiter = new RateLimiter({ limit: 5, windowSeconds: 1 });
    limiter.allow("a");
    limiter.allow("b");
    expect(limiter.stats().activeKeys).toBe(2);

    await new Promise((r) => setTimeout(r, 1100));

    expect(limiter.cleanup()).toBe(2);
    expect(limiter.stats().activeKeys).toBe(0);
  });
});

describe("RateLimiter — configuração inválida", () => {
  it("rejeita limit <= 0", () => {
    expect(() => new RateLimiter({ limit: 0, windowSeconds: 60 })).toThrow();
    expect(() => new RateLimiter({ limit: -1, windowSeconds: 60 })).toThrow();
  });

  it("rejeita windowSeconds <= 0", () => {
    expect(() => new RateLimiter({ limit: 10, windowSeconds: 0 })).toThrow();
    expect(() => new RateLimiter({ limit: 10, windowSeconds: -5 })).toThrow();
  });

  it("rejeita limit fracionário em vez de truncar", () => {
    expect(() => new RateLimiter({ limit: 1.5, windowSeconds: 60 })).toThrow(
      /limit must be a positive integer, got 1.5/,
    );
  });

  it("rejeita NaN, Infinity e janelas menores que 1 ms", () => {
    expect(() => new RateLimiter({ limit: NaN, windowSeconds: 60 })).toThrow(/limit/);
    expect(() => new RateLimiter({ limit: Infinity, windowSeconds: 60 })).toThrow(/limit/);
    expect(() => new RateLimiter({ limit: 1, windowSeconds: NaN })).toThrow(/windowSeconds/);
    expect(() => new RateLimiter({ limit: 1, windowSeconds: Infinity })).toThrow(/windowSeconds/);
    expect(() => new RateLimiter({ limit: 1, windowSeconds: 0.0001 })).toThrow(/at least 0.001/);
  });
});

describe("RateLimiter — janelas menores que 1 segundo", () => {
  for (const algorithm of ["fixed", "sliding"] as const) {
    it(`${algorithm}: windowSeconds 0.2 libera de novo depois de 200 ms`, async () => {
      const limiter = new RateLimiter({ limit: 2, windowSeconds: 0.2, algorithm });
      expect([limiter.allow("k"), limiter.allow("k"), limiter.allow("k")]).toEqual([
        true,
        true,
        false,
      ]);
      // The sliding counter still weighs the previous window just after the boundary,
      // so wait two windows to be past it for both algorithms.
      await new Promise((r) => setTimeout(r, 450));
      expect(limiter.allow("k")).toBe(true);
    });
  }

  it("retryAfterMs tem precisão de ms e retryAfter arredonda para cima", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 0.5 });
    limiter.allow("k");
    const blocked = limiter.check("k");
    expect(blocked.allowed).toBe(false);
    expect(blocked.retryAfterMs).toBeGreaterThan(0);
    expect(blocked.retryAfterMs).toBeLessThanOrEqual(500);
    expect(blocked.retryAfter).toBe(1); // never 0 while blocked
    expect(blocked.resetAfterMs).toBe(blocked.retryAfterMs);
  });

  it("windowSeconds 1.5 não é truncado para 1", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 1.5 });
    const first = limiter.check("k");
    expect(first.resetAfterMs).toBeGreaterThan(1000);
    expect(first.resetAfter).toBe(2);
  });
});

describe("RateLimiter — alta concorrência", () => {
  it("permite exatamente `limit` requisições sob muitas chamadas", () => {
    const limit = 1000;
    const limiter = new RateLimiter({ limit, windowSeconds: 60 });

    let allowed = 0;
    for (let i = 0; i < limit * 2; i++) {
      if (limiter.allow("burst")) allowed += 1;
    }

    expect(allowed).toBe(limit);
    const stats = limiter.stats();
    expect(stats.allowed).toBe(limit);
    expect(stats.blocked).toBe(limit);
  });
});
