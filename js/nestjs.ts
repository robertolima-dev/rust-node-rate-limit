/**
 * NestJS integration.
 *
 * Usage:
 *   import { RateLimitGuard } from "rust-node-rate-limit/nestjs";
 *   app.useGlobalGuards(new RateLimitGuard({ limit: 100, windowSeconds: 60 }));
 *
 * `HttpException`/`HttpStatus` are imported at runtime (from `@nestjs/common`) to
 * respond with a proper `429`. `@nestjs/common` is an optional peerDependency,
 * needed only by users of this subpath.
 */

import { HttpException, HttpStatus } from "@nestjs/common";
import type {
  CanActivate,
  ExecutionContext,
} from "@nestjs/common";
import { RateLimiter } from "../binding.js";

export interface RateLimitGuardOptions {
  /** Maximum requests per window. */
  limit: number;
  /** Window length, in seconds. */
  windowSeconds: number;
  /** Derives the key from the HTTP request. Default: `req.ip`. */
  keyGenerator?: (req: { ip?: string }) => string;
  /** Exception message when blocked. Default: "Too many requests". */
  message?: string;
  /** Reuse an existing `RateLimiter` (ignores `limit`/`windowSeconds`). */
  limiter?: RateLimiter;
}

/**
 * NestJS rate limiting guard. Asks the native core, writes the `X-RateLimit-*`
 * headers on the response and throws `HttpException(429)` when the limit is
 * exceeded.
 */
export class RateLimitGuard implements CanActivate {
  private readonly limiter: RateLimiter;
  private readonly keyGenerator: (req: { ip?: string }) => string;
  private readonly message: string;

  constructor(options: RateLimitGuardOptions) {
    this.limiter =
      options.limiter ??
      new RateLimiter({
        limit: options.limit,
        windowSeconds: options.windowSeconds,
      });
    this.keyGenerator =
      options.keyGenerator ?? ((req) => req.ip ?? "global");
    this.message = options.message ?? "Too many requests";
  }

  canActivate(context: ExecutionContext): boolean {
    const http = context.switchToHttp();
    const req = http.getRequest<{ ip?: string }>();
    const res = http.getResponse<{
      setHeader?: (name: string, value: string | number) => void;
    }>();

    const result = this.limiter.check(this.keyGenerator(req));

    if (typeof res?.setHeader === "function") {
      res.setHeader("X-RateLimit-Limit", result.limit);
      res.setHeader("X-RateLimit-Remaining", Math.max(0, result.remaining));
      res.setHeader("X-RateLimit-Reset", result.resetAfter);
      if (!result.allowed) {
        res.setHeader("Retry-After", result.retryAfter);
      }
    }

    if (!result.allowed) {
      throw new HttpException(this.message, HttpStatus.TOO_MANY_REQUESTS);
    }

    return true;
  }
}
