import { describe, it, expect } from "vitest";
import { RateLimiter, hello } from "../js/index";

describe("hello()", () => {
  it("retorna a saudação vinda do core Rust", () => {
    expect(hello()).toBe("Hello from Rust");
  });
});

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
