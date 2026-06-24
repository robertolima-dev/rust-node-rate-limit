import { describe, it, expect } from "vitest";
import Fastify from "fastify";
import { rateLimitMiddleware } from "../js/express";
import { rateLimitPlugin } from "../js/fastify";

// ---------------------------------------------------------------------------
// Express — testado com objetos req/res simulados (sem subir servidor).
// ---------------------------------------------------------------------------
function mockRes() {
  return {
    statusCode: 200,
    headers: {} as Record<string, string | number>,
    body: undefined as unknown,
    setHeader(name: string, value: string | number) {
      this.headers[name] = value;
    },
    status(code: number) {
      this.statusCode = code;
      return this;
    },
    json(payload: unknown) {
      this.body = payload;
      return this;
    },
  };
}

describe("Express — rateLimitMiddleware", () => {
  it("permite abaixo do limite e seta headers X-RateLimit-*", () => {
    const mw = rateLimitMiddleware({ limit: 2, windowSeconds: 60 });
    const req = { ip: "1.1.1.1" } as any;
    const res = mockRes();
    let nextCalled = false;

    mw(req, res as any, () => {
      nextCalled = true;
    });

    expect(nextCalled).toBe(true);
    expect(res.headers["X-RateLimit-Limit"]).toBe(2);
    expect(res.headers["X-RateLimit-Remaining"]).toBe(1);
  });

  it("bloqueia com 429 e Retry-After ao exceder", () => {
    const mw = rateLimitMiddleware({ limit: 1, windowSeconds: 60 });
    const req = { ip: "2.2.2.2" } as any;

    // 1ª passa
    mw(req, mockRes() as any, () => {});

    // 2ª bloqueia
    const res = mockRes();
    let nextCalled = false;
    mw(req, res as any, () => {
      nextCalled = true;
    });

    expect(nextCalled).toBe(false);
    expect(res.statusCode).toBe(429);
    expect(res.body).toEqual({ message: "Too many requests" });
    expect(res.headers["Retry-After"]).toBeGreaterThan(0);
  });

  it("isola chaves por IP", () => {
    const mw = rateLimitMiddleware({ limit: 1, windowSeconds: 60 });

    const resA = mockRes();
    mw({ ip: "10.0.0.1" } as any, resA as any, () => {});
    const resB = mockRes();
    let bAllowed = false;
    mw({ ip: "10.0.0.2" } as any, resB as any, () => {
      bAllowed = true;
    });

    expect(bAllowed).toBe(true);
    expect(resB.statusCode).toBe(200);
  });
});

// ---------------------------------------------------------------------------
// Fastify — testado com fastify.inject() (sem porta real).
// ---------------------------------------------------------------------------
describe("Fastify — rateLimitPlugin", () => {
  it("permite e depois bloqueia com 429", async () => {
    const app = Fastify();
    await app.register(rateLimitPlugin, { limit: 1, windowSeconds: 60 });
    app.get("/", async () => ({ ok: true }));
    await app.ready();

    const first = await app.inject({ method: "GET", url: "/" });
    expect(first.statusCode).toBe(200);
    expect(first.headers["x-ratelimit-limit"]).toBe("1");

    const second = await app.inject({ method: "GET", url: "/" });
    expect(second.statusCode).toBe(429);
    expect(second.json()).toEqual({ message: "Too many requests" });
    expect(second.headers["retry-after"]).toBeDefined();

    await app.close();
  });
});
