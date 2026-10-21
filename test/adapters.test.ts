import { describe, it, expect } from "vitest";
import type { AddressInfo } from "node:net";
import type { Server } from "node:http";
import express from "express";
import Fastify from "fastify";
import { HttpException } from "@nestjs/common";
import { RateLimiter } from "../js/index";
import { rateLimitMiddleware } from "../js/express";
import { rateLimitPlugin } from "../js/fastify";
import { RateLimitGuard } from "../js/nestjs";

describe("Express com servidor real", () => {
  it("bloqueia a terceira requisição com 429 e headers", async () => {
    const app = express();
    app.use(rateLimitMiddleware({ limit: 2, windowSeconds: 60 }));
    app.get("/", (_req, res) => res.json({ ok: true }));
    const server: Server = await new Promise((r) => {
      const s = app.listen(0, () => r(s));
    });
    try {
      const url = `http://127.0.0.1:${(server.address() as AddressInfo).port}/`;
      const statuses = [];
      let last: Response | undefined;
      for (let i = 0; i < 3; i++) {
        last = await fetch(url);
        statuses.push(last.status);
      }
      expect(statuses).toEqual([200, 200, 429]);
      expect(last!.headers.get("x-ratelimit-remaining")).toBe("0");
      expect(Number(last!.headers.get("retry-after"))).toBeGreaterThanOrEqual(1);
      expect(await last!.json()).toEqual({ message: "Too many requests" });
    } finally {
      server.close();
    }
  });
});

describe("Fastify — rotas ao lado do plugin", () => {
  it("passam pelo limite (plugin sem encapsulamento)", async () => {
    const app = Fastify();
    await app.register(rateLimitPlugin, { limit: 1, windowSeconds: 60 });
    app.get("/", async () => ({ ok: true }));
    const first = await app.inject({ url: "/" });
    const second = await app.inject({ url: "/" });
    expect([first.statusCode, second.statusCode]).toEqual([200, 429]);
    expect(second.headers["retry-after"]).toBe("60");
    await app.close();
  });
});

describe("NestJS — RateLimitGuard", () => {
  function context(res: unknown, ip = "1.2.3.4") {
    return {
      switchToHttp: () => ({ getRequest: () => ({ ip }), getResponse: () => res }),
    } as never;
  }

  it("Express: escreve headers com setHeader e lança 429", () => {
    const guard = new RateLimitGuard({ limit: 1, windowSeconds: 60 });
    const headers: Record<string, number> = {};
    const res = { setHeader: (n: string, v: number) => (headers[n] = v) };
    expect(guard.canActivate(context(res))).toBe(true);
    expect(headers).toMatchObject({ "X-RateLimit-Limit": 1, "X-RateLimit-Remaining": 0 });
    let error: unknown;
    try {
      guard.canActivate(context(res));
    } catch (e) {
      error = e;
    }
    expect(error).toBeInstanceOf(HttpException);
    expect((error as HttpException).getStatus()).toBe(429);
    expect(headers["Retry-After"]).toBeGreaterThanOrEqual(1);
  });

  it("Fastify: escreve headers com reply.header (0.2.x os perdia)", () => {
    const guard = new RateLimitGuard({ limit: 5, windowSeconds: 60 });
    const headers: Record<string, number> = {};
    const reply = { header: (n: string, v: number) => (headers[n] = v) };
    guard.canActivate(context(reply));
    expect(headers).toMatchObject({ "X-RateLimit-Limit": 5, "X-RateLimit-Remaining": 4 });
  });

  it("sem resposta HTTP, só aplica o limite", () => {
    const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
    const guard = new RateLimitGuard({ limit: 1, windowSeconds: 60, limiter });
    expect(guard.canActivate(context(undefined))).toBe(true);
    expect(() => guard.canActivate(context(undefined))).toThrow(HttpException);
  });
});
