/**
 * Integração com Express.
 *
 * Uso:
 *   import { rateLimitMiddleware } from "rust-node-rate-limit/express";
 *   app.use(rateLimitMiddleware({ limit: 100, windowSeconds: 60 }));
 */

import type { Request, Response, NextFunction } from "express";
import { RateLimiter } from "../binding.js";

export interface RateLimitMiddlewareOptions {
  /** Máximo de requisições por janela. */
  limit: number;
  /** Duração da janela, em segundos. */
  windowSeconds: number;
  /**
   * Deriva a chave de rate limit a partir da requisição.
   * Padrão: o IP do cliente (`req.ip`), com fallback `"global"`.
   */
  keyGenerator?: (req: Request) => string;
  /** Corpo JSON retornado quando bloqueado. Padrão: "Too many requests". */
  message?: string;
  /** Status HTTP quando bloqueado. Padrão: 429. */
  statusCode?: number;
  /**
   * Reusar um `RateLimiter` já existente (ex.: compartilhado entre rotas).
   * Quando informado, `limit`/`windowSeconds` são ignorados.
   */
  limiter?: RateLimiter;
}

/**
 * Cria um middleware Express de rate limiting. Em cada requisição consulta o
 * core nativo, escreve os headers `X-RateLimit-*` e, se exceder o limite,
 * responde `429` com `Retry-After`.
 */
export function rateLimitMiddleware(options: RateLimitMiddlewareOptions) {
  const limiter =
    options.limiter ??
    new RateLimiter({
      limit: options.limit,
      windowSeconds: options.windowSeconds,
    });
  const keyGenerator =
    options.keyGenerator ?? ((req: Request) => req.ip ?? "global");
  const message = options.message ?? "Too many requests";
  const statusCode = options.statusCode ?? 429;

  return function rustNodeRateLimit(
    req: Request,
    res: Response,
    next: NextFunction,
  ): void {
    const result = limiter.check(keyGenerator(req));

    res.setHeader("X-RateLimit-Limit", result.limit);
    res.setHeader("X-RateLimit-Remaining", Math.max(0, result.remaining));
    res.setHeader("X-RateLimit-Reset", result.resetAfter);

    if (!result.allowed) {
      res.setHeader("Retry-After", result.retryAfter);
      res.status(statusCode).json({ message });
      return;
    }

    next();
  };
}
