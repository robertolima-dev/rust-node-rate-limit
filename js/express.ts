/**
 * Express integration.
 *
 * Usage:
 *   import { rateLimitMiddleware } from "rust-node-rate-limit/express";
 *   app.use(rateLimitMiddleware({ limit: 100, windowSeconds: 60 }));
 */

import type { Request, Response, NextFunction } from "express";
import { RateLimiter } from "./limiter";

export interface RateLimitMiddlewareOptions {
  /** Maximum requests per window. */
  limit: number;
  /** Window length, in seconds. */
  windowSeconds: number;
  /**
   * Derives the rate limit key from the request.
   * Default: the client IP (`req.ip`), falling back to `"global"`.
   */
  keyGenerator?: (req: Request) => string;
  /** JSON body returned when blocked. Default: "Too many requests". */
  message?: string;
  /** HTTP status when blocked. Default: 429. */
  statusCode?: number;
  /**
   * Reuse an existing `RateLimiter` (e.g. shared between routes).
   * When given, `limit`/`windowSeconds` are ignored.
   */
  limiter?: RateLimiter;
}

/**
 * Creates an Express rate limiting middleware. On each request it asks the
 * native core, writes the `X-RateLimit-*` headers and, when over the limit,
 * responds `429` with `Retry-After`.
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
