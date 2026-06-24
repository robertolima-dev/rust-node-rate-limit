import { describe, it, expect } from "vitest";
import { RateLimiter } from "../js/index";

describe("stats()", () => {
  it("começa zerado", () => {
    const limiter = new RateLimiter({ limit: 2, windowSeconds: 60 });
    const stats = limiter.stats();
    expect(stats.allowed).toBe(0);
    expect(stats.blocked).toBe(0);
    expect(stats.checks).toBe(0);
    expect(stats.activeKeys).toBe(0);
  });

  it("contabiliza allowed, blocked e checks", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
    limiter.allow("a"); // allowed
    limiter.allow("a"); // blocked
    limiter.check("a"); // blocked

    const stats = limiter.stats();
    expect(stats.allowed).toBe(1);
    expect(stats.blocked).toBe(2);
    expect(stats.checks).toBe(3);
  });

  it("activeKeys conta chaves distintas rastreadas", () => {
    const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });
    limiter.allow("a");
    limiter.allow("b");
    limiter.allow("c");
    expect(limiter.stats().activeKeys).toBe(3);
  });

  it("clear() zera activeKeys mas mantém os contadores acumulados", () => {
    const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });
    limiter.allow("a");
    limiter.allow("b");
    limiter.clear();

    const stats = limiter.stats();
    expect(stats.activeKeys).toBe(0);
    expect(stats.allowed).toBe(2);
    expect(stats.checks).toBe(2);
  });
});
